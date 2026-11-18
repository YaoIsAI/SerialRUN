//! Minimal ANSI SGR escape-sequence parser and egui LayoutJob builder.
//!
//! Supports foreground/background colors (standard, bright, 256-color and
//! 24-bit truecolor), bold (rendered as a brighter color, matching egui's
//! `RichText::strong` behavior since egui has no bold font family), italic and
//! underline. All other CSI/OSC sequences are stripped.

use eframe::egui;
use eframe::egui::text::{LayoutJob, TextFormat};

/// One styled run of text produced by [`parse_ansi`].
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub fg: Option<egui::Color32>,
    pub bg: Option<egui::Color32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SgrState {
    fg: Option<egui::Color32>,
    bg: Option<egui::Color32>,
    bold: bool,
    italic: bool,
    underline: bool,
}

/// Standard xterm/VS Code palette (0-7 normal, 8-15 bright).
const ANSI_COLORS: [egui::Color32; 16] = [
    egui::Color32::from_rgb(0, 0, 0),
    egui::Color32::from_rgb(205, 49, 49),
    egui::Color32::from_rgb(13, 188, 121),
    egui::Color32::from_rgb(229, 229, 16),
    egui::Color32::from_rgb(36, 114, 200),
    egui::Color32::from_rgb(188, 63, 188),
    egui::Color32::from_rgb(17, 168, 205),
    egui::Color32::from_rgb(229, 229, 229),
    egui::Color32::from_rgb(102, 102, 102),
    egui::Color32::from_rgb(241, 76, 76),
    egui::Color32::from_rgb(35, 209, 139),
    egui::Color32::from_rgb(245, 245, 67),
    egui::Color32::from_rgb(59, 142, 234),
    egui::Color32::from_rgb(214, 112, 214),
    egui::Color32::from_rgb(41, 184, 219),
    egui::Color32::from_rgb(255, 255, 255),
];

/// Parse text containing ANSI escapes into styled spans.
///
/// All CSI non-SGR sequences, OSC sequences and lone ESC bytes are stripped
/// from the output; unknown SGR parameters are ignored.
pub fn parse_ansi(input: &str) -> Vec<Span> {
    let bytes = input.as_bytes();
    let mut spans: Vec<Span> = Vec::new();
    let mut state = SgrState::default();
    let mut text_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }

        flush_span(&mut spans, input, text_start, i, &state);
        i += 1;
        if i >= bytes.len() {
            text_start = i;
            break;
        }

        match bytes[i] {
            b'[' => {
                // CSI: parameters + intermediate bytes, then a final byte 0x40..=0x7e.
                i += 1;
                let params_start = i;
                while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
                    i += 1;
                }
                if i < bytes.len() {
                    if bytes[i] == b'm' {
                        apply_sgr(&mut state, &input[params_start..i]);
                    }
                    i += 1; // consume the final byte
                }
                text_start = i;
            }
            b']' => {
                // OSC: skip until BEL or ST (ESC \)。
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 0x07 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                text_start = i;
            }
            _ => {
                // Two-byte escape such as ESC ( B: drop ESC and the next byte.
                i += 1;
                text_start = i;
            }
        }
    }

    flush_span(&mut spans, input, text_start, bytes.len(), &state);
    coalesce(spans)
}

/// Remove every ANSI escape sequence, keeping only printable text.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for span in parse_ansi(input) {
        out.push_str(&span.text);
    }
    out
}

/// Expand tab characters to `tab_width`-column tab stops.
///
/// ANSI escape sequences are copied verbatim and do not consume columns, so
/// colored output keeps the same alignment as plain output. CR/LF reset the
/// column counter.
pub fn expand_tabs(input: &str, tab_width: usize) -> String {
    let tab_width = tab_width.max(1);
    let mut out = String::with_capacity(input.len());
    let mut col = 0usize;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\x1b' => {
                out.push(ch);
                match chars.next() {
                    Some('[') => {
                        out.push('[');
                        for c in chars.by_ref() {
                            out.push(c);
                            if ('\x40'..='\x7e').contains(&c) {
                                break;
                            }
                        }
                    }
                    Some(']') => {
                        out.push(']');
                        while let Some(c) = chars.next() {
                            out.push(c);
                            if c == '\x07' {
                                break;
                            }
                            if c == '\x1b' && chars.peek() == Some(&'\\') {
                                chars.next();
                                out.push('\\');
                                break;
                            }
                        }
                    }
                    Some(c) => out.push(c),
                    None => {}
                }
            }
            '\t' => {
                let spaces = tab_width - (col % tab_width);
                for _ in 0..spaces {
                    out.push(' ');
                }
                col += spaces;
            }
            '\n' => {
                out.push('\n');
                col = 0;
            }
            '\r' => {
                out.push('\r');
                col = 0;
            }
            c => {
                out.push(c);
                col += 1;
            }
        }
    }
    out
}

/// Build an egui LayoutJob. `default_fg` is used for spans without an SGR color
/// so the existing TX/RX/SYS direction colors remain visible.
pub fn layout_job(
    spans: &[Span],
    default_fg: egui::Color32,
    font_size: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    for span in spans {
        if span.text.is_empty() {
            continue;
        }
        let mut color = span.fg.unwrap_or(default_fg);
        if span.bold {
            color = brighten(color);
        }
        let format = TextFormat {
            font_id: egui::FontId::monospace(font_size),
            color,
            background: span.bg.unwrap_or(egui::Color32::TRANSPARENT),
            italics: span.italic,
            underline: if span.underline {
                egui::Stroke::new(1.0, color)
            } else {
                egui::Stroke::NONE
            },
            ..Default::default()
        };
        job.append(&span.text, 0.0, format);
    }
    job
}

fn flush_span(spans: &mut Vec<Span>, input: &str, start: usize, end: usize, state: &SgrState) {
    if start >= end {
        return;
    }
    let text = &input[start..end];
    if text.is_empty() {
        return;
    }
    spans.push(Span {
        text: text.to_string(),
        fg: state.fg,
        bg: state.bg,
        bold: state.bold,
        italic: state.italic,
        underline: state.underline,
    });
}

/// Merge adjacent spans that share identical styling.
fn coalesce(spans: Vec<Span>) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = out.last_mut() {
            if last.fg == span.fg
                && last.bg == span.bg
                && last.bold == span.bold
                && last.italic == span.italic
                && last.underline == span.underline
            {
                last.text.push_str(&span.text);
                continue;
            }
        }
        out.push(span);
    }
    out
}

fn apply_sgr(state: &mut SgrState, params: &str) {
    let mut nums: Vec<u32> = params
        .split(';')
        .map(|p| p.trim().parse::<u32>().unwrap_or(0))
        .collect();
    if nums.is_empty() {
        nums.push(0);
    }

    let mut i = 0;
    while i < nums.len() {
        match nums[i] {
            0 => *state = SgrState::default(),
            1 => state.bold = true,
            22 => state.bold = false,
            3 => state.italic = true,
            23 => state.italic = false,
            4 => state.underline = true,
            24 => state.underline = false,
            30..=37 => state.fg = Some(ANSI_COLORS[(nums[i] - 30) as usize]),
            39 => state.fg = None,
            40..=47 => state.bg = Some(ANSI_COLORS[(nums[i] - 40) as usize]),
            49 => state.bg = None,
            90..=97 => state.fg = Some(ANSI_COLORS[(nums[i] - 90 + 8) as usize]),
            100..=107 => state.bg = Some(ANSI_COLORS[(nums[i] - 100 + 8) as usize]),
            38 | 48 => {
                let is_fg = nums[i] == 38;
                if i + 2 < nums.len() && nums[i + 1] == 5 {
                    let color = color_256(nums[i + 2]);
                    if is_fg {
                        state.fg = Some(color);
                    } else {
                        state.bg = Some(color);
                    }
                    i += 2;
                } else if i + 4 < nums.len() && nums[i + 1] == 2 {
                    let color = egui::Color32::from_rgb(
                        nums[i + 2].min(255) as u8,
                        nums[i + 3].min(255) as u8,
                        nums[i + 4].min(255) as u8,
                    );
                    if is_fg {
                        state.fg = Some(color);
                    } else {
                        state.bg = Some(color);
                    }
                    i += 4;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

fn color_256(index: u32) -> egui::Color32 {
    match index {
        0..=15 => ANSI_COLORS[index as usize],
        16..=231 => {
            let n = index - 16;
            let level = |v: u32| -> u8 {
                match v {
                    0 => 0,
                    1 => 95,
                    2 => 135,
                    3 => 175,
                    4 => 215,
                    _ => 255,
                }
            };
            egui::Color32::from_rgb(
                level(n / 36),
                level((n % 36) / 6),
                level(n % 6),
            )
        }
        232..=255 => {
            let v = (8 + (index - 232) * 10) as u8;
            egui::Color32::from_rgb(v, v, v)
        }
        _ => egui::Color32::from_rgb(255, 255, 255),
    }
}

/// egui has no bold font family; `RichText::strong` only brightens the color.
/// Match that behavior for SGR bold.
fn brighten(color: egui::Color32) -> egui::Color32 {
    let mix = |v: u8| -> u8 { (v as u16 + ((255 - v as u16) * 30 / 100)) as u8 };
    egui::Color32::from_rgb(mix(color.r()), mix(color.g()), mix(color.b()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_single_span() {
        let spans = parse_ansi("hello");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "hello");
        assert!(spans[0].fg.is_none());
        assert!(!spans[0].bold);
    }

    #[test]
    fn red_then_reset() {
        let spans = parse_ansi("\x1b[31mRED\x1b[0m plain");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "RED");
        assert_eq!(spans[0].fg, Some(ANSI_COLORS[1]));
        assert_eq!(spans[1].text, " plain");
        assert!(spans[1].fg.is_none());
    }

    #[test]
    fn bold_green() {
        let spans = parse_ansi("\x1b[1;32mok");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "ok");
        assert!(spans[0].bold);
        assert_eq!(spans[0].fg, Some(ANSI_COLORS[2]));
    }

    #[test]
    fn non_sgr_csi_is_stripped() {
        let spans = parse_ansi("ab\x1b[2;5Hcd");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "abcd");
    }

    #[test]
    fn color_256_red() {
        let spans = parse_ansi("\x1b[38;5;196mx");
        assert_eq!(spans[0].fg, Some(egui::Color32::from_rgb(255, 0, 0)));
    }

    #[test]
    fn bright_red() {
        let spans = parse_ansi("\x1b[91mx");
        assert_eq!(spans[0].fg, Some(ANSI_COLORS[9]));
    }

    #[test]
    fn underline_toggle() {
        let spans = parse_ansi("\x1b[4munder\x1b[24m plain");
        assert_eq!(spans.len(), 2);
        assert!(spans[0].underline);
        assert!(!spans[1].underline);
    }

    #[test]
    fn unknown_param_ignored() {
        let spans = parse_ansi("\x1b[99mfoo");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "foo");
        assert!(spans[0].fg.is_none());
    }

    #[test]
    fn truncated_sequence_is_safe() {
        let spans = parse_ansi("abc\x1b[31");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "abc");
    }

    #[test]
    fn tabs_expand_to_next_stop() {
        assert_eq!(expand_tabs("a\tb", 8), "a       b");
        assert_eq!(expand_tabs("\tx", 8), "        x");
        assert_eq!(expand_tabs("abcdefgh\tx", 8), "abcdefgh        x");
    }

    #[test]
    fn tabs_reset_after_newline() {
        assert_eq!(expand_tabs("ab\nc\td", 8), "ab\nc       d");
    }

    #[test]
    fn tabs_do_not_count_ansi() {
        assert_eq!(
            expand_tabs("\x1b[31mab\tc\x1b[0m", 8),
            "\x1b[31mab      c\x1b[0m"
        );
    }
}

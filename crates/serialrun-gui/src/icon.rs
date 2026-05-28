pub fn generate_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../icon_embedded.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    Some(egui::IconData {
        width: info.width,
        height: info.height,
        rgba: buf,
    })
}

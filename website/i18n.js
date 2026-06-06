// ── Language Configuration ──
const LANGUAGES = [
    { code: 'en',    name: 'English',       htmlLang: 'en' },
    { code: 'zh',    name: '简体中文',       htmlLang: 'zh-CN' },
    { code: 'zh-TW', name: '繁體中文',       htmlLang: 'zh-TW' },
    { code: 'ja',    name: '日本語',         htmlLang: 'ja' },
    { code: 'ko',    name: '한국어',         htmlLang: 'ko' },
    { code: 'fr',    name: 'Français',      htmlLang: 'fr' },
    { code: 'de',    name: 'Deutsch',       htmlLang: 'de' },
    { code: 'es',    name: 'Español',       htmlLang: 'es' },
    { code: 'pt-BR', name: 'Português (BR)', htmlLang: 'pt-BR' },
];

const LANG_CODES = LANGUAGES.map(l => l.code);

// ── Translation Registry ──
const _translations = {};

function registerTranslations(obj) {
    for (const lang of LANG_CODES) {
        if (obj[lang]) {
            if (!_translations[lang]) _translations[lang] = {};
            Object.assign(_translations[lang], obj[lang]);
        }
    }
}

// ── Language Detection ──
function detectLanguage() {
    const saved = localStorage.getItem('lang');
    if (saved && LANG_CODES.includes(saved)) return saved;

    const browsers = navigator.languages || [navigator.language];
    for (const tag of browsers) {
        // Exact match
        if (LANG_CODES.includes(tag)) return tag;
        // Prefix match: "zh-CN" -> "zh", "ja-JP" -> "ja"
        const base = tag.split('-')[0];
        if (LANG_CODES.includes(base)) return base;
        // Special: "pt" -> "pt-BR"
        if (base === 'pt' && LANG_CODES.includes('pt-BR')) return 'pt-BR';
    }
    return 'en';
}

// ── Translation Lookup ──
function getTranslation(lang, key) {
    if (_translations[lang] && _translations[lang][key] !== undefined) {
        return _translations[lang][key];
    }
    if (_translations['en'] && _translations['en'][key] !== undefined) {
        return _translations['en'][key];
    }
    return key;
}

// ── Apply Language ──
let currentLang = 'en';

function applyLang(lang) {
    if (!LANG_CODES.includes(lang)) lang = 'en';
    currentLang = lang;
    localStorage.setItem('lang', lang);

    const langObj = LANGUAGES.find(l => l.code === lang);
    document.documentElement.setAttribute('data-lang', lang);
    document.documentElement.setAttribute('lang', langObj ? langObj.htmlLang : 'en');

    // Translate all [data-i18n] elements
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        const text = getTranslation(lang, key);
        if (el.tagName === 'INPUT' && el.hasAttribute('placeholder')) {
            el.placeholder = text;
        } else {
            el.innerHTML = text;
        }
    });

    // Toggle language-specific elements (.lang-en, .lang-zh, etc.)
    LANGUAGES.forEach(l => {
        document.querySelectorAll('.lang-' + l.code.replace('-', '')).forEach(el => {
            el.style.display = lang === l.code ? '' : 'none';
        });
    });

    // Toggle screenshots (fallback to English)
    const imgEn = document.getElementById('hero_screenshot_en');
    const imgZh = document.getElementById('hero_screenshot_zh');
    if (imgEn) imgEn.style.display = lang === 'en' ? '' : 'none';
    if (imgZh) imgZh.style.display = lang === 'zh' ? '' : 'none';

    // Update page title
    const titleKey = '_title_' + lang;
    const titleEn = getTranslation(lang, '_title_en');
    const titleZh = getTranslation(lang, '_title_zh');
    if (titleEn || titleZh) {
        document.title = getTranslation(lang, titleKey) || titleEn || titleZh || document.title;
    }

    // Update language picker UI
    updatePickerUI(lang);

    // Dispatch event for pages that need to react
    window.dispatchEvent(new CustomEvent('langchange', { detail: { lang } }));
}

// ── Language Picker ──
function createLanguagePicker() {
    // Find all existing toggle buttons and replace them
    document.querySelectorAll('.lang-toggle').forEach(oldBtn => {
        const picker = document.createElement('div');
        picker.className = 'lang-picker';
        picker.id = 'langPicker';

        const currentLangObj = LANGUAGES.find(l => l.code === currentLang) || LANGUAGES[0];

        picker.innerHTML = `
            <button class="lang-toggle" id="langToggle" aria-label="Change language">
                <span id="langCurrent">${currentLangObj.name}</span>
                <svg class="lang-arrow" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
            </button>
            <div class="lang-dropdown" id="langDropdown">
                ${LANGUAGES.map(l => `
                    <button data-lang="${l.code}" class="lang-option${l.code === currentLang ? ' active' : ''}">${l.name}</button>
                `).join('')}
            </div>
        `;

        oldBtn.replaceWith(picker);
    });

    // Toggle dropdown
    const toggle = document.getElementById('langToggle');
    const picker = document.getElementById('langPicker');
    if (toggle && picker) {
        toggle.addEventListener('click', (e) => {
            e.stopPropagation();
            picker.classList.toggle('open');
        });
    }

    // Language selection
    document.querySelectorAll('.lang-option').forEach(btn => {
        btn.addEventListener('click', () => {
            const lang = btn.getAttribute('data-lang');
            applyLang(lang);
            if (picker) picker.classList.remove('open');
        });
    });

    // Close on click outside
    document.addEventListener('click', () => {
        if (picker) picker.classList.remove('open');
    });

    // Close on Escape
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && picker) picker.classList.remove('open');
    });
}

function updatePickerUI(lang) {
    const langObj = LANGUAGES.find(l => l.code === lang);
    const label = document.getElementById('langCurrent');
    if (label && langObj) label.textContent = langObj.name;

    document.querySelectorAll('.lang-option').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-lang') === lang);
    });
}

// ── Initialize ──
document.addEventListener('DOMContentLoaded', () => {
    currentLang = detectLanguage();
    applyLang(currentLang);
    createLanguagePicker();
});

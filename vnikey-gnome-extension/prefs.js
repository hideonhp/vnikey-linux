import Adw from 'gi://Adw';
import Gtk from 'gi://Gtk';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

const CONFIG_PATH = GLib.build_filenamev([
    GLib.get_home_dir(), '.config', 'vnikey', 'config.toml'
]);

function readToml() {
    try {
        const file = Gio.File.new_for_path(CONFIG_PATH);
        const [ok, contents] = file.load_contents(null);
        return ok ? new TextDecoder().decode(contents) : '';
    } catch (_) { return ''; }
}

function getTomlValue(toml, key) {
    const m = toml.match(new RegExp(`^${key}\\s*=\\s*"?([^"\\n]+)"?`, 'm'));
    return m ? m[1].trim() : null;
}

function setTomlValue(toml, key, value) {
    const val = typeof value === 'string' ? `"${value}"` : String(value);
    const re = new RegExp(`^(${key}\\s*=\\s*).*$`, 'm');
    return toml.match(re)
        ? toml.replace(re, `$1${val}`)
        : `${toml}\n${key} = ${val}`;
}

function writeToml(content) {
    try {
        const file = Gio.File.new_for_path(CONFIG_PATH);
        const bytes = new TextEncoder().encode(content);
        file.replace_contents(bytes, null, false,
            Gio.FileCreateFlags.REPLACE_DESTINATION, null);
    } catch (e) {
        console.error('[vnikey prefs] write error:', e);
    }
}

export default class VnikeyPrefs {
    fillPreferencesWindow(window) {
        const page = new Adw.PreferencesPage({ title: 'VNIKey' });

        // === Nhóm: Cấu hình bộ gõ ===
        const inputGroup = new Adw.PreferencesGroup({ title: 'Cấu hình bộ gõ' });

        const toml = readToml();

        // Input method: Telex / VNI / VIQR
        const methodRow = new Adw.ComboRow({
            title: 'Kiểu gõ',
            subtitle: 'Telex, VNI hoặc VIQR',
        });
        const methods = new Gtk.StringList({ strings: ['Telex', 'VNI', 'VIQR'] });
        methodRow.model = methods;
        const curMethod = (getTomlValue(toml, 'input_method') ?? 'telex').toLowerCase();
        methodRow.selected = curMethod === 'vni' ? 1 : curMethod === 'viqr' ? 2 : 0;
        methodRow.connect('notify::selected', () => {
            const m = ['telex', 'vni', 'viqr'][methodRow.selected] ?? 'telex';
            writeToml(setTomlValue(readToml(), 'input_method', m));
        });

        // Spell check
        const spellRow = new Adw.SwitchRow({
            title: 'Kiểm tra chính tả (Smart Spell Check)',
        });
        spellRow.active = getTomlValue(toml, 'spell_check') !== 'false';
        spellRow.connect('notify::active', () => {
            writeToml(setTomlValue(readToml(), 'spell_check', spellRow.active));
        });

        // Vim mode
        const vimRow = new Adw.SwitchRow({
            title: 'Vim Mode',
            subtitle: 'ESC tự động tắt tiếng Việt',
        });
        vimRow.active = getTomlValue(toml, 'vim_mode') === 'true';
        vimRow.connect('notify::active', () => {
            writeToml(setTomlValue(readToml(), 'vim_mode', vimRow.active));
        });

        // Per-window state
        const perWindowRow = new Adw.SwitchRow({
            title: 'Per-window state',
            subtitle: 'Nhớ VI/EN theo từng cửa sổ',
        });
        perWindowRow.active = getTomlValue(toml, 'per_window_state') === 'true';
        perWindowRow.connect('notify::active', () => {
            writeToml(setTomlValue(readToml(), 'per_window_state', perWindowRow.active));
        });

        inputGroup.add(methodRow);
        inputGroup.add(spellRow);
        inputGroup.add(vimRow);
        inputGroup.add(perWindowRow);

        // === Nhóm: Thông báo & UX ===
        const uxGroup = new Adw.PreferencesGroup({ title: 'Thông báo & UX' });

        // Notification enabled
        const notifRow = new Adw.SwitchRow({
            title: 'Thông báo Desktop',
            subtitle: 'Hiện thông báo khi bật/tắt tiếng Việt',
        });
        notifRow.active = getTomlValue(toml, 'notification_enabled') !== 'false';
        notifRow.connect('notify::active', () => {
            writeToml(setTomlValue(readToml(), 'notification_enabled', notifRow.active));
        });

        // Start enabled
        const startRow = new Adw.SwitchRow({
            title: 'Khởi động ở chế độ Tiếng Việt',
        });
        startRow.active = getTomlValue(toml, 'start_enabled') !== 'false';
        startRow.connect('notify::active', () => {
            writeToml(setTomlValue(readToml(), 'start_enabled', startRow.active));
        });

        uxGroup.add(notifRow);
        uxGroup.add(startRow);

        page.add(inputGroup);
        page.add(uxGroup);
        window.add(page);
    }
}

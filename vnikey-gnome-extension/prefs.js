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
        const group = new Adw.PreferencesGroup({ title: 'Cấu hình bộ gõ' });

        const toml = readToml();

        // Input method
        const methodRow = new Adw.ComboRow({
            title: 'Kiểu gõ',
            subtitle: 'Telex hoặc VNI',
        });
        const methods = new Gtk.StringList({ strings: ['Telex', 'VNI'] });
        methodRow.model = methods;
        const curMethod = getTomlValue(toml, 'input_method') ?? 'telex';
        methodRow.selected = curMethod.toLowerCase() === 'vni' ? 1 : 0;
        methodRow.connect('notify::selected', () => {
            const updated = setTomlValue(readToml(), 'input_method',
                methodRow.selected === 1 ? 'vni' : 'telex');
            writeToml(updated);
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

        group.add(methodRow);
        group.add(spellRow);
        group.add(vimRow);
        group.add(perWindowRow);
        page.add(group);
        window.add(page);
    }
}

import St from 'gi://St';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

const StateInterface = `
<node>
  <interface name="org.vnikey.State">
    <method name="GetState">
      <arg type="b" direction="out"/>
    </method>
    <method name="ToggleState" />
    <method name="SetInputMethod">
      <arg type="s" direction="in"/>
    </method>
    <method name="GetInputMethod">
      <arg type="s" direction="out"/>
    </method>
    <signal name="StateChanged">
      <arg type="b"/>
    </signal>
  </interface>
</node>`;

const WaylandInterface = `
<node>
  <interface name="org.vnikey.WaylandIntegration">
    <method name="NotifyActiveWindow">
      <arg type="s" direction="in"/>
    </method>
  </interface>
</node>`;

const StateProxy = Gio.DBusProxy.makeProxyWrapper(StateInterface);
const WaylandProxy = Gio.DBusProxy.makeProxyWrapper(WaylandInterface);

export default class VnikeyIndicatorExtension {
    enable() {
        this._indicator = new PanelMenu.Button(0.0, 'VnikeyIndicator', false);
        this._label = new St.Label({
            text: '...',
            y_align: St.Align.MIDDLE,
        });
        this._indicator.add_child(this._label);

        // Right-click menu: Toggle VI/EN + chọn kiểu gõ
        this._buildMenu();

        Main.panel.addToStatusArea('VnikeyIndicator', this._indicator);

        this._dbusProxy = null;
        this._waylandProxy = null;
        this._signalId = 0;
        this._windowFocusId = 0;
        this._nameWatcherId = 0;
        this._initDBus();

        // Left-click: toggle VI/EN
        this._indicator.connect('button-press-event', (_actor, event) => {
            // button 1 = left, button 3 = right
            if (event.get_button() === 1) {
                if (this._dbusProxy) {
                    this._dbusProxy.ToggleStateRemote((_result, error) => {
                        if (error) {
                            console.error('Error toggling VNIKey state:', error);
                        }
                        // StateChanged signal sẽ tự cập nhật label — không cần _updateState()
                    });
                }
                return true; // prevent menu from opening on left click
            }
            return false; // allow right-click to open menu
        });

        this._windowFocusId = global.display.connect('notify::focus-window', () => {
            if (this._waylandProxy) {
                try {
                    const focusWindow = global.display.focus_window;
                    if (focusWindow) {
                        let appId = focusWindow.get_wm_class()
                            || focusWindow.get_wm_class_instance()
                            || focusWindow.get_gtk_application_id?.()
                            || focusWindow.get_title?.();
                        if (appId) {
                            this._waylandProxy.NotifyActiveWindowRemote(appId, () => {});
                        }
                    }
                } catch (e) {
                    console.error('Failed to get active window class:', e);
                }
            }
        });

        // Theo dõi khi vnikey-ibus restart / xuất hiện trở lại trên D-Bus
        this._nameWatcherId = Gio.DBus.session.signal_subscribe(
            'org.freedesktop.DBus',
            'org.freedesktop.DBus',
            'NameOwnerChanged',
            '/org/freedesktop/DBus',
            'org.vnikey.State',
            Gio.DBusSignalFlags.NONE,
            (_conn, _sender, _path, _iface, _signal, params) => {
                const [_name, _oldOwner, newOwner] = params.deepUnpack();
                if (newOwner && newOwner !== '') {
                    // vnikey-ibus vừa restart hoặc launch: reconnect proxy
                    console.log('[VNIKey] vnikey-ibus appeared, reconnecting...');
                    this._reconnectDBus();
                }
            }
        );
    }

    _buildMenu() {
        // Toggle item
        this._toggleItem = new PopupMenu.PopupMenuItem('Bật/Tắt Tiếng Việt');
        this._toggleItem.connect('activate', () => {
            if (this._dbusProxy) {
                this._dbusProxy.ToggleStateRemote((_r, err) => {
                    if (err) console.error('[VNIKey] ToggleState error:', err);
                    // StateChanged signal tự cập nhật label
                });
            }
        });
        this._indicator.menu.addMenuItem(this._toggleItem);
        this._indicator.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Kiểu gõ: Telex / VNI / VIQR
        this._telexItem = new PopupMenu.PopupMenuItem('Telex');
        this._telexItem.connect('activate', () => this._setInputMethod('telex'));
        this._indicator.menu.addMenuItem(this._telexItem);

        this._vniItem = new PopupMenu.PopupMenuItem('VNI');
        this._vniItem.connect('activate', () => this._setInputMethod('vni'));
        this._indicator.menu.addMenuItem(this._vniItem);

        this._viqrItem = new PopupMenu.PopupMenuItem('VIQR');
        this._viqrItem.connect('activate', () => this._setInputMethod('viqr'));
        this._indicator.menu.addMenuItem(this._viqrItem);
    }

    _setInputMethod(method) {
        if (this._dbusProxy) {
            this._dbusProxy.SetInputMethodRemote(method, (_r, err) => {
                if (err) {
                    console.error('[VNIKey] SetInputMethod error:', err);
                } else {
                    // Update menu checkmarks
                    this._updateMethodMenuItems(method);
                }
            });
        }
    }

    /// Cập nhật checkmark/bold trên menu items để hiển thị kiểu gõ đang dùng.
    _updateMethodMenuItems(activeMethod) {
        const items = {
            telex: this._telexItem,
            vni: this._vniItem,
            viqr: this._viqrItem,
        };
        for (const [method, item] of Object.entries(items)) {
            if (!item) continue;
            const isActive = method === activeMethod?.toLowerCase();
            // Thêm '●' prefix cho item đang active để dễ nhận ra
            const label = method.toUpperCase();
            item.label.set_text(isActive ? `● ${label}` : label);
        }
    }

    _initDBus() {
        new StateProxy(
            Gio.DBus.session,
            'org.vnikey.State',
            '/org/vnikey/State',
            (proxy, error) => {
                if (error) {
                    console.error('Failed to connect to VNIKey DBus:', error);
                    return;
                }
                this._dbusProxy = proxy;
                this._updateState();
                this._updateCurrentMethod();

                this._signalId = this._dbusProxy.connectSignal('StateChanged', (_proxy, _sender, [state]) => {
                    this._label.set_text(state ? 'V' : 'E');
                });
            }
        );

        new WaylandProxy(
            Gio.DBus.session,
            'org.vnikey.WaylandIntegration',
            '/org/vnikey/WaylandIntegration',
            (proxy, error) => {
                if (error) {
                    console.error('Failed to connect to VNIKey Wayland DBus:', error);
                    return;
                }
                this._waylandProxy = proxy;
            }
        );
    }

    _reconnectDBus() {
        // Ngắt kết nối cũ
        if (this._dbusProxy && this._signalId) {
            this._dbusProxy.disconnectSignal(this._signalId);
            this._signalId = 0;
        }
        this._dbusProxy = null;
        this._label.set_text('...');

        // Reconnect sau 500ms để IBus có thời gian register xong D-Bus name
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
            this._initDBus();
            return GLib.SOURCE_REMOVE;
        });
    }

    _updateState() {
        if (!this._dbusProxy) return;

        this._dbusProxy.GetStateRemote((result, error) => {
            if (error) {
                console.error('Error getting VNIKey state:', error);
                this._label.set_text('?');
            } else {
                const isVi = result[0];
                this._label.set_text(isVi ? 'V' : 'E');
            }
        });
    }

    _updateCurrentMethod() {
        if (!this._dbusProxy) return;

        this._dbusProxy.GetInputMethodRemote((result, error) => {
            if (!error && result) {
                this._updateMethodMenuItems(result[0]);
            }
        });
    }

    disable() {
        if (this._nameWatcherId) {
            Gio.DBus.session.signal_unsubscribe(this._nameWatcherId);
            this._nameWatcherId = 0;
        }
        if (this._windowFocusId) {
            global.display.disconnect(this._windowFocusId);
            this._windowFocusId = 0;
        }
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
        if (this._dbusProxy) {
            if (this._signalId) {
                this._dbusProxy.disconnectSignal(this._signalId);
                this._signalId = 0;
            }
        }
        this._dbusProxy = null;
        this._waylandProxy = null;
        this._toggleItem = null;
        this._telexItem = null;
        this._vniItem = null;
        this._viqrItem = null;
    }
}

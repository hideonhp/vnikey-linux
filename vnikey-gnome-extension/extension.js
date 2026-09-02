import St from 'gi://St';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';

const StateInterface = `
<node>
  <interface name="org.vnikey.State">
    <method name="GetState">
      <arg type="b" direction="out"/>
    </method>
    <method name="ToggleState" />
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
        Main.panel.addToStatusArea('VnikeyIndicator', this._indicator);

        this._dbusProxy = null;
        this._waylandProxy = null;
        this._signalId = 0;
        this._windowFocusId = 0;
        this._initDBus();

        this._indicator.connect('button-press-event', () => {
            if (this._dbusProxy) {
                this._dbusProxy.ToggleStateRemote((result, error) => {
                    if (error) {
                        console.error('Error toggling VNIKey state:', error);
                    } else {
                        // Optimistically or explicitly call get_state after toggle
                        this._updateState();
                    }
                });
            }
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

                this._signalId = this._dbusProxy.connectSignal('StateChanged', (proxy, sender, [state]) => {
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

    _updateState() {
        if (!this._dbusProxy) return;

        this._dbusProxy.GetStateRemote((result, error) => {
            if (error) {
                console.error('Error getting VNIKey state:', error);
                this._label.set_text('?');
            } else {
                const isVi = result[0]; // Output args are returned as array
                this._label.set_text(isVi ? 'V' : 'E');
            }
        });
    }

    disable() {
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
    }
}

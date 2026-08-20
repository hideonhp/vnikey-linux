import St from 'gi://St';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';

const StateInterface = `
<node>
  <interface name="org.vnikey.State">
    <method name="get_state">
      <arg type="b" direction="out"/>
    </method>
    <method name="toggle_state" />
  </interface>
</node>`;

const StateProxy = Gio.DBusProxy.makeProxyWrapper(StateInterface);

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
        this._initDBus();

        this._indicator.connect('button-press-event', () => {
            if (this._dbusProxy) {
                this._dbusProxy.toggle_stateRemote((result, error) => {
                    if (error) {
                        console.error('Error toggling VNIKey state:', error);
                    } else {
                        // Optimistically or explicitly call get_state after toggle
                        this._updateState();
                    }
                });
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
            }
        );
    }

    _updateState() {
        if (!this._dbusProxy) return;

        this._dbusProxy.get_stateRemote((result, error) => {
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
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
        this._dbusProxy = null;
    }
}

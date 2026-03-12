import Foundation
import Capacitor

@objc(NativeHapticsPlugin)
public class NativeHapticsPlugin: CAPPlugin, CAPBridgedPlugin {
    public let identifier = "NativeHapticsPlugin"
    public let jsName = "NativeHaptics"
    public let pluginMethods: [CAPPluginMethod] = [
        CAPPluginMethod(name: "light", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "hard", returnType: CAPPluginReturnPromise),
    ]

    private let haptics = HapticsService()

    @objc override public func load() {
        DispatchQueue.main.async {
            self.haptics.prepare()
        }
    }

    @objc func light(_ call: CAPPluginCall) {
        DispatchQueue.main.async {
            self.haptics.light()
            call.resolve()
        }
    }

    @objc func hard(_ call: CAPPluginCall) {
        DispatchQueue.main.async {
            self.haptics.hard()
            call.resolve()
        }
    }
}

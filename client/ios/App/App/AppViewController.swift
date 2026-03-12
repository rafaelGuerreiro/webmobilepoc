import UIKit
import Capacitor

class AppViewController: CAPBridgeViewController {
    override open func instanceDescriptor() -> InstanceDescriptor {
        let descriptor = super.instanceDescriptor()

        if let serverHost = Bundle.main.object(forInfoDictionaryKey: "CAP_SERVER_HOST") as? String {
            let trimmedServerHost = serverHost.trimmingCharacters(in: .whitespacesAndNewlines)
            if !trimmedServerHost.isEmpty {
                let serverPort = (Bundle.main.object(forInfoDictionaryKey: "CAP_SERVER_PORT") as? String)?
                    .trimmingCharacters(in: .whitespacesAndNewlines) ?? "5173"
                descriptor.serverURL = "http://\(trimmedServerHost):\(serverPort)"
                descriptor.allowedNavigationHostnames.append(trimmedServerHost)
            }
        }

        return descriptor
    }

    override open func capacitorDidLoad() {
        super.capacitorDidLoad()
        bridge?.registerPluginInstance(NativeHapticsPlugin())
    }
}

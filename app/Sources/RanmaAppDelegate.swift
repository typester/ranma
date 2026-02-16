import AppKit

class RanmaAppDelegate: NSObject, NSApplicationDelegate {
    private var viewModel: BarViewModel!

    func applicationDidFinishLaunching(_ notification: Notification) {
        viewModel = BarViewModel()
        registerHandler(handler: viewModel)

        updateDisplayList()

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(screenParametersChanged),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )

        let socketPath = defaultSocketPath()
        DispatchQueue.global(qos: .utility).async {
            startServer(socketPath: socketPath)
        }
    }

    @objc private func screenParametersChanged(_ notification: Notification) {
        updateDisplayList()
    }

    private func updateDisplayList() {
        let displays = NSScreen.screens.map { screen in
            DisplayInfo(
                id: screen.displayID,
                name: screen.localizedName,
                isMain: screen == NSScreen.main
            )
        }
        setDisplays(displays: displays)
    }

    private func defaultSocketPath() -> String {
        let uid = getuid()
        let tmpDir = NSTemporaryDirectory()
        return "\(tmpDir)ranma_\(uid).sock"
    }
}

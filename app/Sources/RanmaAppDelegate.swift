import AppKit

class RanmaAppDelegate: NSObject, NSApplicationDelegate {
    private var viewModel: BarViewModel!

    func applicationDidFinishLaunching(_ notification: Notification) {
        viewModel = BarViewModel()
        registerHandler(handler: viewModel)

        updateDisplayList()
        viewModel.updateFullscreenState()

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(screenParametersChanged),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )

        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(activeSpaceDidChange),
            name: NSWorkspace.activeSpaceDidChangeNotification,
            object: nil
        )

        let socketPath = defaultSocketPath()
        DispatchQueue.global(qos: .utility).async {
            startServer(socketPath: socketPath)
        }

        DispatchQueue.global(qos: .utility).async {
            runInitScript(socketPath: socketPath)
        }
    }

    @MainActor @objc private func screenParametersChanged(_ notification: Notification) {
        updateDisplayList()
        let activeIDs = Set(NSScreen.screens.map { $0.displayID })
        viewModel.handleDisplayChange(activeDisplayIDs: activeIDs)
        viewModel.updateFullscreenState()
    }

    @MainActor @objc private func activeSpaceDidChange(_ notification: Notification) {
        viewModel.updateFullscreenState()
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

private func runInitScript(socketPath: String) {
    for _ in 0..<40 {
        if FileManager.default.fileExists(atPath: socketPath) { break }
        Thread.sleep(forTimeInterval: 0.05)
    }

    let initPath: String
    if let envPath = ProcessInfo.processInfo.environment["RANMA_INIT"] {
        initPath = (envPath as NSString).expandingTildeInPath
    } else {
        initPath = NSString("~/.config/ranma/init").expandingTildeInPath
    }
    guard FileManager.default.isExecutableFile(atPath: initPath) else { return }

    eprintln("running init: \(initPath)")
    let process = Process()
    process.executableURL = URL(fileURLWithPath: initPath)
    process.standardOutput = FileHandle.standardError
    process.standardError = FileHandle.standardError
    try? process.run()
    process.waitUntilExit()
}

private func eprintln(_ message: String) {
    FileHandle.standardError.write(Data((message + "\n").utf8))
}

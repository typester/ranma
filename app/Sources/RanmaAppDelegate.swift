import AppKit

class RanmaAppDelegate: NSObject, NSApplicationDelegate {
    private var barWindow: BarWindow!
    private var contentView: BarContentView!
    private var viewModel: BarViewModel!

    func applicationDidFinishLaunching(_ notification: Notification) {
        contentView = BarContentView()
        barWindow = BarWindow()
        barWindow.contentView = contentView

        viewModel = BarViewModel(window: barWindow, contentView: contentView)
        registerHandler(handler: viewModel)

        barWindow.updateFrame(contentSize: contentView.intrinsicContentSize, animate: false)
        barWindow.orderFrontRegardless()

        let socketPath = defaultSocketPath()
        DispatchQueue.global(qos: .utility).async {
            startServer(socketPath: socketPath)
        }
    }

    private func defaultSocketPath() -> String {
        let uid = getuid()
        let tmpDir = NSTemporaryDirectory()
        return "\(tmpDir)ranma_\(uid).sock"
    }
}

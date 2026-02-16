import AppKit

class BarWindow: NSWindow {
    init() {
        super.init(
            contentRect: .zero,
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.statusWindow)))
        collectionBehavior = [.canJoinAllSpaces, .stationary]
    }

    func updateFrame(contentSize: NSSize, animate: Bool) {
        guard let screen = NSScreen.main else { return }
        let screenFrame = screen.frame
        let x = screenFrame.midX - contentSize.width / 2
        let y = screenFrame.maxY - contentSize.height - 2
        let newFrame = NSRect(
            x: x, y: y,
            width: contentSize.width,
            height: contentSize.height
        )
        setFrame(newFrame, display: true, animate: animate)
    }
}

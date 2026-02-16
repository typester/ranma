import AppKit

class BarWindow: NSWindow {
    let displayID: CGDirectDisplayID

    init(screen: NSScreen) {
        self.displayID = screen.displayID
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
        guard let screen = screenForDisplay() else { return }
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

    private func screenForDisplay() -> NSScreen? {
        NSScreen.screens.first { $0.displayID == displayID }
    }
}

extension NSScreen {
    var displayID: CGDirectDisplayID {
        (deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? 0
    }
}

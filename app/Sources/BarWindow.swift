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
        hasShadow = false
        level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.statusWindow)))
        collectionBehavior = [.canJoinAllSpaces, .stationary]
    }

    enum Alignment: Hashable {
        case left, center, right
    }

    func updateFrame(contentSize: NSSize, alignment: Alignment, animate: Bool) {
        guard let screen = screenForDisplay() else { return }
        let screenFrame = screen.frame
        let x: CGFloat
        let y: CGFloat
        switch alignment {
        case .center:
            x = screenFrame.midX - contentSize.width / 2
            let visibleFrame = screen.visibleFrame
            let menuBarHeight = screenFrame.maxY - visibleFrame.maxY
            y = screenFrame.maxY - menuBarHeight + floor((menuBarHeight - contentSize.height) / 2)
        case .left:
            if let area = screen.auxiliaryTopLeftArea {
                x = area.maxX - contentSize.width
                y = area.midY - contentSize.height / 2
            } else {
                x = screenFrame.midX - contentSize.width / 2
                let visibleFrame = screen.visibleFrame
                let menuBarHeight = screenFrame.maxY - visibleFrame.maxY
                y = screenFrame.maxY - menuBarHeight + (menuBarHeight - contentSize.height) / 2
            }
        case .right:
            if let area = screen.auxiliaryTopRightArea {
                x = area.minX
                y = area.midY - contentSize.height / 2
            } else {
                x = screenFrame.midX - contentSize.width / 2
                let visibleFrame = screen.visibleFrame
                let menuBarHeight = screenFrame.maxY - visibleFrame.maxY
                y = screenFrame.maxY - menuBarHeight + (menuBarHeight - contentSize.height) / 2
            }
        }
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

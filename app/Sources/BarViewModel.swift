import AppKit

// @unchecked Sendable: required by UniFFI's StateChangeHandler (Send + Sync).
// All mutable state is only accessed on the main thread via DispatchQueue.main.async.
final class BarViewModel: StateChangeHandler, @unchecked Sendable {
    private var windows: [UInt32: (BarWindow, BarContentView)] = [:]
    private var nodes: [UInt32: [BarNode]] = [:]

    func onStateChange(event: StateChangeEvent) throws {
        DispatchQueue.main.async { [self] in
            switch event {
            case let .nodeAdded(display, node):
                nodes[display, default: []].append(node)
                refreshDisplay(display)

            case let .nodeRemoved(display, _):
                let updated = getNodesForDisplay(display: display)
                nodes[display] = updated
                refreshDisplay(display)

            case let .nodeUpdated(display, node):
                if let idx = nodes[display]?.firstIndex(where: { $0.name == node.name }) {
                    nodes[display]?[idx] = node
                }
                refreshDisplay(display)

            case let .nodeMoved(oldDisplay, newDisplay, node):
                nodes[oldDisplay]?.removeAll { $0.name == node.name }
                refreshDisplay(oldDisplay)
                nodes[newDisplay, default: []].append(node)
                refreshDisplay(newDisplay)

            case let .fullRefresh(display, newNodes):
                nodes[display] = newNodes
                refreshDisplay(display)
            }
        }
    }

    @MainActor
    private func refreshDisplay(_ displayID: UInt32) {
        let displayNodes = nodes[displayID] ?? []

        if displayNodes.isEmpty {
            if let (window, _) = windows.removeValue(forKey: displayID) {
                window.orderOut(nil)
            }
            return
        }

        guard let (window, contentView) = ensureWindow(for: displayID) else {
            return
        }
        contentView.updateNodes(displayNodes)
        let size = contentView.intrinsicContentSize
        window.updateFrame(contentSize: size, animate: true)
    }

    @MainActor
    private func ensureWindow(for displayID: UInt32) -> (BarWindow, BarContentView)? {
        if let existing = windows[displayID] {
            return existing
        }

        guard let screen = NSScreen.screens.first(where: { $0.displayID == displayID }) else {
            return nil
        }

        return createWindow(for: screen, displayID: displayID)
    }

    @MainActor
    private func createWindow(for screen: NSScreen, displayID: UInt32) -> (BarWindow, BarContentView) {
        let contentView = BarContentView()
        let window = BarWindow(screen: screen)
        window.contentView = contentView
        window.orderFrontRegardless()
        windows[displayID] = (window, contentView)
        return (window, contentView)
    }
}

import AppKit

// @unchecked Sendable: required by UniFFI's StateChangeHandler (Send + Sync).
// All mutable state is only accessed on the main thread via DispatchQueue.main.async.
final class BarViewModel: StateChangeHandler, @unchecked Sendable {
    private var windows: [UInt32: [BarWindow.Alignment: (BarWindow, BarContentView)]] = [:]
    private var nodes: [UInt32: [BarNode]] = [:]
    private var pendingDisplays: Set<UInt32> = []
    private var refreshTimer: Timer?

    func onStateChange(event: StateChangeEvent) throws {
        DispatchQueue.main.async { [self] in
            switch event {
            case let .nodeAdded(display, node):
                nodes[display, default: []].append(node)
                scheduleRefresh(display)

            case let .nodeRemoved(display, _):
                let updated = getNodesForDisplay(display: display)
                nodes[display] = updated
                scheduleRefresh(display)

            case let .nodeUpdated(display, node):
                if let idx = nodes[display]?.firstIndex(where: { $0.name == node.name }) {
                    nodes[display]?[idx] = node
                }
                scheduleRefresh(display)

            case let .nodeMoved(oldDisplay, newDisplay, node):
                nodes[oldDisplay]?.removeAll { $0.name == node.name }
                scheduleRefresh(oldDisplay)
                nodes[newDisplay, default: []].append(node)
                scheduleRefresh(newDisplay)

            case let .fullRefresh(display, newNodes):
                nodes[display] = newNodes
                scheduleRefresh(display)
            }
        }
    }

    @MainActor
    private func scheduleRefresh(_ displayID: UInt32) {
        pendingDisplays.insert(displayID)
        refreshTimer?.invalidate()
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 0.016, repeats: false) { _ in
            DispatchQueue.main.async { [self] in
                let displays = pendingDisplays
                pendingDisplays.removeAll()
                for id in displays {
                    refreshDisplay(id)
                }
            }
        }
    }

    @MainActor
    private func refreshDisplay(_ displayID: UInt32) {
        let displayNodes = nodes[displayID] ?? []

        if displayNodes.isEmpty {
            removeAllWindows(for: displayID)
            return
        }

        let screen = NSScreen.screens.first { $0.displayID == displayID }
        let hasNotch = screen?.auxiliaryTopLeftArea != nil

        if hasNotch {
            refreshWithNotch(displayID: displayID, displayNodes: displayNodes)
        } else {
            refreshCentered(displayID: displayID, displayNodes: displayNodes)
        }
    }

    @MainActor
    private func refreshCentered(displayID: UInt32, displayNodes: [BarNode]) {
        // Remove any left/right windows from a previous notch state
        for alignment in [BarWindow.Alignment.left, .right] {
            if let (window, _) = windows[displayID]?.removeValue(forKey: alignment) {
                window.orderOut(nil)
            }
        }

        guard let (window, contentView) = ensureWindow(for: displayID, alignment: .center) else {
            return
        }
        contentView.updateNodes(displayNodes)
        let size = contentView.intrinsicContentSize
        window.updateFrame(contentSize: size, alignment: .center, animate: true)
    }

    @MainActor
    private func refreshWithNotch(displayID: UInt32, displayNodes: [BarNode]) {
        // Remove center window if it exists
        if let (window, _) = windows[displayID]?.removeValue(forKey: .center) {
            window.orderOut(nil)
        }

        let tree = resolveTree(displayNodes)

        // Group tree entries by notch alignment
        var grouped: [BarWindow.Alignment: [BarNode]] = [:]
        for entry in tree {
            let alignment = notchAlignmentForEntry(entry)
            grouped[alignment, default: []].append(contentsOf: collectNodes(from: entry))
        }

        for alignment in [BarWindow.Alignment.left, .right] {
            if let alignNodes = grouped[alignment], !alignNodes.isEmpty {
                guard let (window, contentView) = ensureWindow(for: displayID, alignment: alignment) else {
                    continue
                }
                contentView.updateNodes(alignNodes)
                let size = contentView.intrinsicContentSize
                window.updateFrame(contentSize: size, alignment: alignment, animate: true)
            } else {
                if let (window, _) = windows[displayID]?.removeValue(forKey: alignment) {
                    window.orderOut(nil)
                }
            }
        }

        if windows[displayID]?.isEmpty == true {
            windows.removeValue(forKey: displayID)
        }
    }

    @MainActor
    private func removeAllWindows(for displayID: UInt32) {
        if let alignWindows = windows.removeValue(forKey: displayID) {
            for (_, (window, _)) in alignWindows {
                window.orderOut(nil)
            }
        }
    }

    @MainActor
    func handleDisplayChange(activeDisplayIDs: Set<UInt32>) {
        for (displayID, alignWindows) in windows {
            if !activeDisplayIDs.contains(displayID) {
                for (_, (window, _)) in alignWindows {
                    window.orderOut(nil)
                }
                windows.removeValue(forKey: displayID)
            }
        }

        for displayID in activeDisplayIDs {
            if windows[displayID] == nil, let displayNodes = nodes[displayID], !displayNodes.isEmpty {
                refreshDisplay(displayID)
            }
        }
    }

    @MainActor
    private func ensureWindow(for displayID: UInt32, alignment: BarWindow.Alignment) -> (BarWindow, BarContentView)? {
        if let existing = windows[displayID]?[alignment] {
            return existing
        }

        guard let screen = NSScreen.screens.first(where: { $0.displayID == displayID }) else {
            return nil
        }

        return createWindow(for: screen, displayID: displayID, alignment: alignment)
    }

    @MainActor
    private func createWindow(for screen: NSScreen, displayID: UInt32, alignment: BarWindow.Alignment) -> (BarWindow, BarContentView) {
        let contentView = BarContentView()
        let window = BarWindow(screen: screen)
        window.contentView = contentView
        window.orderFrontRegardless()
        windows[displayID, default: [:]][alignment] = (window, contentView)
        return (window, contentView)
    }
}

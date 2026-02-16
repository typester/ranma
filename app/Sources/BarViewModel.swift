import AppKit

// @unchecked Sendable: required by UniFFI's StateChangeHandler (Send + Sync).
// All mutable state is only accessed on the main thread via DispatchQueue.main.async.
final class BarViewModel: StateChangeHandler, @unchecked Sendable {
    private var windows: [UInt32: (BarWindow, BarContentView)] = [:]
    private var items: [UInt32: [BarItem]] = [:]

    func onStateChange(event: StateChangeEvent) throws {
        DispatchQueue.main.async { [self] in
            switch event {
            case let .itemAdded(display, item):
                items[display, default: []].append(item)
                refreshDisplay(display)

            case let .itemRemoved(display, _):
                let updated = getItemsForDisplay(display: display)
                items[display] = updated
                refreshDisplay(display)

            case let .itemUpdated(display, item):
                if let idx = items[display]?.firstIndex(where: { $0.name == item.name }) {
                    items[display]?[idx] = item
                }
                refreshDisplay(display)

            case let .itemMoved(oldDisplay, newDisplay, item):
                items[oldDisplay]?.removeAll { $0.name == item.name }
                refreshDisplay(oldDisplay)
                items[newDisplay, default: []].append(item)
                refreshDisplay(newDisplay)

            case let .fullRefresh(display, newItems):
                items[display] = newItems
                refreshDisplay(display)
            }
        }
    }

    @MainActor
    private func refreshDisplay(_ displayID: UInt32) {
        let displayItems = items[displayID] ?? []

        if displayItems.isEmpty {
            if let (window, _) = windows.removeValue(forKey: displayID) {
                window.orderOut(nil)
            }
            return
        }

        guard let (window, contentView) = ensureWindow(for: displayID) else {
            return
        }
        contentView.updateItems(displayItems)
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

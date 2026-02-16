import AppKit

// @unchecked Sendable: required by UniFFI's StateChangeHandler (Send + Sync).
// All mutable state is only accessed on the main thread via DispatchQueue.main.async.
final class BarViewModel: StateChangeHandler, @unchecked Sendable {
    private let window: BarWindow
    private let contentView: BarContentView
    private var items: [BarItem] = []

    @MainActor
    init(window: BarWindow, contentView: BarContentView) {
        self.window = window
        self.contentView = contentView
    }

    func onStateChange(event: StateChangeEvent) throws {
        DispatchQueue.main.async { [self] in
            switch event {
            case let .itemAdded(item):
                items.append(item)
            case let .itemRemoved(name):
                items.removeAll { $0.name == name }
            case let .itemUpdated(item):
                if let index = items.firstIndex(where: { $0.name == item.name }) {
                    items[index] = item
                }
            case let .fullRefresh(newItems):
                items = newItems
            }
            refreshUI()
        }
    }

    @MainActor
    private func refreshUI() {
        contentView.updateItems(items)
        let size = contentView.intrinsicContentSize
        window.updateFrame(contentSize: size, animate: true)
    }
}

import AppKit

enum WindowSizer {
    static func calculateWidth(for items: [BarItem], padding: CGFloat) -> CGFloat {
        var totalWidth: CGFloat = padding * 2
        for (index, item) in items.enumerated() {
            if index > 0 {
                totalWidth += 8
            }
            totalWidth += measureItem(item)
        }
        return max(totalWidth, 40)
    }

    private static func measureItem(_ item: BarItem) -> CGFloat {
        var width: CGFloat = 0
        let font = NSFont.systemFont(ofSize: 12, weight: .medium)

        if item.icon != nil {
            width += 16
        }

        if let label = item.label {
            let attrs: [NSAttributedString.Key: Any] = [.font: font]
            let size = (label as NSString).size(withAttributes: attrs)
            if width > 0 { width += 4 }
            width += size.width
        }

        return max(width, 8)
    }
}

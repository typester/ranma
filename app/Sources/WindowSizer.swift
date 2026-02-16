import AppKit

enum WindowSizer {
    private static let defaultBarHeight: CGFloat = 24
    private static let itemSpacing: CGFloat = 8
    private static let containerSpacing: CGFloat = 8
    private static let iconLabelGap: CGFloat = 4
    private static let defaultFontSize: CGFloat = 13
    private static let defaultIconSize: CGFloat = 16

    static func calculateWidth(for nodes: [BarNode]) -> CGFloat {
        let tree = resolveTree(nodes)
        if tree.isEmpty { return 0 }

        var totalWidth: CGFloat = 0
        for (index, topLevel) in tree.enumerated() {
            if index > 0 {
                totalWidth += containerSpacing
            }
            totalWidth += measureTopLevel(topLevel)
        }
        return max(totalWidth, 40)
    }

    private static func measureTopLevel(_ entry: TreeEntry) -> CGFloat {
        switch entry {
        case .container(let node, let children):
            let pl = CGFloat(node.style.paddingLeft ?? 0)
            let pr = CGFloat(node.style.paddingRight ?? 0)
            let gap = CGFloat(node.style.gap ?? 8)
            var inner: CGFloat = 0
            for (index, child) in children.enumerated() {
                if index > 0 { inner += gap }
                inner += measureItem(child)
            }
            return pl + inner + pr
        case .item(let node):
            return measureItem(node)
        }
    }

    private static func measureItem(_ node: BarNode) -> CGFloat {
        if let w = node.style.width { return CGFloat(w) }

        var width: CGFloat = 0
        let font = fontForNode(node)

        if let iconName = node.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil) {
            let config = NSImage.SymbolConfiguration(pointSize: iconSizeForNode(node), weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image
            width += configured.size.width
        }

        if let label = node.label {
            let attrs: [NSAttributedString.Key: Any] = [.font: font]
            let size = (label as NSString).size(withAttributes: attrs)
            if width > 0 { width += iconLabelGap }
            width += size.width
        }

        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)
        return pl + width + pr
    }

    private static func fontForNode(_ node: BarNode) -> NSFont {
        let size = CGFloat(node.fontSize ?? Float(defaultFontSize))
        if let family = node.fontFamily, let font = NSFont(name: family, size: size) {
            return font
        }
        let weight = fontWeight(from: node.fontWeight)
        return NSFont.systemFont(ofSize: size, weight: weight)
    }

    private static func iconSizeForNode(_ node: BarNode) -> CGFloat {
        CGFloat(node.fontSize ?? Float(defaultIconSize))
    }

    private static func fontWeight(from name: String?) -> NSFont.Weight {
        switch name {
        case "ultralight": return .ultraLight
        case "thin": return .thin
        case "light": return .light
        case "regular": return .regular
        case "medium": return .medium
        case "semibold": return .semibold
        case "bold": return .bold
        case "heavy": return .heavy
        case "black": return .black
        default: return .regular
        }
    }
}

enum TreeEntry {
    case container(BarNode, [BarNode])
    case item(BarNode)
}

func notchAlignmentForEntry(_ entry: TreeEntry) -> BarWindow.Alignment {
    let alignStr: String?
    switch entry {
    case .container(let node, _):
        alignStr = node.style.notchAlign
    case .item(let node):
        alignStr = node.style.notchAlign
    }
    switch alignStr {
    case "left": return .left
    case "right": return .right
    default: return .right
    }
}

func resolveTree(_ nodes: [BarNode]) -> [TreeEntry] {
    let sorted = nodes.sorted { $0.position < $1.position }
    let topLevel = sorted.filter { $0.parent == nil }

    return topLevel.map { node in
        switch node.nodeType {
        case .container:
            let children = sorted
                .filter { $0.parent == node.name }
                .sorted { $0.position < $1.position }
            return .container(node, children)
        case .item:
            return .item(node)
        }
    }
}

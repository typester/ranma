import AppKit

enum WindowSizer {
    private static let defaultBarHeight: CGFloat = 28
    private static let itemSpacing: CGFloat = 8
    private static let containerSpacing: CGFloat = 8
    private static let iconLabelGap: CGFloat = 4
    private static let defaultFontSize: CGFloat = 12
    private static let defaultIconSize: CGFloat = 14

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
            var inner: CGFloat = 0
            for (index, child) in children.enumerated() {
                if index > 0 { inner += itemSpacing }
                inner += measureItem(child)
            }
            return pl + inner + pr
        case .item(let node):
            return measureItem(node)
        }
    }

    private static func measureItem(_ node: BarNode) -> CGFloat {
        var width: CGFloat = 0
        let font = fontForNode(node)

        if node.icon != nil {
            width += iconSizeForNode(node) + 2
        }

        if let label = node.label {
            let attrs: [NSAttributedString.Key: Any] = [.font: font]
            let size = (label as NSString).size(withAttributes: attrs)
            if width > 0 { width += iconLabelGap }
            width += size.width
        }

        return max(width, 8)
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
        default: return .medium
        }
    }
}

enum TreeEntry {
    case container(BarNode, [BarNode])
    case item(BarNode)
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

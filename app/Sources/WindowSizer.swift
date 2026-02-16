import AppKit

indirect enum TreeEntry {
    case row(BarNode, [TreeEntry])
    case column(BarNode, [TreeEntry])
    case box(BarNode, [TreeEntry])
    case item(BarNode)
}

enum WindowSizer {
    private static let containerSpacing: CGFloat = 8
    private static let iconLabelGap: CGFloat = 4
    private static let defaultFontSize: CGFloat = 13
    private static let defaultIconSize: CGFloat = 16

    static func calculateSize(for nodes: [BarNode]) -> CGSize {
        let tree = resolveTree(nodes)
        if tree.isEmpty { return .zero }

        var totalWidth: CGFloat = 0
        var maxHeight: CGFloat = 0
        for (index, entry) in tree.enumerated() {
            if index > 0 { totalWidth += containerSpacing }
            let size = measureEntry(entry)
            totalWidth += size.width
            maxHeight = max(maxHeight, size.height)
        }
        return CGSize(width: max(totalWidth, 40), height: max(maxHeight, 24))
    }

    static func measureEntry(_ entry: TreeEntry) -> CGSize {
        switch entry {
        case .row(let node, let children):
            return measureLayout(node: node, children: children, axis: .horizontal)
        case .column(let node, let children):
            return measureLayout(node: node, children: children, axis: .vertical)
        case .box(let node, let children):
            return measureLayout(node: node, children: children, axis: .stacked)
        case .item(let node):
            return measureItem(node)
        }
    }

    private enum Axis {
        case horizontal, vertical, stacked
    }

    private static func measureLayout(node: BarNode, children: [TreeEntry], axis: Axis) -> CGSize {
        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)
        let pt = CGFloat(node.style.paddingTop ?? 0)
        let pb = CGFloat(node.style.paddingBottom ?? 0)
        let gap = CGFloat(node.style.gap ?? 0)
        let ml = CGFloat(node.style.marginLeft ?? 0)
        let mr = CGFloat(node.style.marginRight ?? 0)
        let mt = CGFloat(node.style.marginTop ?? 0)
        let mb = CGFloat(node.style.marginBottom ?? 0)

        var innerWidth: CGFloat = 0
        var innerHeight: CGFloat = 0

        for (index, child) in children.enumerated() {
            let childSize = measureEntry(child)
            switch axis {
            case .horizontal:
                if index > 0 { innerWidth += gap }
                innerWidth += childSize.width
                innerHeight = max(innerHeight, childSize.height)
            case .vertical:
                if index > 0 { innerHeight += gap }
                innerWidth = max(innerWidth, childSize.width)
                innerHeight += childSize.height
            case .stacked:
                innerWidth = max(innerWidth, childSize.width)
                innerHeight = max(innerHeight, childSize.height)
            }
        }

        let contentWidth = pl + innerWidth + pr
        let contentHeight = pt + innerHeight + pb

        let w = node.style.width.map { CGFloat($0) } ?? contentWidth
        let h = node.style.height.map { CGFloat($0) } ?? contentHeight

        return CGSize(width: ml + w + mr, height: mt + h + mb)
    }

    private static func measureItem(_ node: BarNode) -> CGSize {
        let ml = CGFloat(node.style.marginLeft ?? 0)
        let mr = CGFloat(node.style.marginRight ?? 0)
        let mt = CGFloat(node.style.marginTop ?? 0)
        let mb = CGFloat(node.style.marginBottom ?? 0)

        if let w = node.style.width, let h = node.style.height {
            return CGSize(width: ml + CGFloat(w) + mr, height: mt + CGFloat(h) + mb)
        }

        var contentWidth: CGFloat = 0
        let font = fontForNode(node)

        if let iconName = node.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil) {
            let config = NSImage.SymbolConfiguration(pointSize: iconSizeForNode(node), weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image
            contentWidth += configured.size.width
        }

        if let label = node.label {
            let attrs: [NSAttributedString.Key: Any] = [.font: font]
            let size = (label as NSString).size(withAttributes: attrs)
            if contentWidth > 0 { contentWidth += iconLabelGap }
            contentWidth += size.width
        }

        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)
        let w = node.style.width.map { CGFloat($0) } ?? (pl + contentWidth + pr)
        let h = node.style.height.map { CGFloat($0) } ?? font.pointSize + 4

        return CGSize(width: ml + w + mr, height: mt + h + mb)
    }

    static func fontForNode(_ node: BarNode) -> NSFont {
        let size = CGFloat(node.fontSize ?? Float(defaultFontSize))
        if let family = node.fontFamily, let font = NSFont(name: family, size: size) {
            return font
        }
        let weight = fontWeight(from: node.fontWeight)
        return NSFont.systemFont(ofSize: size, weight: weight)
    }

    static func iconSizeForNode(_ node: BarNode) -> CGFloat {
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

func nodeForEntry(_ entry: TreeEntry) -> BarNode {
    switch entry {
    case .row(let node, _), .column(let node, _), .box(let node, _), .item(let node):
        return node
    }
}

func notchAlignmentForEntry(_ entry: TreeEntry) -> BarWindow.Alignment {
    let alignStr = nodeForEntry(entry).style.notchAlign
    switch alignStr {
    case "left": return .left
    case "right": return .right
    default: return .right
    }
}

func collectNodes(from entry: TreeEntry) -> [BarNode] {
    switch entry {
    case .row(let node, let children), .column(let node, let children), .box(let node, let children):
        var result = [node]
        for child in children {
            result.append(contentsOf: collectNodes(from: child))
        }
        return result
    case .item(let node):
        return [node]
    }
}

func resolveTree(_ nodes: [BarNode]) -> [TreeEntry] {
    let sorted = nodes.sorted { $0.position < $1.position }
    let topLevel = sorted.filter { $0.parent == nil }

    func buildEntry(_ node: BarNode) -> TreeEntry {
        switch node.nodeType {
        case .row:
            let children = sorted
                .filter { $0.parent == node.name }
                .map { buildEntry($0) }
            return .row(node, children)
        case .column:
            let children = sorted
                .filter { $0.parent == node.name }
                .map { buildEntry($0) }
            return .column(node, children)
        case .box:
            let children = sorted
                .filter { $0.parent == node.name }
                .map { buildEntry($0) }
            return .box(node, children)
        case .item:
            return .item(node)
        }
    }

    return topLevel.map { buildEntry($0) }
}

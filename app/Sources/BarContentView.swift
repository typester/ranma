import AppKit

class BarContentView: NSView {
    private let defaultBarHeight: CGFloat = 24
    private let itemSpacing: CGFloat = 8
    private let containerSpacing: CGFloat = 8
    private let iconLabelGap: CGFloat = 4
    private let defaultFontSize: CGFloat = 13
    private let defaultIconSize: CGFloat = 16

    private var nodes: [BarNode] = []

    override var intrinsicContentSize: NSSize {
        let totalWidth = WindowSizer.calculateWidth(for: nodes)
        let height = calculateBarHeight()
        return NSSize(width: totalWidth, height: height)
    }

    func updateNodes(_ newNodes: [BarNode]) {
        nodes = newNodes
        invalidateIntrinsicContentSize()
        needsDisplay = true
    }

    private func calculateBarHeight() -> CGFloat {
        let tree = resolveTree(nodes)
        var maxHeight = defaultBarHeight
        for entry in tree {
            if case .container(let node, _) = entry, let h = node.style.height {
                maxHeight = max(maxHeight, CGFloat(h))
            }
        }
        return maxHeight
    }

    override func draw(_ dirtyRect: NSRect) {
        let tree = resolveTree(nodes)
        var x: CGFloat = 0

        for (index, entry) in tree.enumerated() {
            if index > 0 { x += containerSpacing }

            switch entry {
            case .container(let container, let children):
                x += drawContainer(container, children: children, at: x)
            case .item(let node):
                x += drawItem(node, at: x, containerHeight: bounds.height)
            }
        }
    }

    private func drawContainer(_ container: BarNode, children: [BarNode], at x: CGFloat) -> CGFloat {
        let pl = CGFloat(container.style.paddingLeft ?? 0)
        let pr = CGFloat(container.style.paddingRight ?? 0)
        let gap = CGFloat(container.style.gap ?? 8)

        var innerWidth: CGFloat = 0
        for (index, child) in children.enumerated() {
            if index > 0 { innerWidth += gap }
            innerWidth += measureItemWidth(child)
        }

        let totalWidth = pl + innerWidth + pr
        let containerHeight = CGFloat(container.style.height ?? Float(defaultBarHeight))
        let containerY = (bounds.height - containerHeight) / 2
        let containerRect = NSRect(x: x, y: containerY, width: totalWidth, height: containerHeight)

        let cr = CGFloat(container.style.cornerRadius ?? 0)
        let path = NSBezierPath(roundedRect: containerRect, xRadius: cr, yRadius: cr)

        let gfxContext = NSGraphicsContext.current
        if let shadowHex = container.style.shadowColor, let shadowColor = NSColor.fromHex(shadowHex) {
            gfxContext?.saveGraphicsState()
            let shadow = NSShadow()
            shadow.shadowColor = shadowColor
            shadow.shadowBlurRadius = CGFloat(container.style.shadowRadius ?? 4)
            shadow.shadowOffset = NSSize(width: 0, height: -1)
            shadow.set()
        }

        if let bgHex = container.style.backgroundColor, let bgColor = NSColor.fromHex(bgHex) {
            bgColor.setFill()
            path.fill()
        }

        if container.style.shadowColor != nil {
            gfxContext?.restoreGraphicsState()
        }

        let bw = CGFloat(container.style.borderWidth ?? 0)
        if bw > 0, let borderHex = container.style.borderColor, let borderColor = NSColor.fromHex(borderHex) {
            borderColor.setStroke()
            path.lineWidth = bw
            path.stroke()
        }

        var cx = x + pl
        for (index, child) in children.enumerated() {
            if index > 0 { cx += gap }
            cx += drawItem(child, at: cx, containerHeight: containerHeight, containerY: containerY)
        }

        return totalWidth
    }

    private func drawItem(_ node: BarNode, at x: CGFloat, containerHeight: CGFloat, containerY: CGFloat = 0) -> CGFloat {
        let centerY = containerY + containerHeight / 2
        let font = fontForNode(node)
        let iconSize = iconSizeForNode(node)

        let hasBackground = node.style.backgroundColor != nil
            || node.style.borderColor != nil
            || node.style.shadowColor != nil
        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)

        let contentWidth = measureContentWidth(node)
        let totalWidth = node.style.width.map { CGFloat($0) } ?? (pl + contentWidth + pr)

        // Draw background/border/shadow if styled
        if hasBackground {
            let bgHeight = node.style.height.map { CGFloat($0) } ?? containerHeight
            let bgY = containerY + (containerHeight - bgHeight) / 2
            let bgRect = NSRect(x: x, y: bgY, width: totalWidth, height: bgHeight)
            let cr = CGFloat(node.style.cornerRadius ?? 0)
            let path = NSBezierPath(roundedRect: bgRect, xRadius: cr, yRadius: cr)

            let gfxContext = NSGraphicsContext.current
            if let shadowHex = node.style.shadowColor, let shadowColor = NSColor.fromHex(shadowHex) {
                gfxContext?.saveGraphicsState()
                let shadow = NSShadow()
                shadow.shadowColor = shadowColor
                shadow.shadowBlurRadius = CGFloat(node.style.shadowRadius ?? 4)
                shadow.shadowOffset = NSSize(width: 0, height: -1)
                shadow.set()
            }

            if let bgHex = node.style.backgroundColor, let bgColor = NSColor.fromHex(bgHex) {
                bgColor.setFill()
                path.fill()
            }

            if node.style.shadowColor != nil {
                gfxContext?.restoreGraphicsState()
            }

            let bw = CGFloat(node.style.borderWidth ?? 0)
            if bw > 0, let borderHex = node.style.borderColor, let borderColor = NSColor.fromHex(borderHex) {
                borderColor.setStroke()
                path.lineWidth = bw
                path.stroke()
            }
        }

        // Draw content — center within totalWidth
        let contentOffset = (totalWidth - pl - pr - contentWidth) / 2
        var currentX = x + pl + contentOffset

        if let iconName = node.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil)
        {
            let config = NSImage.SymbolConfiguration(pointSize: iconSize, weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image

            let tintColor = node.iconColor.flatMap { NSColor.fromHex($0) } ?? .white
            let tinted = configured.tinted(with: tintColor)

            let imageSize = tinted.size
            let imageRect = NSRect(
                x: currentX,
                y: centerY - imageSize.height / 2,
                width: imageSize.width,
                height: imageSize.height
            )
            tinted.draw(in: imageRect)
            currentX += imageSize.width

            if node.label != nil {
                currentX += iconLabelGap
            }
        }

        if let label = node.label {
            let labelColor = node.labelColor.flatMap { NSColor.fromHex($0) } ?? .white
            let attrs: [NSAttributedString.Key: Any] = [
                .foregroundColor: labelColor,
                .font: font,
            ]
            let textSize = (label as NSString).size(withAttributes: attrs)
            let textRect = NSRect(
                x: currentX,
                y: centerY - textSize.height / 2,
                width: textSize.width,
                height: textSize.height
            )
            (label as NSString).draw(in: textRect, withAttributes: attrs)
        }

        return totalWidth
    }

    private func measureContentWidth(_ node: BarNode) -> CGFloat {
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

        return width
    }

    private func measureItemWidth(_ node: BarNode) -> CGFloat {
        if let w = node.style.width { return CGFloat(w) }
        let contentWidth = measureContentWidth(node)
        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)
        return pl + contentWidth + pr
    }

    private func fontForNode(_ node: BarNode) -> NSFont {
        let size = CGFloat(node.fontSize ?? Float(defaultFontSize))
        if let family = node.fontFamily, let font = NSFont(name: family, size: size) {
            return font
        }
        let weight = fontWeight(from: node.fontWeight)
        return NSFont.systemFont(ofSize: size, weight: weight)
    }

    private func iconSizeForNode(_ node: BarNode) -> CGFloat {
        CGFloat(node.fontSize ?? Float(defaultIconSize))
    }

    private func fontWeight(from name: String?) -> NSFont.Weight {
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

extension NSColor {
    static func fromHex(_ hex: String) -> NSColor? {
        var hexStr = hex.trimmingCharacters(in: .whitespacesAndNewlines)
        if hexStr.hasPrefix("#") { hexStr.removeFirst() }

        guard let value = UInt64(hexStr, radix: 16) else { return nil }

        switch hexStr.count {
        case 6:
            let r = CGFloat((value >> 16) & 0xFF) / 255.0
            let g = CGFloat((value >> 8) & 0xFF) / 255.0
            let b = CGFloat(value & 0xFF) / 255.0
            return NSColor(red: r, green: g, blue: b, alpha: 1.0)
        case 8:
            let r = CGFloat((value >> 24) & 0xFF) / 255.0
            let g = CGFloat((value >> 16) & 0xFF) / 255.0
            let b = CGFloat((value >> 8) & 0xFF) / 255.0
            let a = CGFloat(value & 0xFF) / 255.0
            return NSColor(red: r, green: g, blue: b, alpha: a)
        default:
            return nil
        }
    }
}

extension NSImage {
    func tinted(with color: NSColor) -> NSImage {
        let image = self.copy() as! NSImage
        image.lockFocus()
        color.set()
        let rect = NSRect(origin: .zero, size: image.size)
        rect.fill(using: .sourceAtop)
        image.unlockFocus()
        image.isTemplate = false
        return image
    }
}

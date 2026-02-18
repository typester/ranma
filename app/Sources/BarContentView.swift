import AppKit

class BarContentView: NSView {
    private let containerSpacing: CGFloat = 8
    private let iconLabelGap: CGFloat = 4

    private var nodes: [BarNode] = []
    private var containerRects: [(NSRect, BarNode)] = []
    private var hoveredContainer: String? = nil

    private struct DrawContext {
        var hoverLabelColor: NSColor?
        var hoverIconColor: NSColor?
    }

    override var intrinsicContentSize: NSSize {
        let size = WindowSizer.calculateSize(for: nodes)
        return NSSize(width: size.width, height: size.height)
    }

    func updateNodes(_ newNodes: [BarNode]) {
        nodes = newNodes
        invalidateIntrinsicContentSize()
        needsDisplay = true
    }

    // MARK: - Mouse tracking

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        for area in trackingAreas { removeTrackingArea(area) }
        let area = NSTrackingArea(
            rect: bounds,
            options: [.mouseMoved, .mouseEnteredAndExited, .activeAlways],
            owner: self, userInfo: nil
        )
        addTrackingArea(area)
    }

    override func mouseMoved(with event: NSEvent) {
        let pt = convert(event.locationInWindow, from: nil)
        let hit = containerRects.last(where: { $0.0.contains(pt) })?.1.name
        if hoveredContainer != hit {
            hoveredContainer = hit
            needsDisplay = true
        }
    }

    override func mouseExited(with event: NSEvent) {
        if hoveredContainer != nil {
            hoveredContainer = nil
            needsDisplay = true
        }
    }

    override func mouseDown(with event: NSEvent) {
        let pt = convert(event.locationInWindow, from: nil)
        for (rect, node) in containerRects.reversed() {
            if rect.contains(pt), let cmd = node.onClick {
                DispatchQueue.global(qos: .utility).async {
                    let proc = Process()
                    proc.executableURL = URL(fileURLWithPath: "/bin/sh")
                    proc.arguments = ["-c", cmd]
                    try? proc.run()
                }
                return
            }
        }
    }

    override func draw(_ dirtyRect: NSRect) {
        containerRects.removeAll()
        let tree = resolveTree(nodes)
        let ctx = DrawContext(hoverLabelColor: nil, hoverIconColor: nil)
        var x: CGFloat = 0

        for (index, entry) in tree.enumerated() {
            if index > 0 { x += containerSpacing }
            let size = drawEntry(entry, at: NSPoint(x: x, y: 0), availableHeight: bounds.height, context: ctx)
            x += size.width
        }
    }

    // MARK: - Recursive drawing

    private func drawEntry(_ entry: TreeEntry, at origin: NSPoint, availableHeight: CGFloat, context: DrawContext) -> CGSize {
        switch entry {
        case .row(let node, let children):
            return drawLayout(node: node, children: children, axis: .horizontal, at: origin, availableHeight: availableHeight, context: context)
        case .column(let node, let children):
            return drawLayout(node: node, children: children, axis: .vertical, at: origin, availableHeight: availableHeight, context: context)
        case .box(let node, let children):
            return drawLayout(node: node, children: children, axis: .stacked, at: origin, availableHeight: availableHeight, context: context)
        case .item(let node):
            return drawItem(node, at: origin, availableHeight: availableHeight, context: context)
        }
    }

    private enum Axis {
        case horizontal, vertical, stacked
    }

    private func drawLayout(node: BarNode, children: [TreeEntry], axis: Axis, at origin: NSPoint, availableHeight: CGFloat, context: DrawContext) -> CGSize {
        let totalSize = WindowSizer.measureEntry(axis == .horizontal ? .row(node, children) :
                                                  axis == .vertical ? .column(node, children) :
                                                  .box(node, children))

        let ml = CGFloat(node.style.marginLeft ?? 0)
        let mr = CGFloat(node.style.marginRight ?? 0)
        let mt = CGFloat(node.style.marginTop ?? 0)
        let mb = CGFloat(node.style.marginBottom ?? 0)

        let contentWidth = totalSize.width - ml - mr
        let contentHeight = totalSize.height - mt - mb

        // Center vertically within available height
        let contentY = origin.y + (availableHeight - totalSize.height) / 2 + mb
        let contentX = origin.x + ml
        let contentRect = NSRect(x: contentX, y: contentY, width: contentWidth, height: contentHeight)

        // Record container rect for hit testing
        containerRects.append((contentRect, node))

        // Determine hover state
        let isHovered = hoveredContainer == node.name
        var childContext = context
        if isHovered {
            if let c = node.style.hoverLabelColor.flatMap({ NSColor.fromHex($0) }) { childContext.hoverLabelColor = c }
            if let c = node.style.hoverIconColor.flatMap({ NSColor.fromHex($0) }) { childContext.hoverIconColor = c }
        }

        // Draw background/border/shadow
        drawDecoration(node: node, in: contentRect, hovered: isHovered)

        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)
        let pt = CGFloat(node.style.paddingTop ?? 0)
        let pb = CGFloat(node.style.paddingBottom ?? 0)
        let gap = CGFloat(node.style.gap ?? 0)
        let alignItems = node.style.alignItems ?? "start"
        let justifyContent = node.style.justifyContent ?? "start"

        switch axis {
        case .horizontal:
            let innerWidth = contentWidth - pl - pr
            let innerHeight = contentHeight - pt - pb
            // Calculate total children width for justify_content
            var totalChildWidth: CGFloat = 0
            for (index, child) in children.enumerated() {
                if index > 0 { totalChildWidth += gap }
                totalChildWidth += WindowSizer.measureEntry(child).width
            }
            var cx: CGFloat
            switch justifyContent {
            case "center": cx = contentX + pl + (innerWidth - totalChildWidth) / 2
            case "end":    cx = contentX + pl + innerWidth - totalChildWidth
            default:       cx = contentX + pl
            }
            for (index, child) in children.enumerated() {
                if index > 0 { cx += gap }
                let childSize = WindowSizer.measureEntry(child)
                let childY: CGFloat
                switch alignItems {
                case "center": childY = contentY + pb + (innerHeight - childSize.height) / 2
                case "end":    childY = contentY + pb
                default:       childY = contentY + pb + innerHeight - childSize.height // start = top
                }
                let drawn = drawEntry(child, at: NSPoint(x: cx, y: childY), availableHeight: childSize.height, context: childContext)
                cx += drawn.width
            }
        case .vertical:
            let innerWidth = contentWidth - pl - pr
            let innerHeight = contentHeight - pt - pb
            // Calculate total children height for justify_content
            var totalChildHeight: CGFloat = 0
            for (index, child) in children.enumerated() {
                if index > 0 { totalChildHeight += gap }
                totalChildHeight += WindowSizer.measureEntry(child).height
            }
            var cy: CGFloat
            switch justifyContent {
            case "center": cy = contentY + contentHeight - pt - (innerHeight - totalChildHeight) / 2
            case "end":    cy = contentY + pb + totalChildHeight
            default:       cy = contentY + contentHeight - pt // start = top
            }
            for (index, child) in children.enumerated() {
                if index > 0 { cy -= gap }
                let childSize = WindowSizer.measureEntry(child)
                cy -= childSize.height
                let childX: CGFloat
                switch alignItems {
                case "center": childX = contentX + pl + (innerWidth - childSize.width) / 2
                case "end":    childX = contentX + pl + innerWidth - childSize.width
                default:       childX = contentX + pl
                }
                let _ = drawEntry(child, at: NSPoint(x: childX, y: cy), availableHeight: childSize.height, context: childContext)
            }
        case .stacked:
            for child in children {
                let _ = drawEntry(child, at: NSPoint(x: contentX + pl, y: contentY + pb), availableHeight: contentHeight - pt - pb, context: childContext)
            }
        }

        return totalSize
    }

    private func drawDecoration(node: BarNode, in rect: NSRect, hovered: Bool = false) {
        let effectiveBg = hovered ? (node.style.hoverBackgroundColor ?? node.style.backgroundColor) : node.style.backgroundColor
        let hasDecoration = effectiveBg != nil
            || node.style.borderColor != nil
            || node.style.shadowColor != nil

        guard hasDecoration else { return }

        let cr = CGFloat(node.style.cornerRadius ?? 0)
        let path = NSBezierPath(roundedRect: rect, xRadius: cr, yRadius: cr)

        let gfxContext = NSGraphicsContext.current
        if let shadowHex = node.style.shadowColor, let shadowColor = NSColor.fromHex(shadowHex) {
            gfxContext?.saveGraphicsState()
            let shadow = NSShadow()
            shadow.shadowColor = shadowColor
            shadow.shadowBlurRadius = CGFloat(node.style.shadowRadius ?? 4)
            shadow.shadowOffset = NSSize(width: 0, height: -1)
            shadow.set()
        }

        if let bgHex = effectiveBg, let bgColor = NSColor.fromHex(bgHex) {
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

    // MARK: - Item drawing

    private func drawItem(_ node: BarNode, at origin: NSPoint, availableHeight: CGFloat, context: DrawContext) -> CGSize {
        let totalSize = WindowSizer.measureEntry(.item(node))

        let ml = CGFloat(node.style.marginLeft ?? 0)
        let mr = CGFloat(node.style.marginRight ?? 0)
        let mt = CGFloat(node.style.marginTop ?? 0)
        let mb = CGFloat(node.style.marginBottom ?? 0)

        let itemWidth = totalSize.width - ml - mr
        let itemHeight = totalSize.height - mt - mb

        // Center vertically within available height
        let itemY = origin.y + (availableHeight - totalSize.height) / 2 + mb
        let itemX = origin.x + ml
        let itemRect = NSRect(x: itemX, y: itemY, width: itemWidth, height: itemHeight)

        // Draw background/border/shadow
        drawDecoration(node: node, in: itemRect)

        // Draw content centered within item
        let font = WindowSizer.fontForNode(node)
        let iconSize = WindowSizer.iconSizeForNode(node)
        let centerY = itemY + itemHeight / 2
        let pl = CGFloat(node.style.paddingLeft ?? 0)
        let pr = CGFloat(node.style.paddingRight ?? 0)

        let contentWidth = measureContentWidth(node)
        let contentOffset = (itemWidth - pl - pr - contentWidth) / 2
        var currentX = itemX + pl + contentOffset

        if let iconName = node.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil)
        {
            let config = NSImage.SymbolConfiguration(pointSize: iconSize, weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image

            let tintColor = context.hoverIconColor ?? node.iconColor.flatMap { NSColor.fromHex($0) } ?? .white
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

            if node.label != nil || node.image != nil {
                currentX += iconLabelGap
            }
        }

        if let imagePath = node.image,
           let img = ImageCache.shared.image(for: imagePath) {
            let scale = CGFloat(node.imageScale ?? 1.0)
            let scaledWidth = img.size.width * scale
            let scaledHeight = img.size.height * scale

            let gfxCtx = NSGraphicsContext.current
            let prevInterpolation = gfxCtx?.imageInterpolation
            gfxCtx?.imageInterpolation = .none

            if let explicitWidth = node.style.width, CGFloat(explicitWidth) > scaledWidth {
                let tileStartX = currentX - contentOffset
                let rightEdge = itemX + itemWidth - pr
                let tileAreaWidth = rightEdge - tileStartX
                var tx: CGFloat = 0
                while tx < tileAreaWidth {
                    let remaining = tileAreaWidth - tx
                    let drawWidth = min(scaledWidth, remaining)
                    let srcWidth = drawWidth / scale
                    let destRect = NSRect(
                        x: tileStartX + tx,
                        y: centerY - scaledHeight / 2,
                        width: drawWidth,
                        height: scaledHeight
                    )
                    let srcRect = NSRect(
                        x: 0, y: 0,
                        width: srcWidth,
                        height: img.size.height
                    )
                    img.draw(in: destRect, from: srcRect, operation: .sourceOver, fraction: 1.0)
                    tx += scaledWidth
                }
                currentX = rightEdge
            } else {
                let destRect = NSRect(
                    x: currentX,
                    y: centerY - scaledHeight / 2,
                    width: scaledWidth,
                    height: scaledHeight
                )
                img.draw(in: destRect, from: NSRect(origin: .zero, size: img.size), operation: .sourceOver, fraction: 1.0)
                currentX += scaledWidth
            }

            gfxCtx?.imageInterpolation = prevInterpolation ?? .default

            if node.label != nil {
                currentX += iconLabelGap
            }
        }

        if let label = node.label {
            let labelColor = context.hoverLabelColor ?? node.labelColor.flatMap { NSColor.fromHex($0) } ?? .white
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

        return totalSize
    }

    private func measureContentWidth(_ node: BarNode) -> CGFloat {
        var width: CGFloat = 0
        let font = WindowSizer.fontForNode(node)

        if let iconName = node.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil) {
            let config = NSImage.SymbolConfiguration(pointSize: WindowSizer.iconSizeForNode(node), weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image
            width += configured.size.width
        }

        if let imagePath = node.image,
           let img = ImageCache.shared.image(for: imagePath) {
            let scale = CGFloat(node.imageScale ?? 1.0)
            if width > 0 { width += iconLabelGap }
            width += img.size.width * scale
        }

        if let label = node.label {
            let attrs: [NSAttributedString.Key: Any] = [.font: font]
            let size = (label as NSString).size(withAttributes: attrs)
            if width > 0 { width += iconLabelGap }
            width += size.width
        }

        return width
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

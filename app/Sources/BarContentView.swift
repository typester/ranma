import AppKit

class BarContentView: NSView {
    private let barHeight: CGFloat = 28
    private let barPadding: CGFloat = 12
    private let itemSpacing: CGFloat = 8
    private let iconLabelGap: CGFloat = 4
    private let iconSize: CGFloat = 14
    private let cornerRadius: CGFloat = 10
    private let font = NSFont.systemFont(ofSize: 12, weight: .medium)

    private var items: [BarItem] = []

    override var intrinsicContentSize: NSSize {
        let totalWidth = WindowSizer.calculateWidth(for: items, padding: barPadding)
        return NSSize(width: totalWidth, height: barHeight)
    }

    func updateItems(_ newItems: [BarItem]) {
        items = newItems
        invalidateIntrinsicContentSize()
        needsDisplay = true
    }

    override func draw(_ dirtyRect: NSRect) {
        let path = NSBezierPath(roundedRect: bounds, xRadius: cornerRadius, yRadius: cornerRadius)
        NSColor(white: 0.15, alpha: 0.9).setFill()
        path.fill()

        var x = barPadding
        for item in items {
            x += drawItem(item, at: x)
            x += itemSpacing
        }
    }

    private func drawItem(_ item: BarItem, at x: CGFloat) -> CGFloat {
        var currentX = x
        let centerY = bounds.height / 2

        if let iconName = item.icon,
           let image = NSImage(systemSymbolName: iconName, accessibilityDescription: nil)
        {
            let config = NSImage.SymbolConfiguration(pointSize: iconSize, weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image

            let tintColor = item.iconColor.flatMap { NSColor.fromHex($0) } ?? .white
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

            if item.label != nil {
                currentX += iconLabelGap
            }
        }

        if let label = item.label {
            let attrs: [NSAttributedString.Key: Any] = [
                .foregroundColor: NSColor.white,
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
            currentX += textSize.width
        }

        return currentX - x
    }
}

extension NSColor {
    static func fromHex(_ hex: String) -> NSColor? {
        var hexStr = hex.trimmingCharacters(in: .whitespacesAndNewlines)
        if hexStr.hasPrefix("#") { hexStr.removeFirst() }

        guard hexStr.count == 6, let value = UInt64(hexStr, radix: 16) else {
            return nil
        }

        let r = CGFloat((value >> 16) & 0xFF) / 255.0
        let g = CGFloat((value >> 8) & 0xFF) / 255.0
        let b = CGFloat(value & 0xFF) / 255.0
        return NSColor(red: r, green: g, blue: b, alpha: 1.0)
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

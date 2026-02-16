import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let delegate = RanmaAppDelegate()
app.delegate = delegate
app.run()

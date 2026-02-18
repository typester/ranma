import AppKit

enum ImageCache {
    static let shared = Cache()

    final class Cache: @unchecked Sendable {
        private struct Entry {
            let image: NSImage
            let mtime: Date
            var lastAccess: UInt64
        }

        private var cache: [String: Entry] = [:]
        private var counter: UInt64 = 0

        func image(for path: String) -> NSImage? {
            let currentMtime = mtime(of: path)

            if var entry = cache[path] {
                if let currentMtime, entry.mtime == currentMtime {
                    counter += 1
                    entry.lastAccess = counter
                    cache[path] = entry
                    return entry.image
                }
                cache.removeValue(forKey: path)
            }

            guard let img = NSImage(contentsOfFile: path) else {
                return nil
            }
            counter += 1
            evictIfNeeded()
            cache[path] = Entry(image: img, mtime: currentMtime ?? .distantPast, lastAccess: counter)
            return img
        }

        private func mtime(of path: String) -> Date? {
            try? FileManager.default.attributesOfItem(atPath: path)[.modificationDate] as? Date
        }

        private func evictIfNeeded() {
            guard cache.count > 64 else { return }
            let sorted = cache.sorted { $0.value.lastAccess < $1.value.lastAccess }
            for (key, _) in sorted.prefix(cache.count / 2) {
                cache.removeValue(forKey: key)
            }
        }
    }
}

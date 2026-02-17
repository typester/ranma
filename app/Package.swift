// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Ranma",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(
            name: "CRanmaCore",
            path: "Sources/CRanmaCore"
        ),
        .executableTarget(
            name: "ranma-server",
            dependencies: ["CRanmaCore"],
            path: "Sources",
            exclude: ["CRanmaCore"],
            linkerSettings: [
                .linkedLibrary("ranma_core"),
                .unsafeFlags(["-F", "/System/Library/PrivateFrameworks"]),
                .linkedFramework("SkyLight"),
            ]
        ),
    ]
)

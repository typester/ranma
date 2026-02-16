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
            name: "Ranma",
            dependencies: ["CRanmaCore"],
            path: "Sources",
            exclude: ["CRanmaCore"],
            linkerSettings: [
                .linkedLibrary("ranma_core"),
            ]
        ),
    ]
)

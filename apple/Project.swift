import ProjectDescription

let deploymentTarget = DeploymentTargets.macOS("14.0")

let project = Project(
    name: "MadobeMac",
    settings: .settings(base: [
        "ENABLE_USER_SCRIPT_SANDBOXING": "YES",
        "MACOSX_DEPLOYMENT_TARGET": "14.0",
        "SWIFT_VERSION": "6.0",
    ]),
    targets: [
        .target(
            name: "MadobeClientCore",
            destinations: .macOS,
            product: .framework,
            bundleId: "dev.catcave.madobe.clientcore",
            deploymentTargets: deploymentTarget,
            infoPlist: .default,
            sources: ["Sources/MadobeClientCore/**"]
        ),
        .target(
            name: "MadobeMac",
            destinations: .macOS,
            product: .app,
            bundleId: "dev.catcave.madobe.mac",
            deploymentTargets: deploymentTarget,
            infoPlist: .extendingDefault(with: [
                "CFBundleDisplayName": "Madobe",
                "LSMinimumSystemVersion": "14.0",
            ]),
            sources: ["Sources/MadobeMac/**"],
            dependencies: [
                .target(name: "MadobeClientCore"),
            ]
        ),
        .target(
            name: "MadobeClientCoreTests",
            destinations: .macOS,
            product: .unitTests,
            bundleId: "dev.catcave.madobe.clientcore.tests",
            deploymentTargets: deploymentTarget,
            infoPlist: .default,
            sources: ["Tests/MadobeClientCoreTests/**"],
            dependencies: [
                .target(name: "MadobeClientCore"),
            ]
        ),
    ],
    schemes: [
        .scheme(
            name: "MadobeMac",
            shared: true,
            buildAction: .buildAction(targets: ["MadobeMac"]),
            testAction: .targets(["MadobeClientCoreTests"])
        ),
    ]
)

# Firezone Apple Client

Firezone clients for macOS and iOS.

## Pre-requisites

1. Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
1. Request your Firezone email added to our Apple Developer Account
1. Open Xcode, go to Settings -> Account and log in.

If you're working on the macOS client, you'll need to disable SIP and enable
system extension development mode:

1. Follow [these instructions](https://developer.apple.com/documentation/security/disabling-and-enabling-system-integrity-protection) to disable SIP.
1. After that's complete, turn on system extension development mode:

```bash
systemextensionsctl developer on
```

This will prevent macOS from blocking the Network Extension from loading due to notarization or filepath restrictions.

**Be sure to re-enable SIP to test the app in a production-like environment.**

You may consider using a macOS VM (such as Parallels Desktop) to develop the macOS client, as it's easier to
disable SIP, take snapshots, and muck around with your system configuration without risking your main machine.

## Building

1. Add required Rust targets:

   Ensure you've activated the correct toolchain version for your local
   environment with `rustup default <toolchain>` (find this from
   `/rust/rust-toolchain.toml` file), then run:

   ```
   rustup target add aarch64-apple-ios aarch64-apple-darwin x86_64-apple-darwin
   ```

1. Clone this repo:

   ```bash
   git clone https://github.com/firezone/firezone
   ```

1. `cd` to the Apple clients code

   ```bash
   cd swift/apple
   ```

1. Copy an appropriate xcconfig:

If building for Development, the debug.xcconfig should work out-of-the-box.

```bash
cp Firezone/xcconfig/debug.xcconfig Firezone/xcconfig/config.xcconfig
vim Firezone/xcconfig/config.xcconfig
```

1. Open project in Xcode:

```bash
open Firezone.xcodeproj
```

1. Build and run the `Firezone` target.

### Making macOS standalone release builds for local testing

For the macOS standalone app, it's a good idea to smoke test release builds of the app
whenever anything related to the app's packaging or notarization changes. This
is because standalone binaries on macOS don't go through the App Store submission
and distribution process which typically catches any packaging issues. Standalone binaries
are subject to Gatekeeper restrictions which means they can build and run just fine in
Development but fail to run successfully on another user's machine.

To build a standalone release binary:

1. Go to https://developer.apple.com/account/resources/certificates/list.
1. Download the "Developer ID Application" certificate which has the latest expiration date and double-click it to install it in your keychain.
1. Go to https://developer.apple.com/account/resources/profiles/list.
1. Download both of the "Developer ID Application" provisioning profiles (one each for the App and Network Extension).
1. Copy the `standalone` release xcconfig:

```bash
cp Firezone/xcconfig/standalone.xcconfig Firezone/xcconfig/config.xcconfig
```

1. Open Xcode, drag the provisioning profiles onto the Xcode app icon in the Dock to install them.
1. In Xcode, ensure the `Firezone` scheme is selected, then go to `Product -> Archive`. This will build the app and open the Organizer window.
1. In the Organizer window, select the latest build and click `Distribute App`.
1. Choose `Direct Distribution` and click `Distribute`.
1. Apple will then sign and notarize the app. Notarization typically takes a minute or two, but can take up to an hour during busy times.
1. Once notarization is complete, Xcode will notify you that the app is ready to distribute. Click `Export` and save the app to the `/Applications` folder. macOS will not allow system extensions to be activated unless they are in `/Applications`.
1. Launch the app from `/Applications` and ensure it works as expected.

Because it can be a bit of a hassle to ensure your development machine will mimic the behavior of a user's machine, it's a good idea to test standalone builds on a clean macOS VM. Parallels for Mac is a good choice for this.

## Debugging

[This Network Extension debugging guide](https://developer.apple.com/forums/thread/725805)
is a great resource to use as a starting point.

### Debugging on iOS simulator

Network Extensions
[can't be debugged](https://developer.apple.com/forums/thread/101663) in the iOS
simulator, so you'll need a physical iOS device to develop the iOS build on.

### NetworkExtension not loading (macOS)

If the tunnel fails to come up after signing in, it can be for a large number of reasons. Here are some of the more common ones:

Try clearing your LaunchAgent db:

```bash
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -delete
```

**Note**: You MUST reboot after doing this!

### Outdated version of NetworkExtension loading

If you're making changes to the Network Extension and it doesn't seem to be
reflected when you run/debug, it could be that PluginKit is still launching your
old NetworkExtension. Try this to remove it:

```bash
pluginkit -v -m -D -i <bundle-id>
pluginkit -a <path>
pluginkit -r <path>
```

## Cleaning up

Occasionally you might encounter strange issues where it seems like the
artifacts being debugged don't match the code, among other things. In these
cases it's good to clean up using one of the methods below.

### Resetting Xcode package cache

Removes cached packages, built extensions, etc.

```bash
rm -rf ~/Library/Developer/Xcode/DerivedData
```

### Removing build artifacts

To cleanup Swift build objects:

```bash
cd swift/apple
./cleanup.sh
```

To cleanup both Swift and Rust build objects:

```bash
cd swift/apple
./cleanup.sh all
```

### Wiping connlib log directory

```
rm -rf $HOME/Library/Group\ Containers/47R2M6779T.dev.firezone.firezone/Library/Caches/logs/connlib
```

### Clearing the Keychain item

Sometimes it's helpful to be able to test how the app behaves when the keychain
item is missing. You can remove the keychain item with the following command:

```bash
security delete-generic-password -s "dev.firezone.firezone"
```

## Generating new signing certificates and provisioning profiles

Certs are only good for a year, then you need to generate new ones. Since we use
GitHub CI, we have to use manually-managed signing and provisioning. Here's how
you populate the required GitHub secrets.

**Note**: Be sure to enter these variables for Dependabot as well, otherwise its
CI runs will fail.

### Certificates

You first need two certs: The build / signing cert (Apple Distribution) and the
installer cert (Mac Installer Distribution). You can generate these in the Apple
Developer portal.

These are the secrets in GH actions:

```
APPLE_BUILD_CERTIFICATE_BASE64
APPLE_BUILD_CERTIFICATE_P12_PASSWORD
APPLE_MAC_INSTALLER_CERTIFICATE_BASE64
APPLE_MAC_INSTALLER_CERTIFICATE_P12_PASSWORD
```

How to do it:

1. Go to
   [Apple Developer](https://developer.apple.com/account/resources/certificates/list)
1. Click the "+" button to generate a new distribution certificate for App Store
1. It will ask for a CSR. Open Keychain Access, go to Keychain Access ->
   Certificate Assistant -> Request a Certificate from a Certificate Authority
   and follow the prompts. Make sure to select "save to disk" to save the CSR.
1. Upload the CSR to Apple Developer. Download the resulting certificate.
   Double-click to install it in Keychain Access.
1. Right-click the cert in Keychain access. Export the certificate, choose p12
   file. Make sure to set a password -- this is the
   `APPLE_BUILD_CERTIFICATE_P12_PASSWORD`.
1. Convert the p12 file to base64:
   ```bash
   cat cert.p12 | base64
   ```
1. Save the base64 output as `APPLE_BUILD_CERTIFICATE_BASE64`.

Repeat the steps above but choose "Mac Installer certificate" instead of
"distribution certificate" in step 2, and save the resulting base64 and password
as `APPLE_MAC_INSTALLER_CERTIFICATE_BASE64` and
`APPLE_MAC_INSTALLER_CERTIFICATE_P12_PASSWORD`.

### Provisioning profiles

```
APPLE_IOS_APP_PROVISIONING_PROFILE
APPLE_IOS_NE_PROVISIONING_PROFILE
APPLE_MACOS_APP_PROVISIONING_PROFILE
APPLE_MACOS_NE_PROVISIONING_PROFILE
```

1. Go to
   [Apple Developer](https://developer.apple.com/account/resources/profiles/list)
1. Click the "+" button to generate a new provisioning profile for App Store
1. Select the appropriate app ID and distribution certificate you just created.
   You'll need a provisioning profile for each app and network extension, so 4
   total (mac app, mac network extension, ios app, ios network extension).
1. Download the resulting provisioning profiles.
1. Encode to base64 and save each using the secrets names above:

```bash
cat profile.mobileprovision | base64
```

1. Now, you need to update the XCConfig to use these. Edit
   Firezone/xcconfig/release.xcconfig and update the provisioning profile UUIDs.
   The UUID can be found by grepping for them in the provisioning profile files
   themselves, or just opening them in a text editor and looking halfway down
   the file.
1. Now, for iOS only, you need to edit Firezone/ExportOptions.plist and update
   the provisioning profile UUIDs there as well.

### Runner keychain password

This can be randomly generated. It's only used ephemerally to load the secrets
into the runner's keychain for the build.

```
APPLE_RUNNER_KEYCHAIN_PASSWORD
```

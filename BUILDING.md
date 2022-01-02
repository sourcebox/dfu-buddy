# Building

## Requirements

- [Rust toolchain](https://www.rust-lang.org/)
- C/C++ compiler (for building libusb)

Dependencies:

    - libusb
    - pkg-config

On Linux, a couple of additional dependencies must be installed:

    - libxcb-render0-dev
    - libxcb-shape0-dev
    - libxcb-xfixes0-dev
    - libspeechd-dev
    - libxkbcommon-dev

USB access is provided via the [rusb](https://github.com/a1ien/rusb) crate, which uses pkg-config to locate the libusb sources. Make sure the sources are in a location where pkg-config can find them.

### Mac Application Bundle (optional)

To build a macOS application bundle, additional dependencies must be installed:

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
- [Python3](https://python.org) (any recent version should work)

Run `./build-mac-bundle.sh` from the project directory. Make sure the script has executable permissions.
The bundle will be created in the `./target/release/bundle/osx` directory.

### Linux AppImage (optional)

To build an AppImage for Linux, additional dependencies must be installed:

- [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy)
- [linuxdeploy-plugin-appimage](https://github.com/linuxdeploy/linuxdeploy-plugin-appimage)

Run `./build-linux-appimage.sh` from the project directory. Make sure the script has executable permissions.
The AppImage will be created in the `./target/release/appimage` directory.

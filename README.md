# DFU Buddy

## About

DFU Buddy is a cross-platform GUI utility for performing firmware upgrades on embedded devices via USB.

It is based on the DFU standard [USB Device Firmware Upgrade Specification, Revision 1.1](https://usb.org/sites/default/files/DFU_1.1.pdf), published by the [USB Implementers Forum](https://www.usb.org) and supports the DfuSe extensions by STMicroelectronics.

Operation is designed to be simple and straightforward for non-expert users. Therefore, a number of more advanced (and potentially dangerous) features are not provided. If you need these and you know what you're doing, use a tool like [dfu-util](http://dfu-util.sourceforge.net/).

## Status

DFU Buddy is still work in progress and lacking functionality. Also, some devices don't work yet. Support by other users, mainly in form of testing with USB devices is highly appreciated.

- Devices must be in DFU mode to appear in the selection menu.
- Plain DFU is not supported yet, only DfuSe devices like STM32.
- Only the internal flash of STM32 MCUs can be programmed, no OTP, no option bytes.
- Workarounds for specific non-compliant devices are not implemented.
- Tests were done using the following devices:
    - STM32L433VC internal bootloader: working
    - STM32G474VC internal bootloader: working
    - STM32F405RG (pyboard) internal bootloader: not working (overflow error).

## Build Requirements

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

## License

Published under the MIT license.

Author: Oliver Rockstedt <info@sourcebox.de>

## Donations

If you like to support my work, you can [buy me a coffee.](https://www.buymeacoffee.com/sourcebox)

<a href="https://www.buymeacoffee.com/sourcebox" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>

# DFU Buddy

## About

DFU Buddy is a cross-platform GUI utility for performing firmware upgrades on embedded devices via USB.

![Screenshot](screenshot.png)

It is based on the DFU standard [USB Device Firmware Upgrade Specification, Revision 1.1](https://usb.org/sites/default/files/DFU_1.1.pdf), published by the [USB Implementers Forum](https://www.usb.org) and supports the DfuSe extensions by STMicroelectronics.

Operation is designed to be simple and straightforward for non-expert users. Therefore, a number of more advanced (and potentially dangerous) features are not provided. If you need these and you know what you're doing, use a tool like [dfu-util](http://dfu-util.sourceforge.net/).

## Status

DFU Buddy is still work in progress and lacking functionality. Also, some devices don't work yet. Support by other users, mainly in form of testing with USB devices is highly appreciated.

- Devices must be in DFU mode to appear in the selection menu.
- Plain DFU is not supported yet, only DfuSe devices like STM32.
- Only the internal flash of STM32 MCUs can be programmed, no OTP, no option bytes.
- Workarounds for specific non-compliant devices are not implemented.
- Tests were done using the internal ROM bootloader on the following devices:
  - STM32L433VC
  - STM32G474VC
  - STM32F405RG
  - STM32F303VC (STM32F3DISCOVERY)

## Usage

- Download and install the package for your platform from the [releases page](https://github.com/sourcebox/dfu-buddy/releases/latest).
- **Important**:
  - When running Windows, a USB DFU driver suitable for your device must be installed.
  - On Linux, make sure that you have setup the udev rules correctly. Otherwise, your user account will not have the required access permissions.
- Connect the hardware device to be updated and power it up in DFU mode. Refer to the user manual of the device for specific instructions on how to enter this mode.
- Launch the application. Depending on the platform, there may be security warnings about being from an untrusted developer or source. You have to accept these warnings or [build the application from source](BUILDING.md) yourself. This is a common issue for open source applications because they are not signed by their developers at the OS manufacturers.
- Select the device from the *Device* dropdown menu. Please note that it may show a generic name like *STM32 Bootloader* instead of its usual brand name.
- Select the DFU file containing the firmware by either clicking the *Open...* button and choosing it via the file dialog or by dropping the file onto the application window.
- After having selected both device and file, some checks are performed to prove that they match. This is done to prevent accidently flashing the device with a wrong firmware that is intended for some other unit.
- Check to *Confirm to proceed* checkbox in the lower left corner.
- Press the *Start update* button to initiate to update process.
- The update procedure will now start. 3 steps are executed: erasing the old firmware, writing the new one, verifying the written data. Each steps progress is shown by bar in the lower right corner.
- After all steps are finished, a result message is displayed.
- Close the application and restart the device in normal mode. The new firmware should now be running.
- The application window can be zoomed via key commands:
  - macOS: <kbd>Cmd</kbd> + <kbd>+</kbd>,  <kbd>Cmd</kbd> + <kbd>-</kbd> and  <kbd>Cmd</kbd> + <kbd>0</kbd>.
  - Windows/Linux: <kbd>Ctrl</kbd> + <kbd>+</kbd>,  <kbd>Ctrl</kbd> + <kbd>-</kbd> and  <kbd>Ctrl</kbd> + <kbd>0</kbd>.

## Building from Source

See [separate document](BUILDING.md) for detailed instructions.

## License

Published under the MIT license. All contributions to this project must be provided under the same license conditions.

Author: Oliver Rockstedt <info@sourcebox.de>

## Donations

If you like to support my work, you can [buy me a coffee.](https://www.buymeacoffee.com/sourcebox)

<a href="https://www.buymeacoffee.com/sourcebox" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>

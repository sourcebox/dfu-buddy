#!/bin/bash

cargo build --release

rm -rf target/release/appimage/*

linuxdeploy-x86_64.AppImage \
    --executable ./target/release/dfu-buddy \
    --desktop-file ./tools/dfu-buddy.desktop \
    --icon-file ./tools/dfu-buddy.png \
    --appdir ./target/release/appimage/AppDir \
    --output appimage

echo "Moving appimage to target directory"
mv *.AppImage ./target/release/appimage/

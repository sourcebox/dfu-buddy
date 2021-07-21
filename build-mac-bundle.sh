#!/bin/bash

BUNDLE_DIR="./target/release/bundle/osx/DFU Buddy.app"
POST_BUILD_SCRIPT="./tools/mac_bundle_post_build.py"

cargo bundle --release

echo "Running post build script $POST_BUILD_SCRIPT"
chmod 755 "$POST_BUILD_SCRIPT"
$POST_BUILD_SCRIPT "$BUNDLE_DIR"

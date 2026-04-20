#!/bin/bash
# Remove 'target' from path in WiX files to fix Windows installer build
# This addresses the path mismatch issue when creating MSI with WiX

echo "Fixing WiX file paths..."

# Check if the Wix directory exists
if [ -d "wix" ]; then
    # Process main.wxs file to remove target directory from paths
    sed -i 's|target/||g' wix/main.wxs
    echo "Fixed paths in wix/main.wxs"
else
    echo "Wix directory not found"
    exit 1
fi

echo "WiX path fix completed"
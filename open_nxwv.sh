#!/bin/bash

# make it available
# chmod +x open_nxwv.sh
# open it
# ./open_flutter.sh

# Get the absolute path of the directory where this script lives (your project root)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FLUTTER_DIR="$PROJECT_ROOT/nxwv"

# Check if the nxwv folder actually exists
if [ ! -d "$FLUTTER_DIR" ]; then
    echo "Error: The Flutter project directory 'nxwv' was not found at:"
    echo "       $FLUTTER_DIR"
    exit 1
fi

echo "Opening 'nxwv' in a new VS Code window..."

# Open VS Code in a new window (-n / --new-window) targeting the flutter folder
code -n "$FLUTTER_DIR"
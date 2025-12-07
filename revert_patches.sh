#!/bin/bash

# Simple script to revert all Android patches
# set -e  # Disabled to prevent early exit on individual patch failures

PATCHES_DIR="android-patches"
SUCCESS_COUNT=0
FAIL_COUNT=0
DELETED_FILES=0

echo "🔄 Starting patch reversion..."

# Function to check if patch creates a new file
is_new_file_patch() {
    local patch="$1"
    # Check for @@ -0,0 +1, pattern which indicates a new file
    # AND ensure there are no deletion lines (lines starting with - followed by a number)
    if grep -q "^@@ -0,0" "$patch" && ! grep -q "^-[0-9]" "$patch"; then
        return 0
    fi
    return 1
}

# Function to get target file from patch
get_target_file() {
    local patch="$1"
    grep "^+++ b/" "$patch" | sed 's/^+++ b\///' | head -1
}

# Find and revert all patches
while IFS= read -r -d '' patch; do
    patch_name=${patch#$PATCHES_DIR/}
    echo "🔧 Reverting: $patch_name"
    
    if git apply -R "$patch" 2>/dev/null; then
        echo "  ✅ Success"
        ((SUCCESS_COUNT++))
        
        # Delete file if this was a new file patch
        if is_new_file_patch "$patch"; then
            target_file=$(get_target_file "$patch")
            if [[ -n "$target_file" && -f "$target_file" ]]; then
                echo "  🗑️  Deleting: $target_file"
                rm -f "$target_file"
                ((DELETED_FILES++))
            fi
        fi
    else
        echo "  ❌ Failed"
        ((FAIL_COUNT++))
    fi
done < <(find "$PATCHES_DIR" -name "*.patch" -type f -print0 | sort -z)

echo
echo "📊 Summary:"
echo "  Reverted: $SUCCESS_COUNT"
echo "  Failed: $FAIL_COUNT"
echo "  Files deleted: $DELETED_FILES"
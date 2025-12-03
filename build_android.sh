#!/bin/bash

# Android Build Script for codex-rust-bindings
# This script builds the codex-rust-bindings for Android arm64-v8a architecture

set -e  # Exit on any error

# Default values
TARGET_ARCH="aarch64-linux-android"
LINKING_MODE="dynamic"  # Can be "static" or "dynamic"
CLEAN_BUILD=false
VERBOSE=false
CLEAN_PATCHES=false
VALIDATE_PATCHES=true

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build codex-rust-bindings for Android with automatic patch management

OPTIONS:
    -n, --ndk-path PATH       Android NDK path (default: $ANDROID_NDK_HOME)
    -s, --sdk-path PATH       Android SDK path (default: $ANDROID_SDK_ROOT)
    -a, --arch ARCH           Target architecture (default: $TARGET_ARCH)
    -m, --mode MODE           Linking mode: static or dynamic (default: $LINKING_MODE)
    -c, --clean               Clean build before building
    -p, --clean-patches       Clean patch backups before building
    --no-validate-patches     Skip patch validation (not recommended)
    -v, --verbose             Verbose output
    -h, --help                Show this help message

ENVIRONMENT VARIABLES:
    ANDROID_NDK_HOME          Android NDK path
    ANDROID_SDK_ROOT          Android SDK path

EXAMPLES:
    $0                        # Build with default settings
    $0 -m static              # Build with static linking
    $0 -c -v                  # Clean build with verbose output
    $0 -n /opt/android-ndk    # Use custom NDK path
    $0 -p -c                  # Clean build with patch cleanup

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--ndk-path)
            ANDROID_NDK_HOME="$2"
            shift 2
            ;;
        -s|--sdk-path)
            ANDROID_SDK_ROOT="$2"
            shift 2
            ;;
        -a|--arch)
            TARGET_ARCH="$2"
            shift 2
            ;;
        -m|--mode)
            LINKING_MODE="$2"
            if [[ ! "$LINKING_MODE" =~ ^(static|dynamic)$ ]]; then
                print_error "Invalid linking mode: $LINKING_MODE. Must be 'static' or 'dynamic'"
                exit 1
            fi
            shift 2
            ;;
        -c|--clean)
            CLEAN_BUILD=true
            shift
            ;;
        -p|--clean-patches)
            CLEAN_PATCHES=true
            shift
            ;;
        --no-validate-patches)
            VALIDATE_PATCHES=false
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate Android NDK
if [[ ! -d "$ANDROID_NDK_HOME" ]]; then
    print_error "Android NDK not found at: $ANDROID_NDK_HOME"
    print_error "Please install Android NDK or set ANDROID_NDK_HOME environment variable"
    exit 1
fi

# Validate Android SDK
if [[ ! -d "$ANDROID_SDK_ROOT" ]]; then
    print_error "Android SDK not found at: $ANDROID_SDK_ROOT"
    print_error "Please install Android SDK or set ANDROID_SDK_ROOT environment variable"
    exit 1
fi

# Set up environment
print_status "Setting up Android build environment..."
export ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export ANDROID_NDK_HOME="$ANDROID_NDK_HOME"

# Determine architecture-specific settings
case $TARGET_ARCH in
    aarch64-linux-android|arm64)
        ARCH="arm64"
        LLVM_TRIPLE="aarch64-linux-android"
        # Set the correct target for cargo
        if [ "$TARGET_ARCH" = "arm64" ]; then
            TARGET_ARCH="aarch64-linux-android"
        fi
        ;;
    armv7-linux-androideabi|arm)
        ARCH="arm"
        LLVM_TRIPLE="armv7a-linux-androideabi"
        # Set the correct target for cargo
        if [ "$TARGET_ARCH" = "arm" ]; then
            TARGET_ARCH="armv7-linux-androideabi"
        fi
        ;;
    x86_64-linux-android|amd64)
        ARCH="amd64"
        LLVM_TRIPLE="x86_64-linux-android"
        # Set the correct target for cargo
        if [ "$TARGET_ARCH" = "amd64" ]; then
            TARGET_ARCH="x86_64-linux-android"
        fi
        ;;
    i686-linux-android|x86)
        ARCH="386"
        LLVM_TRIPLE="i686-linux-android"
        # Set the correct target for cargo
        if [ "$TARGET_ARCH" = "x86" ]; then
            TARGET_ARCH="i686-linux-android"
        fi
        ;;
    *)
        print_error "Unsupported architecture: $TARGET_ARCH"
        exit 1
        ;;
esac

# Set up toolchain paths
TOOLCHAIN_PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
CC="$TOOLCHAIN_PATH/${LLVM_TRIPLE}21-clang"
CXX="$TOOLCHAIN_PATH/${LLVM_TRIPLE}21-clang++"
AR="$TOOLCHAIN_PATH/llvm-ar"
RANLIB="$TOOLCHAIN_PATH/llvm-ranlib"

# Validate tools
for tool in "$CC" "$CXX" "$AR" "$RANLIB"; do
    if [[ ! -x "$tool" ]]; then
        print_error "Tool not found: $tool"
        exit 1
    fi
done

# Clean build if requested or if existing library is wrong architecture
NEEDS_CLEAN=false
if [[ "$CLEAN_BUILD" == "true" ]]; then
    NEEDS_CLEAN=true
elif [[ -f "vendor/nim-codex/build/libcodex.so" ]]; then
    # Check if existing library is wrong architecture
    if file vendor/nim-codex/build/libcodex.so | grep -q "x86-64"; then
        if [[ "$TARGET_ARCH" == "aarch64-linux-android" ]]; then
            print_status "Existing library is x86-64, need to rebuild for ARM64"
            NEEDS_CLEAN=true
        fi
    elif file vendor/nim-codex/build/libcodex.so | grep -q "aarch64"; then
        if [[ "$TARGET_ARCH" != "aarch64-linux-android" ]]; then
            print_status "Existing library is ARM64, need to rebuild for $TARGET_ARCH"
            NEEDS_CLEAN=true
        fi
    fi
fi

if [[ "$NEEDS_CLEAN" == "true" ]]; then
    print_status "Cleaning build directory..."
    cargo clean
    if [[ -d "vendor/nim-codex/build" ]]; then
        rm -rf vendor/nim-codex/build
    fi
fi

# Clean patch backups if requested
if [[ "$CLEAN_PATCHES" == "true" ]]; then
    print_status "Cleaning patch backups..."
    if [[ -d "target/patch_backups" ]]; then
        rm -rf target/patch_backups
    fi
fi

# Validate patch system
if [[ "$VALIDATE_PATCHES" == "true" ]]; then
    print_status "Validating Android patch system..."
    
    # Validate patches for the target architecture
    case $ARCH in
        arm64)
            PATCH_ARCH="arm64"
            ;;
        arm)
            PATCH_ARCH="arm32"
            ;;
        amd64)
            PATCH_ARCH="x86_64"
            ;;
        386)
            PATCH_ARCH="x86"
            ;;
        *)
            print_error "Unknown patch architecture mapping for: $ARCH"
            exit 1
            ;;
    esac
    
    # Check if patches directory exists
    if [[ ! -d "android-patches" ]]; then
        print_error "Patch directory not found: android-patches/"
        exit 1
    fi
    
    # Check if architecture-specific patches exist
    if [[ ! -d "android-patches/$PATCH_ARCH" ]]; then
        print_error "No patches found for architecture: $PATCH_ARCH"
        exit 1
    fi
    
    # Count patches for this architecture using recursive discovery
    ARCH_PATCH_COUNT=$(find "android-patches/$PATCH_ARCH" -name "*.patch" | wc -l)
    SHARED_PATCH_COUNT=$(find "android-patches/shared" -name "*.patch" 2>/dev/null | wc -l)
    TOTAL_PATCH_COUNT=$((ARCH_PATCH_COUNT + SHARED_PATCH_COUNT))
    
    print_status "Found $ARCH_PATCH_COUNT architecture-specific patches for $PATCH_ARCH"
    print_status "Found $SHARED_PATCH_COUNT shared patches"
    print_status "Total: $TOTAL_PATCH_COUNT patches available"
    
    if [[ $TOTAL_PATCH_COUNT -eq 0 ]]; then
        print_error "No patches found for architecture: $PATCH_ARCH"
        exit 1
    fi
    
    print_status "Patch system validation passed for $PATCH_ARCH"
fi

# Set Rust target
print_status "Installing Rust target: $TARGET_ARCH"
rustup target add "$TARGET_ARCH"

# Build command
BUILD_CMD="cargo build --target $TARGET_ARCH"
if [[ "$LINKING_MODE" == "static" ]]; then
    BUILD_CMD="$BUILD_CMD --features static-linking"
else
    BUILD_CMD="$BUILD_CMD --features dynamic-linking"
fi
BUILD_CMD="$BUILD_CMD --features android-patches"
BUILD_CMD="$BUILD_CMD --release"

if [[ "$VERBOSE" == "true" ]]; then
    BUILD_CMD="$BUILD_CMD --verbose"
fi

# Print build configuration
print_status "Build configuration:"
echo "  Target Architecture: $TARGET_ARCH"
echo "  Patch Architecture: $PATCH_ARCH"
echo "  Linking Mode: $LINKING_MODE"
echo "  Android NDK: $ANDROID_NDK_HOME"
echo "  Android SDK: $ANDROID_SDK_ROOT"
echo "  C Compiler: $CC"
echo "  C++ Compiler: $CXX"
echo "  Archiver: $AR"
echo "  Patch System: Enabled"
echo "  Validate Patches: $VALIDATE_PATCHES"
echo "  Clean Patches: $CLEAN_PATCHES"
echo "  Build Command: $BUILD_CMD"
echo ""

# Execute build
print_status "Starting build for Android $TARGET_ARCH..."
if [[ "$VERBOSE" == "true" ]]; then
    print_status "Running: $BUILD_CMD"
fi

# Set environment variables for cargo
# Convert target arch to valid env var format (replace - with _)
TARGET_ARCH_ENV=$(echo "$TARGET_ARCH" | tr '-' '_')
export CC_"$TARGET_ARCH_ENV"="$CC"
export CXX_"$TARGET_ARCH_ENV"="$CXX"
export AR_"$TARGET_ARCH_ENV"="$AR"
export RANLIB_"$TARGET_ARCH_ENV"="$RANLIB"

# CRITICAL: Also set TARGET for Rust build system to detect Android builds
export TARGET="$TARGET_ARCH"

# Run the build
if eval "$BUILD_CMD"; then
    print_status "Build completed successfully!"
    
    # Show patch information
    if [[ -d "target/patch_backups/$PATCH_ARCH" ]]; then
        BACKUP_COUNT=$(find "target/patch_backups/$PATCH_ARCH" -name "backup_*" | wc -l)
        print_status "Applied patches with $BACKUP_COUNT backup files created"
    fi
    
    # Show output files
    LIB_DIR="target/$TARGET_ARCH/release"
    if [[ -d "$LIB_DIR" ]]; then
        print_status "Output files in $LIB_DIR:"
        ls -la "$LIB_DIR"/libcodex* 2>/dev/null || true
    fi
    
    print_status "Android build with patch system completed successfully!"
else
    print_error "Build failed!"
    print_error "Check the build output above for patch application errors"
    exit 1
fi
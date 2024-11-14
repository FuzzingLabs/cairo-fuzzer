#!/bin/bash

###
### This script automates the setup of the Cairo 2 compiler and Scarb on Linux and macOS systems.
### It downloads the appropriate Cairo release, decompresses it, installs Scarb, and sets up
### the necessary environment variables for macOS. Additionally, it creates a symbolic link
### to the Cairo core library.
###

set -e

UNAME=$(uname)
CAIRO_2_VERSION=2.8.4
SCARB_VERSION=2.8.4

# Decompress the Cairo tarball
function decompress_cairo {
    local source=$1
    local target=$2
    rm -rf "$target"
    tar -xzvf "$source"
    mv cairo/ "$target"
}

# Download the Cairo tarball
function download_cairo {
    local version=$1
    local os=$2
    local url=""

    if [ "$os" == "macos" ]; then
        url="https://github.com/starkware-libs/cairo/releases/download/v${version}/release-aarch64-apple-darwin.tar"
    else
        url="https://github.com/starkware-libs/cairo/releases/download/v${version}/release-x86_64-unknown-linux-musl.tar.gz"
    fi

    curl -L -o "cairo-${version}-${os}.tar" "$url"
}

# Install Scarb
function install_scarb {
    curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh -s -- --no-modify-path --version "$SCARB_VERSION"
}

# Build the Cairo 2 compiler
function build_cairo_2_compiler {
    local os=$1
    local cairo_dir="cairo2"

    if [ "$os" == "macos" ]; then
        cairo_dir="cairo2-macos"
    fi

    download_cairo "$CAIRO_2_VERSION" "$os"
    decompress_cairo "cairo-${CAIRO_2_VERSION}-${os}.tar" "$cairo_dir"
}

# Install dependencies for macOS
function deps_macos {
    build_cairo_2_compiler "macos"
    install_scarb
    brew install llvm@19 --quiet
    echo "You can execute the env-macos.sh script to setup the needed env variables."
}

# Install dependencies for Linux
function deps_linux {
    build_cairo_2_compiler "linux"
    install_scarb
}

# Determine the OS and call the appropriate function
function main {
    if [ "$UNAME" == "Linux" ]; then
        deps_linux
    elif [ "$UNAME" == "Darwin" ]; then
        deps_macos
    else
        echo "Unsupported operating system: $UNAME"
        exit 1
    fi

    rm -rf corelib
    ln -s cairo2/corelib corelib
}

main

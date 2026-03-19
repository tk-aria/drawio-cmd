#!/bin/sh
set -e

DEFAULT_INSTALL_PATH="/usr/local/bin"
REPO="tk-aria/drawio-tools"
BINARY_NAME="drawio-tools"

_latest_version() {
    curl -sSfL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
}

_detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-musl" ;;
        Darwin*) echo "apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) echo "pc-windows-msvc" ;;
        *) echo "unsupported" ;;
    esac
}

_detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) echo "unsupported" ;;
    esac
}

_get_ext() {
    case "$1" in
        *windows*) echo "zip" ;;
        *) echo "tar.gz" ;;
    esac
}

_download_url() {
    local version="$1"
    local target="$2"
    local ext
    ext=$(_get_ext "$target")
    echo "https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${version}-${target}.${ext}"
}

cmd_install() {
    local install_path="${DRAWIO_TOOLS_INSTALL_PATH:-$DEFAULT_INSTALL_PATH}"
    local version="${DRAWIO_TOOLS_VERSION:-$(_latest_version)}"
    local arch
    arch=$(_detect_arch)
    local os
    os=$(_detect_os)

    if [ "$arch" = "unsupported" ] || [ "$os" = "unsupported" ]; then
        echo "Error: Unsupported platform: $(uname -m) / $(uname -s)" >&2
        exit 1
    fi

    local target="${arch}-${os}"
    local url
    url=$(_download_url "$version" "$target")
    local ext
    ext=$(_get_ext "$target")

    echo "Installing ${BINARY_NAME} ${version} for ${target}..."
    echo "  Download: ${url}"
    echo "  Install path: ${install_path}"

    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -sSfL "$url" -o "${tmpdir}/archive.${ext}"

    case "$ext" in
        tar.gz)
            tar xzf "${tmpdir}/archive.tar.gz" -C "$tmpdir"
            ;;
        zip)
            unzip -q "${tmpdir}/archive.zip" -d "$tmpdir"
            ;;
    esac

    local binary_file
    binary_file=$(find "$tmpdir" -name "$BINARY_NAME" -type f | head -1)
    if [ -z "$binary_file" ]; then
        binary_file=$(find "$tmpdir" -name "${BINARY_NAME}.exe" -type f | head -1)
    fi

    if [ -z "$binary_file" ]; then
        echo "Error: Binary not found in archive" >&2
        exit 1
    fi

    chmod +x "$binary_file"

    if [ -w "$install_path" ]; then
        cp "$binary_file" "${install_path}/"
    else
        echo "Requires elevated permissions to install to ${install_path}"
        sudo cp "$binary_file" "${install_path}/"
    fi

    echo "Successfully installed ${BINARY_NAME} ${version} to ${install_path}/${BINARY_NAME}"
    "${install_path}/${BINARY_NAME}" --version 2>/dev/null || true
}

cmd_uninstall() {
    local install_path="${DRAWIO_TOOLS_INSTALL_PATH:-$DEFAULT_INSTALL_PATH}"
    local binary="${install_path}/${BINARY_NAME}"

    if [ ! -f "$binary" ]; then
        echo "${BINARY_NAME} is not installed at ${binary}" >&2
        exit 1
    fi

    if [ -w "$install_path" ]; then
        rm -f "$binary"
    else
        echo "Requires elevated permissions to uninstall from ${install_path}"
        sudo rm -f "$binary"
    fi

    echo "Successfully uninstalled ${BINARY_NAME} from ${install_path}"
}

usage() {
    cat <<USAGE
Usage: $0 <command>

Commands:
  install     Download and install ${BINARY_NAME}
  uninstall   Remove ${BINARY_NAME}

Environment variables:
  DRAWIO_TOOLS_INSTALL_PATH  Installation directory (default: ${DEFAULT_INSTALL_PATH})
  DRAWIO_TOOLS_VERSION       Version to install (default: latest)

Examples:
  # Install latest version
  curl -sSLf https://raw.githubusercontent.com/${REPO}/main/scripts/setup.sh | sh -s install

  # Install specific version
  DRAWIO_TOOLS_VERSION=v0.1.0 sh setup.sh install

  # Install to custom path
  DRAWIO_TOOLS_INSTALL_PATH=\$HOME/.local/bin sh setup.sh install

  # Uninstall
  sh setup.sh uninstall
USAGE
}

main() {
    case "${1:-}" in
        install)   cmd_install ;;
        uninstall) cmd_uninstall ;;
        *)         usage; exit 1 ;;
    esac
}

main "$@"

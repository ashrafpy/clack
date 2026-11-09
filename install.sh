#!/bin/sh
set -e

# Setup colors if connected to a terminal
if [ -t 1 ] && [ "${TERM:-}" != "dumb" ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    BOLD=''
    RESET=''
fi

info() {
    printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"
}

success() {
    printf "${GREEN}✔${RESET} %s\n" "$1"
}

warn() {
    printf "${YELLOW}⚠${RESET} %s\n" "$1"
}

error() {
    printf "${RED}✘ Error:${RESET} %s\n" "$1" >&2
}

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                ;;
            *)
                error "Unsupported Linux architecture: $ARCH"
                exit 1
                ;;
        esac
        BINARY_NAME="clack"
        ARCHIVE_EXT="tar.gz"
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64|aarch64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                error "Unsupported macOS architecture: $ARCH"
                exit 1
                ;;
        esac
        BINARY_NAME="clack"
        ARCHIVE_EXT="tar.gz"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-pc-windows-msvc"
                ;;
            *)
                error "Unsupported Windows architecture: $ARCH"
                exit 1
                ;;
        esac
        BINARY_NAME="clack.exe"
        ARCHIVE_EXT="zip"
        ;;
    *)
        error "Unsupported operating system: $OS. Clack installer currently supports Linux, macOS, and Windows (Git Bash/MSYS2/Cygwin)."
        exit 1
        ;;
esac

RELEASE_URL="https://github.com/ashrafpy/clack/releases/latest/download/clack-${TARGET}.${ARCHIVE_EXT}"

download() {
    url="$1"
    output="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$output" "$url"
    else
        error "Neither curl nor wget was found. Please install one of them to download Clack."
        exit 1
    fi
}

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'clack-install')"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

ARCHIVE_FILE="$TMP_DIR/clack.${ARCHIVE_EXT}"
info "Downloading Clack for ${TARGET}..."
download "$RELEASE_URL" "$ARCHIVE_FILE"

info "Extracting archive..."
if [ "$ARCHIVE_EXT" = "zip" ]; then
    if command -v unzip >/dev/null 2>&1; then
        unzip -q "$ARCHIVE_FILE" -d "$TMP_DIR"
    elif command -v tar >/dev/null 2>&1 && tar -tf "$ARCHIVE_FILE" >/dev/null 2>&1; then
        tar -xf "$ARCHIVE_FILE" -C "$TMP_DIR"
    elif command -v powershell.exe >/dev/null 2>&1; then
        powershell.exe -NoProfile -Command "Expand-Archive -Path '$ARCHIVE_FILE' -DestinationPath '$TMP_DIR' -Force"
    else
        error "No zip extraction tool found (unzip, tar, or powershell.exe required)."
        exit 1
    fi
else
    tar -xzf "$ARCHIVE_FILE" -C "$TMP_DIR"
fi

if [ -f "$TMP_DIR/$BINARY_NAME" ]; then
    SRC_BINARY="$TMP_DIR/$BINARY_NAME"
else
    SRC_BINARY="$(find "$TMP_DIR" -type f -name "$BINARY_NAME" | head -n 1)"
fi

if [ -z "$SRC_BINARY" ] || [ ! -f "$SRC_BINARY" ]; then
    error "Failed to locate $BINARY_NAME in the downloaded archive."
    exit 1
fi

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"
cp "$SRC_BINARY" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
success "Installed $BINARY_NAME to ${INSTALL_DIR}/$BINARY_NAME"

# Verify PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        warn "${INSTALL_DIR} is not in your PATH."
        printf "  To run 'clack' from anywhere, add it to your shell configuration (e.g., ~/.bashrc, ~/.zshrc):\n"
        printf "    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
        ;;
esac

# Linux input group check
if [ "$OS" = "Linux" ]; then
    CURRENT_USER="${USER:-$(id -un)}"
    USER_GROUPS=" $(id -nG "$CURRENT_USER" 2>/dev/null || groups "$CURRENT_USER" 2>/dev/null) "
    case "$USER_GROUPS" in
        *" input "*)
            ;;
        *)
            echo ""
            warn "Your user account is not currently in the 'input' group."
            printf "  Clack requires access to /dev/input for global keyboard event capture.\n"
            printf "  Add your user to the input group by running:\n"
            printf "    ${BOLD}sudo usermod -aG input %s${RESET}\n" "$CURRENT_USER"
            printf "  Then log out and log back in for the group permissions to take effect.\n"
            ;;
    esac
fi

# macOS accessibility notice
if [ "$OS" = "Darwin" ]; then
    echo ""
    info "macOS Setup:"
    printf "  Grant Accessibility permissions when prompted, or manually via:\n"
    printf "    ${BOLD}System Settings → Privacy & Security → Accessibility${RESET}\n"
    printf "  and enable permissions for your terminal or Clack.\n"
fi

echo ""
printf "${GREEN}${BOLD}✨ Clack was successfully installed! ✨${RESET}\n\n"
printf "Quick start:\n"
printf "  ${BOLD}clack${RESET}               # Run with default settings\n"
printf "  ${BOLD}clack --volume 0.5${RESET}  # Set volume to 50%%\n"
printf "  ${BOLD}clack --help${RESET}        # See all available options\n\n"

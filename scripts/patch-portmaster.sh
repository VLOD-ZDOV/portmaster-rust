#!/usr/bin/env bash
#
# Portmaster Patcher / Installer for Linux
# Replaces or upgrades the Portmaster core with the Rust core (pm-core)
# and modern unlocked web UI with Xray proxy integration.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/VLOD-ZDOV/portmaster-rust/main/scripts/patch-portmaster.sh | sudo bash
#   sudo ./scripts/patch-portmaster.sh [--core rs|go] [--tag latest|<version>] [--restore]
#
set -euo pipefail

REPO="VLOD-ZDOV/portmaster-rust"
TARGET_CORE="rs"
TAG="latest"
ACTION="patch"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --core)
      TARGET_CORE="$2"
      shift 2
      ;;
    --tag)
      TAG="$2"
      shift 2
      ;;
    --restore)
      ACTION="restore"
      shift
      ;;
    --help|-h)
      echo "Portmaster Patcher (Linux)"
      echo "Usage: $0 [options]"
      echo "  --core rs|go       Choose core implementation (default: rs)"
      echo "  --tag <tag>        GitHub release tag or 'latest' (default: latest)"
      echo "  --restore          Restore previous core from backup"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if [[ ${EUID} -ne 0 ]]; then
  echo "[!] Error: Root privileges required. Run with sudo." >&2
  exit 1
fi

ARCH=$(uname -m)
if [[ "${ARCH}" != "x86_64" && "${ARCH}" != "amd64" ]]; then
  echo "[!] Unsupported architecture: ${ARCH}. Only x86_64 / amd64 is supported." >&2
  exit 1
fi

# Detect install directory
INSTALL_DIR="/usr/lib/portmaster"
if [[ ! -d "${INSTALL_DIR}" && -d "/opt/safing/portmaster" ]]; then
  INSTALL_DIR="/opt/safing/portmaster"
fi
DATA_DIR="/var/lib/portmaster"
BACKUP_DIR="${DATA_DIR}/core-backups"

mkdir -p "${INSTALL_DIR}" "${DATA_DIR}" "${BACKUP_DIR}"

if [[ "${ACTION}" == "restore" ]]; then
  echo "[*] Restoring from backup..."
  LATEST_BACKUP=$(ls -1td "${BACKUP_DIR}"/*/ 2>/dev/null | head -n1 || true)
  if [[ -z "${LATEST_BACKUP}" || ! -d "${LATEST_BACKUP}" ]]; then
    echo "[!] No backup found in ${BACKUP_DIR}" >&2
    exit 1
  fi
  echo "[*] Restoring from ${LATEST_BACKUP}..."
  if systemctl is-active --quiet portmaster 2>/dev/null; then
    systemctl stop portmaster || true
  fi
  if [[ -f "${LATEST_BACKUP}/portmaster-core" ]]; then
    cp -af "${LATEST_BACKUP}/portmaster-core" "${INSTALL_DIR}/portmaster-core"
  fi
  if [[ -f "${LATEST_BACKUP}/portmaster" ]]; then
    cp -af "${LATEST_BACKUP}/portmaster" "${INSTALL_DIR}/portmaster"
    chmod 755 "${INSTALL_DIR}/portmaster"
  fi
  if [[ -f "${LATEST_BACKUP}/config.json" ]]; then
    cp -af "${LATEST_BACKUP}/config.json" "${DATA_DIR}/config.json"
  fi
  if [[ -d "${LATEST_BACKUP}/ui" ]]; then
    mkdir -p "${DATA_DIR}/ui/modules/portmaster"
    cp -af "${LATEST_BACKUP}/ui/." "${DATA_DIR}/ui/modules/portmaster/" 2>/dev/null || true
  fi
  rm -f /etc/systemd/system/portmaster.service.d/50-rust-core.conf || true
  systemctl daemon-reload || true
  if ! systemctl start portmaster; then
    echo "[!] Error: failed to start portmaster service after restore." >&2
    exit 1
  fi
  echo "[+] Restore completed successfully."
  exit 0
fi

echo "=========================================================="
echo " Portmaster Upgrade & Patcher (Core: ${TARGET_CORE}, Release: ${TAG})"
echo "=========================================================="

download_file() {
  local url="$1"
  local dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fSL "${url}" -o "${dest}"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "${dest}" "${url}"
  else
    echo "[!] Neither curl nor wget was found." >&2
    return 1
  fi
}

# Resolve download URL
echo "[*] Resolving GitHub release URL for ${REPO} (${TAG})..."
if [[ "${TAG}" == "latest" ]]; then
  BASE_URL="https://github.com/${REPO}/releases/latest/download"
else
  BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"
fi

BIN_NAME="portmaster-core-${TARGET_CORE}"
APP_NAME="portmaster-app-linux-amd64"
DOWNLOAD_BIN_URL="${BASE_URL}/${BIN_NAME}"
DOWNLOAD_APP_URL="${BASE_URL}/${APP_NAME}"
DOWNLOAD_UI_URL="${BASE_URL}/portmaster.zip"
DOWNLOAD_PILOT_URL="${BASE_URL}/pilot-web.zip"

TMP_DIR=$(mktemp -d /tmp/pm-patch.XXXXXX)
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "[*] Downloading ${BIN_NAME}..."
if ! download_file "${DOWNLOAD_BIN_URL}" "${TMP_DIR}/${BIN_NAME}"; then
  echo "[!] Failed to download binary from: ${DOWNLOAD_BIN_URL}" >&2
  exit 1
fi
chmod +x "${TMP_DIR}/${BIN_NAME}"

HAS_APP=0
echo "[*] Downloading tray UI binary (${APP_NAME})..."
if download_file "${DOWNLOAD_APP_URL}" "${TMP_DIR}/${APP_NAME}"; then
  chmod +x "${TMP_DIR}/${APP_NAME}"
  HAS_APP=1
else
  echo "[*] Notice: ${APP_NAME} download failed or not found, continuing without tray binary patch."
fi

echo "[*] Downloading modern UI bundle (portmaster.zip)..."
download_file "${DOWNLOAD_UI_URL}" "${TMP_DIR}/portmaster.zip" || {
  echo "[*] Notice: portmaster.zip download failed or not found, continuing without UI patch."
}

echo "[*] Downloading Rust Pilot UI bundle (pilot-web.zip)..."
download_file "${DOWNLOAD_PILOT_URL}" "${TMP_DIR}/pilot-web.zip" || {
  echo "[*] Notice: pilot-web.zip download failed or not found, continuing."
}

# Stop Portmaster service
SERVICE_WAS_ACTIVE=0
if systemctl is-active --quiet portmaster 2>/dev/null; then
  echo "[*] Stopping portmaster service..."
  systemctl stop portmaster || true
  SERVICE_WAS_ACTIVE=1
fi

# Backup existing binaries, config and UI
STAMP=$(date +%Y%m%d-%H%M%S)
CURRENT_BACKUP="${BACKUP_DIR}/${STAMP}"
mkdir -p "${CURRENT_BACKUP}"

for target_file in portmaster-core portmaster-core-rs portmaster-core-go portmaster; do
  if [[ -f "${INSTALL_DIR}/${target_file}" ]]; then
    cp -af "${INSTALL_DIR}/${target_file}" "${CURRENT_BACKUP}/"
  fi
done
if [[ -f "${DATA_DIR}/config.json" ]]; then
  cp -af "${DATA_DIR}/config.json" "${CURRENT_BACKUP}/"
fi
if [[ -d "${DATA_DIR}/ui/modules/portmaster" ]]; then
  mkdir -p "${CURRENT_BACKUP}/ui"
  cp -af "${DATA_DIR}/ui/modules/portmaster/." "${CURRENT_BACKUP}/ui/" 2>/dev/null || true
fi
echo "[*] Existing binaries, configuration and UI backed up to: ${CURRENT_BACKUP}"

# Install new binary
TARGET_BIN_PATH="${INSTALL_DIR}/${BIN_NAME}"
cp -af "${TMP_DIR}/${BIN_NAME}" "${TARGET_BIN_PATH}"
chmod 755 "${TARGET_BIN_PATH}"

# Ensure default execution points to chosen core
if [[ "${TARGET_CORE}" == "rs" ]]; then
  cp -af "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/portmaster-core"
  # Also setup systemd drop-in override if service exists
  DROPIN_DIR="/etc/systemd/system/portmaster.service.d"
  if [[ -f "/usr/lib/systemd/system/portmaster.service" || -f "/lib/systemd/system/portmaster.service" || -f "/etc/systemd/system/portmaster.service" ]]; then
    mkdir -p "${DROPIN_DIR}"
    cat > "${DROPIN_DIR}/50-rust-core.conf" <<EOF
[Service]
ExecStart=
ExecStart=${TARGET_BIN_PATH}
RestartForceExitStatus=23
RestartSec=1
EOF
    systemctl daemon-reload || true
  fi
else
  cp -af "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/portmaster-core"
  rm -f /etc/systemd/system/portmaster.service.d/50-rust-core.conf || true
  systemctl daemon-reload || true
fi

if [[ ${HAS_APP} -eq 1 ]]; then
  echo "[*] Updating tray UI binary..."
  cp -af "${TMP_DIR}/${APP_NAME}" "${INSTALL_DIR}/portmaster"
  chmod 755 "${INSTALL_DIR}/portmaster"
  if [[ -f "/usr/bin/portmaster" && ! -L "/usr/bin/portmaster" ]]; then
    mv -f "/usr/bin/portmaster" "${CURRENT_BACKUP}/usr-bin-portmaster" || true
  fi
  ln -sf "${INSTALL_DIR}/portmaster" "/usr/bin/portmaster" || true
  echo "[*] Notice: restart the Portmaster tray UI if it is currently running."
fi

# Unpack UI bundle if present
if [[ -f "${TMP_DIR}/portmaster.zip" ]] && command -v unzip >/dev/null 2>&1; then
  echo "[*] Updating Web UI..."
  UI_DEST="${DATA_DIR}/ui/modules/portmaster"
  mkdir -p "${UI_DEST}"
  if ! unzip -q -o "${TMP_DIR}/portmaster.zip" -d "${UI_DEST}"; then
    echo "[!] Warning: failed to unpack portmaster.zip to ${UI_DEST}" >&2
  fi

  # Also update updates path if present
  UPDATES_UI="${INSTALL_DIR}/updates/linux_amd64/ui/modules/portmaster"
  if [[ -d "${UPDATES_UI}" ]]; then
    if ! unzip -q -o "${TMP_DIR}/portmaster.zip" -d "${UPDATES_UI}"; then
      echo "[!] Warning: failed to unpack portmaster.zip to ${UPDATES_UI}" >&2
    fi
  fi
fi

# Unpack Rust Pilot UI bundle if present
if [[ -f "${TMP_DIR}/pilot-web.zip" ]] && command -v unzip >/dev/null 2>&1; then
  echo "[*] Updating Rust Pilot UI..."
  PILOT_DEST="${DATA_DIR}/pilot-web"
  mkdir -p "${PILOT_DEST}"
  if ! unzip -q -o "${TMP_DIR}/pilot-web.zip" -d "${PILOT_DEST}"; then
    echo "[!] Warning: failed to unpack pilot-web.zip to ${PILOT_DEST}" >&2
  fi
fi

# Sanitize configuration only for the known old default. Do not silently
# rewrite a custom listen address or turn off the user's internet blocking.
if [[ -f "${DATA_DIR}/config.json" ]]; then
  if grep -q '"listenAddress": "127.0.0.1:8171"' "${DATA_DIR}/config.json"; then
    echo "[*] Replacing obsolete default listen address 127.0.0.1:8171 with 127.0.0.1:817..."
    cp -a "${DATA_DIR}/config.json" "${DATA_DIR}/config.json.bak"
    sed -i 's/"listenAddress": "127.0.0.1:8171"/"listenAddress": "127.0.0.1:817"/g' "${DATA_DIR}/config.json"
  fi
  if grep -q '"blockInternet": true' "${DATA_DIR}/config.json"; then
    echo "[!] Notice: keeping your existing \"blockInternet\": true setting." >&2
  fi
fi

# Restart Portmaster
if [[ ${SERVICE_WAS_ACTIVE} -eq 1 ]] || systemctl is-enabled --quiet portmaster 2>/dev/null; then
  echo "[*] Starting portmaster service..."
  if ! systemctl start portmaster; then
    echo "[!] Error: failed to start portmaster service. Check: journalctl -u portmaster -n 50" >&2
    exit 1
  fi
fi

echo ""
echo "=========================================================="
echo " [✓] Portmaster patched successfully!"
echo "     Active binary: ${TARGET_BIN_PATH}"
echo "     Backup saved:  ${CURRENT_BACKUP}"
echo "=========================================================="

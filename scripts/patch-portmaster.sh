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
  if [[ -f "${LATEST_BACKUP}/portmaster-qt" ]]; then
    cp -af "${LATEST_BACKUP}/portmaster-qt" "${INSTALL_DIR}/portmaster-qt"
    chmod 755 "${INSTALL_DIR}/portmaster-qt"
  fi
  if [[ -f "${LATEST_BACKUP}/portmaster.zip" ]]; then
    cp -af "${LATEST_BACKUP}/portmaster.zip" "${INSTALL_DIR}/portmaster.zip"
    cp -af "${LATEST_BACKUP}/portmaster.zip" "${DATA_DIR}/portmaster.zip"
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
    curl -fSL --retry 3 --retry-delay 2 "${url}" -o "${dest}"
  elif command -v wget >/dev/null 2>&1; then
    wget --tries=3 -qO "${dest}" "${url}"
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
DOWNLOAD_ASSETS_URL="${BASE_URL}/assets.zip"
DOWNLOAD_PILOT_URL="${BASE_URL}/pilot-web.zip"
DOWNLOAD_RUST_UI_URL="${BASE_URL}/rust-ui.zip"
DOWNLOAD_QT_URL="${BASE_URL}/portmaster-qt-linux-amd64"

TMP_DIR=$(mktemp -d /tmp/pm-patch.XXXXXX)
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "[*] Downloading ${BIN_NAME}..."
if ! download_file "${DOWNLOAD_BIN_URL}" "${TMP_DIR}/${BIN_NAME}"; then
  echo "[!] Failed to download binary from: ${DOWNLOAD_BIN_URL}" >&2
  exit 1
fi
chmod +x "${TMP_DIR}/${BIN_NAME}"

echo "[*] Downloading tray UI binary (${APP_NAME})..."
if ! download_file "${DOWNLOAD_APP_URL}" "${TMP_DIR}/${APP_NAME}"; then
  echo "[!] Failed to download tray UI binary from: ${DOWNLOAD_APP_URL}" >&2
  exit 1
fi
chmod +x "${TMP_DIR}/${APP_NAME}"

echo "[*] Downloading modern UI bundle (portmaster.zip)..."
if ! download_file "${DOWNLOAD_UI_URL}" "${TMP_DIR}/portmaster.zip"; then
  echo "[!] Failed to download UI bundle from: ${DOWNLOAD_UI_URL}" >&2
  exit 1
fi

echo "[*] Downloading UI static assets (assets.zip)..."
download_file "${DOWNLOAD_ASSETS_URL}" "${TMP_DIR}/assets.zip" || {
  echo "[*] Notice: assets.zip not found in release ${TAG}, keeping existing if any."
}

echo "[*] Downloading Rust WASM UI bundle (rust-ui.zip)..."
download_file "${DOWNLOAD_RUST_UI_URL}" "${TMP_DIR}/rust-ui.zip" || {
  echo "[*] Notice: rust-ui.zip not found in release ${TAG}, continuing."
}

echo "[*] Downloading Rust Pilot UI bundle (pilot-web.zip)..."
download_file "${DOWNLOAD_PILOT_URL}" "${TMP_DIR}/pilot-web.zip" || {
  echo "[*] Notice: pilot-web.zip download failed or not found, continuing."
}

echo "[*] Downloading Qt 6 Native UI binary (portmaster-qt-linux-amd64)..."
if download_file "${DOWNLOAD_QT_URL}" "${TMP_DIR}/portmaster-qt-linux-amd64"; then
  chmod +x "${TMP_DIR}/portmaster-qt-linux-amd64"
else
  echo "[*] Notice: portmaster-qt-linux-amd64 not found in release ${TAG}, continuing."
fi

# Capture any running tray UI processes (exe + desktop session env) before stopping them
TRAY_PID_LIST=()
TRAY_EXE_LIST=()
TRAY_USER=""
TRAY_ENV_VARS=()
TRAY_LAUNCH_EXE=""

for exe_link in /proc/[0-9]*/exe; do
  [[ -L "${exe_link}" ]] || continue
  exe_target=$(readlink -f "${exe_link}" 2>/dev/null || true)
  exe_target="${exe_target% (deleted)}"
  exe_base=$(basename "${exe_target}")
  if [[ "${exe_base}" == "portmaster" || "${exe_base}" == "portmaster-app" || "${exe_base}" == "portmaster-qt" ]]; then
    pid="${exe_link#/proc/}"
    pid="${pid%/exe}"
    TRAY_PID_LIST+=("${pid}")
    TRAY_EXE_LIST+=("${exe_target}")
    if [[ -z "${TRAY_USER}" ]]; then
      TRAY_USER=$(ps -p "${pid}" -o user= 2>/dev/null | xargs || true)
      TRAY_LAUNCH_EXE="${exe_target}"
      if [[ -r "/proc/${pid}/environ" ]]; then
        while IFS= read -r -d '' env_entry; do
          case "${env_entry}" in
            DISPLAY=*|WAYLAND_DISPLAY=*|DBUS_SESSION_BUS_ADDRESS=*|XDG_RUNTIME_DIR=*|XAUTHORITY=*)
              TRAY_ENV_VARS+=("${env_entry}")
              ;;
          esac
        done < "/proc/${pid}/environ" || true
      fi
    fi
  fi
done

if [[ ${#TRAY_PID_LIST[@]} -gt 0 ]]; then
  echo "[*] Stopping running tray UI process(es): ${TRAY_PID_LIST[*]}..."
  kill "${TRAY_PID_LIST[@]}" 2>/dev/null || true
  for _ in 1 2 3 4 5 6; do
    still_alive=0
    for pid in "${TRAY_PID_LIST[@]}"; do
      if kill -0 "${pid}" 2>/dev/null; then
        still_alive=1
        break
      fi
    done
    [[ ${still_alive} -eq 0 ]] && break
    sleep 0.5
  done
  for pid in "${TRAY_PID_LIST[@]}"; do
    kill -9 "${pid}" 2>/dev/null || true
  done
fi

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

for target_file in portmaster-core portmaster-core-rs portmaster-core-go portmaster portmaster-qt portmaster.zip; do
  if [[ -f "${INSTALL_DIR}/${target_file}" ]]; then
    cp -af "${INSTALL_DIR}/${target_file}" "${CURRENT_BACKUP}/"
  fi
done
if [[ -f "${DATA_DIR}/portmaster.zip" ]]; then
  cp -af "${DATA_DIR}/portmaster.zip" "${CURRENT_BACKUP}/data-portmaster.zip"
fi
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

# Install standalone systemd service & desktop integration if missing
mkdir -p "${DATA_DIR}/log"
if [[ ! -f "/usr/lib/systemd/system/portmaster.service" && ! -f "/lib/systemd/system/portmaster.service" && ! -f "/etc/systemd/system/portmaster.service" ]]; then
  echo "[*] No existing portmaster.service found; installing standalone systemd service..."
  mkdir -p /usr/lib/systemd/system
  cat > /usr/lib/systemd/system/portmaster.service <<EOF
[Unit]
Description=Portmaster by Safing (standalone fork install)
After=systemd-networkd.service
Before=nss-lookup.target network.target shutdown.target
Conflicts=shutdown.target firewalld.service
Wants=nss-lookup.target

[Service]
Type=simple
Restart=on-failure
RestartSec=10
RestartForceExitStatus=23
RestartPreventExitStatus=24
LockPersonality=yes
NoNewPrivileges=yes
PrivateTmp=yes
Environment=LOGLEVEL=info
ProtectSystem=true
ReadWritePaths=${INSTALL_DIR}
RestrictAddressFamilies=AF_UNIX AF_NETLINK AF_INET AF_INET6
RestrictNamespaces=yes
ProtectHome=read-only
ProtectKernelTunables=yes
ProtectKernelLogs=yes
ProtectControlGroups=yes
PrivateDevices=yes
AmbientCapabilities=cap_chown cap_kill cap_net_admin cap_net_bind_service cap_net_broadcast cap_net_raw cap_sys_module cap_sys_ptrace cap_dac_override cap_fowner cap_fsetid cap_sys_resource cap_bpf cap_perfmon
CapabilityBoundingSet=cap_chown cap_kill cap_net_admin cap_net_bind_service cap_net_broadcast cap_net_raw cap_sys_module cap_sys_ptrace cap_dac_override cap_fowner cap_fsetid cap_sys_resource cap_bpf cap_perfmon
StateDirectory=portmaster
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_DIR}/portmaster-core --log=\${LOGLEVEL} --log-dir=${DATA_DIR}/log
ExecStopPost=-${INSTALL_DIR}/portmaster-core --recover-iptables

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload || true
  systemctl enable portmaster.service || true
fi

if [[ ! -f "/usr/share/applications/Portmaster.desktop" && -d "/usr/share/applications" ]]; then
  cat > /usr/share/applications/Portmaster.desktop <<EOF
[Desktop Entry]
Name=Portmaster
GenericName=Application Firewall & Xray VPN
Exec=/usr/bin/portmaster --with-prompts --with-notifications
Icon=portmaster
StartupWMClass=portmaster
Terminal=false
Type=Application
Categories=System;Network;
EOF
fi
if [[ ! -f "/etc/xdg/autostart/portmaster-autostart.desktop" && -d "/etc/xdg/autostart" ]]; then
  cat > /etc/xdg/autostart/portmaster-autostart.desktop <<EOF
[Desktop Entry]
Name=Portmaster
GenericName=Application Firewall Notifier
Exec=/usr/bin/portmaster --with-prompts --with-notifications --background
Icon=portmaster
Terminal=false
Type=Application
Categories=System;
NoDisplay=true
EOF
fi

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

echo "[*] Updating tray UI binary..."
cp -af "${TMP_DIR}/${APP_NAME}" "${INSTALL_DIR}/portmaster"
chmod 755 "${INSTALL_DIR}/portmaster"
for extra_exe in "${TRAY_EXE_LIST[@]}"; do
  if [[ -n "${extra_exe}" && "${extra_exe}" != "${INSTALL_DIR}/portmaster" && "${extra_exe}" != "/usr/bin/portmaster" ]]; then
    if [[ -f "${extra_exe}" ]]; then
      cp -af "${extra_exe}" "${CURRENT_BACKUP}/$(basename "${extra_exe}").prev" 2>/dev/null || true
    fi
    cp -af "${TMP_DIR}/${APP_NAME}" "${extra_exe}"
    chmod 755 "${extra_exe}"
  fi
done
if [[ -f "/opt/safing/portmaster/portmaster-app" ]]; then
  cp -af "${TMP_DIR}/${APP_NAME}" "/opt/safing/portmaster/portmaster-app"
  chmod 755 "/opt/safing/portmaster/portmaster-app"
fi
if [[ -f "/usr/bin/portmaster" && ! -L "/usr/bin/portmaster" ]]; then
  mv -f "/usr/bin/portmaster" "${CURRENT_BACKUP}/usr-bin-portmaster" || true
fi
ln -sf "${INSTALL_DIR}/portmaster" "/usr/bin/portmaster" || true

# Clear WebKit/Tauri HTTP cache so UI reloads fresh assets immediately
rm -rf /home/*/.cache/io.safing.portmaster /home/*/.cache/portmaster /root/.cache/io.safing.portmaster 2>/dev/null || true

# Update Web UI archive (read directly by pm-core UiArchives) and extracted dirs
echo "[*] Updating Web UI archive (portmaster.zip)..."
cp -af "${TMP_DIR}/portmaster.zip" "${INSTALL_DIR}/portmaster.zip"
chmod 644 "${INSTALL_DIR}/portmaster.zip"
cp -af "${TMP_DIR}/portmaster.zip" "${DATA_DIR}/portmaster.zip"
chmod 644 "${DATA_DIR}/portmaster.zip"

if [[ -f "${TMP_DIR}/assets.zip" ]]; then
  echo "[*] Installing UI assets archive (assets.zip)..."
  cp -af "${TMP_DIR}/assets.zip" "${INSTALL_DIR}/assets.zip"
  chmod 644 "${INSTALL_DIR}/assets.zip"
  cp -af "${TMP_DIR}/assets.zip" "${DATA_DIR}/assets.zip"
  chmod 644 "${DATA_DIR}/assets.zip"
fi

if command -v unzip >/dev/null 2>&1; then
  UI_DEST="${DATA_DIR}/ui/modules/portmaster"
  rm -rf "${UI_DEST}"
  mkdir -p "${UI_DEST}"
  unzip -q -o "${TMP_DIR}/portmaster.zip" -d "${UI_DEST}" || {
    echo "[!] Warning: failed to unpack portmaster.zip to ${UI_DEST}" >&2
  }

  UPDATES_UI="${INSTALL_DIR}/updates/linux_amd64/ui/modules/portmaster"
  if [[ -d "${UPDATES_UI}" ]]; then
    cp -af "${TMP_DIR}/portmaster.zip" "${UPDATES_UI}/portmaster.zip"
    unzip -q -o "${TMP_DIR}/portmaster.zip" -d "${UPDATES_UI}" || true
  fi
fi

# Install Qt 6 Native UI if present
if [[ -f "${TMP_DIR}/portmaster-qt-linux-amd64" ]]; then
  echo "[*] Installing Qt 6 Native UI (portmaster-qt)..."
  cp -af "${TMP_DIR}/portmaster-qt-linux-amd64" "${INSTALL_DIR}/portmaster-qt"
  chmod 755 "${INSTALL_DIR}/portmaster-qt"
  ln -sf "${INSTALL_DIR}/portmaster-qt" "/usr/bin/portmaster-qt" || true
  if command -v ldd >/dev/null 2>&1 && ldd "${INSTALL_DIR}/portmaster-qt" 2>/dev/null | grep -q "not found"; then
    echo "[!] Notice: portmaster-qt needs Qt 6 runtime libraries (qt6-declarative, qt6-websockets / qml6-module-qtquick-controls)."
  fi
fi

# Unpack Rust WASM UI bundle if present
if [[ -f "${TMP_DIR}/rust-ui.zip" ]] && command -v unzip >/dev/null 2>&1; then
  echo "[*] Updating Rust WASM UI..."
  RUST_UI_DEST="${DATA_DIR}/rust-ui"
  rm -rf "${RUST_UI_DEST}"
  mkdir -p "${RUST_UI_DEST}"
  if ! unzip -q -o "${TMP_DIR}/rust-ui.zip" -d "${RUST_UI_DEST}"; then
    echo "[!] Warning: failed to unpack rust-ui.zip to ${RUST_UI_DEST}" >&2
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

# Relaunch tray UI under original desktop user session if it was running
if [[ -n "${TRAY_USER}" && -n "${TRAY_LAUNCH_EXE}" && ${#TRAY_ENV_VARS[@]} -gt 0 ]] && command -v runuser >/dev/null 2>&1; then
  echo "[*] Relaunching tray UI (${TRAY_LAUNCH_EXE}) as ${TRAY_USER}..."
  runuser -u "${TRAY_USER}" -- env "${TRAY_ENV_VARS[@]}" setsid -f "${TRAY_LAUNCH_EXE}" --background --with-prompts --with-notifications >/dev/null 2>&1 || true
fi

TRAY_VERIFY="unknown"
if command -v strings >/dev/null 2>&1; then
  if strings "${INSTALL_DIR}/portmaster" 2>/dev/null | grep -q "Xray VPN"; then
    TRAY_VERIFY="verified Xray VPN tray (no Enable SPN)"
  else
    TRAY_VERIFY="warning: Xray VPN string not found in ${INSTALL_DIR}/portmaster"
  fi
fi

echo ""
echo "=========================================================="
echo " [✓] Portmaster patched successfully!"
echo "     Core binary: ${TARGET_BIN_PATH}"
echo "     Tray binary: ${INSTALL_DIR}/portmaster (${TRAY_VERIFY})"
echo "     UI archive:  ${INSTALL_DIR}/portmaster.zip"
echo "     Backup saved: ${CURRENT_BACKUP}"
echo "=========================================================="

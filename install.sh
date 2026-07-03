#!/bin/bash
set -euo pipefail
# only for agent download
# root is required
if [ "$EUID" -ne 0 ]; then
  echo "This script must be run as root"
  exit 1
fi

URL="<download link>.gz"
FILE="<downloaded file>.gz"
echo "Downloading..."
cd /usr/local/bin
curl -L "$URL" -o "$FILE"

echo "Extracting..."
tar -xzf "$FILE"
[ -f "$FILE" ] && rm "$FILE"
# need to check the hash
echo "Setting Permissions" 
chmod +x DeployBridge
#need to write

# systemd part
SERVICE_NAME="bridge"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=DeployBridge service
After=network-online.target
Wants=network-online.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
Type=exec

# --- what runs ---
ExecStart=/usr/local/bin/DeployBridge
WorkingDirectory=/usr/local/bin/

# --- identity ---
User=root
Group=root

# --- restart behaviour ---
Restart=on-failure
RestartSec=5s

# --- logging: send all stdout/stderr to a file ---
LogsDirectory=deploybridge
StandardOutput=append:/var/log/deploybridge/agent.log
StandardError=append:/var/log/deploybridge/agent.log
NoNewPrivileges=true
ProtectSystem=full
ProtectHome=read-only
PrivateTmp=true
RestrictSUIDSGID=true
ProtectKernelModules=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
LockPersonality=true
MemoryDenyWriteExecute=true
RestrictRealtime=true
SystemCallArchitectures=native

[Install]
WantedBy=multi-user.target
EOF

# restart the systemd 
echo "Reloading systemd..."
systemctl daemon-reload
echo "Enabling service..."
systemctl enable "$SERVICE_NAME"

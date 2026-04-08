#!/bin/sh
set -e

CONFIG_DIR=/etc/turn-server
CONFIG_FILE="$CONFIG_DIR/turnserver.conf"
TEMPLATE_FILE="/opt/turn-server/turnserver.conf.template"

# If config file does not exist, copy from template
if [ ! -f "$CONFIG_FILE" ]; then
    echo "No turnserver.conf found, copying from template..."
    cp "$TEMPLATE_FILE" "$CONFIG_FILE"
    echo "Configuration initialized at $CONFIG_FILE"
else
    echo "Using existing turnserver.conf at $CONFIG_FILE"
fi

# Execute turn-server with config path
exec /usr/local/bin/turn-server --config "$CONFIG_FILE"

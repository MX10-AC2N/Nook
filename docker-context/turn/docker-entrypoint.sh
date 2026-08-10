#!/bin/sh
set -e

CONFIG_DIR=/etc/turn-server
CONFIG_FILE="$CONFIG_DIR/config.toml"
TEMPLATE_FILE="/opt/turn-server/turnserver.conf.template"

# If config file does not exist, copy from template
if [ ! -f "$CONFIG_FILE" ]; then
    echo "No config.toml found, copying from template..."
    cp "$TEMPLATE_FILE" "$CONFIG_FILE"
    
    # Replace placeholder with actual TURN_SECRET from environment
    if [ -n "$TURN_SECRET" ]; then
        sed -i "s|\\${TURN_SECRET}|$TURN_SECRET|g" "$CONFIG_FILE"
        echo "TURN_SECRET configured"
    else
        echo "ERROR: TURN_SECRET must be set!"
        exit 1
    fi
    
    echo "Configuration initialized at $CONFIG_FILE"
else
    echo "Using existing config.toml at $CONFIG_FILE"
fi

# Execute turn-server with config path (drop to nook user if root)
if [ "$(id -u)" = "0" ]; then
    chown -R nook:nook /etc/turn-server 2>/dev/null || true
    exec su-exec nook /usr/local/bin/turn-server --config "$CONFIG_FILE"
else
    exec /usr/local/bin/turn-server --config "$CONFIG_FILE"
fi

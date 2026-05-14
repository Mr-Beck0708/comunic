#!/bin/bash

echo "========================================="
echo "Starting Client Device"
echo "========================================="

# Check if device ID and name are provided
if [ $# -ne 2 ]; then
    echo "Usage: $0 <device_id> <device_name>"
    echo "Example: $0 laptop01 \"John's Laptop\""
    exit 1
fi

DEVICE_ID=$1
DEVICE_NAME=$2
DRONE_IP="192.168.1.100"  # Change to your drone's IP
DRONE_PORT="8888"

# Run the device
cargo run -- device --id "$DEVICE_ID" --name "$DEVICE_NAME" --drone-addr "${DRONE_IP}:${DRONE_PORT}"

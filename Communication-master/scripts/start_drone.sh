#!/bin/bash

echo "========================================="
echo "Starting Central Drone (Raspberry Pi)"
echo "========================================="

# Set the IP address (change to your Pi's IP)
DRONE_IP="0.0.0.0"
DRONE_PORT="8888"

# Run the drone
cargo run -- drone --addr "${DRONE_IP}:${DRONE_PORT}"

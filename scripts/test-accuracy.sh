#!/bin/bash
# ============================================================
# InSysIn — Accuracy Verification Tests
# Compares Tauri backend data against system ground truth
#
# Usage:
#   ./scripts/test-accuracy.sh              # Node.js WS test
#   ./scripts/test-accuracy.sh --system     # system commands only (reference)
#   ./scripts/test-accuracy.sh --help       # this help
# ============================================================
set -euo pipefail

cd "$(dirname "$0")/.."

if [ "${1:-}" = "--help" ]; then
    grep '^#' "$0" | grep -v '^#!/bin/bash' | sed 's/^# //'
    exit 0
fi

if [ "${1:-}" = "--system" ]; then
    echo "=== Reference values from system commands ==="
    echo ""
    echo "--- Hardware ---"
    echo "total_memory=$(sysctl -n hw.memsize)"
    echo "cpu_cores=$(sysctl -n hw.ncpu)"
    echo "cpu_brand=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Apple M1')"
    echo "hostname=$(hostname)"
    echo "os_name=$(uname -s)"
    echo "os_version=$(sw_vers -productVersion)"
    echo "kernel=$(uname -r)"
    BOOTTIME=$(sysctl -n kern.boottime | awk -F'[= ,]' '{print $6}')
    NOW=$(date +%s)
    echo "uptime_secs=$((NOW - BOOTTIME))"
    echo ""
    echo "--- Memory ---"
    sysctl -n hw.memsize | awk '{printf "total_gb=%.2f\n", $1/1073741824}'
    vm_stat | head -5
    sysctl -n vm.swapusage 2>/dev/null || echo "swap: none"
    echo ""
    echo "--- Disk (root) ---"
    df -h / | tail -1
    echo ""
    echo "--- Network ---"
    echo "gateway=$(netstat -rn -f inet 2>/dev/null | grep '^default' | awk '{print $2}')"
    echo "local_ip=$(ifconfig en0 2>/dev/null | grep 'inet ' | awk '{print $2}')"
    echo ""
    echo "--- DNS ---"
    scutil --dns 2>/dev/null | grep 'nameserver\[' | awk -F': ' '{print $2}' | sort -u
    echo ""
    echo "--- ARP Table ---"
    arp -a 2>/dev/null | grep -v incomplete | grep -v '224.0.0' | grep -v 'ff:ff:ff:ff:ff:ff'
    echo ""
    echo "--- WiFi SSID ---"
    /usr/sbin/ipconfig getsummary en0 2>/dev/null | grep "SSID : " | sed 's/^[[:space:]]*SSID : //' || echo "(not found)"
    /usr/sbin/networksetup -getairportnetwork en0 2>/dev/null || echo "(networksetup: not available)"
    echo ""
    echo "--- Processes (top 15 by RSS) ---"
    ps aux --sort=-rss 2>/dev/null | head -16 || ps -eo pid,rss,comm -r 2>/dev/null | head -16
    exit 0
fi

# --- Node.js test runner ---
echo "============================================"
echo " InSysIn — Accuracy Verification Suite"
echo "============================================"
echo ""

if ! command -v node &>/dev/null; then
    echo "ERROR: node not found. Install Node.js first."
    exit 1
fi

if [ ! -f node_modules/ws/index.js ]; then
    echo "Installing ws dependency..."
    npm install ws --save-dev
fi

echo "Connecting to WS bridge at ws://localhost:9233 ..."
echo "Make sure the Tauri app is running (npm run tauri dev)"
echo ""

exec node scripts/test-accuracy.js

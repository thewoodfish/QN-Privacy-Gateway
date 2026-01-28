#!/bin/bash

# QN Privacy Gateway Demo Script
# This script demonstrates various RPC calls through the gateway

set -e

GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
COLORS=true

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}    ${MAGENTA}QN Privacy Gateway - Demo Script${NC}              ${CYAN}║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_step() {
    echo -e "${YELLOW}▶${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

check_gateway() {
    print_step "Checking gateway health..."
    if curl -s -f "${GATEWAY_URL}/health" > /dev/null 2>&1; then
        print_success "Gateway is running at ${GATEWAY_URL}"
    else
        print_error "Gateway is not running at ${GATEWAY_URL}"
        print_info "Start the gateway with: cargo run --release"
        exit 1
    fi
    echo ""
}

make_rpc_call() {
    local method=$1
    local params=$2
    local description=$3
    
    print_step "${description}"
    
    local payload=$(cat <<JSON_PAYLOAD
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "${method}",
    "params": ${params}
}
JSON_PAYLOAD
)
    
    echo -e "${CYAN}Request:${NC} ${method}"
    
    response=$(curl -s -X POST "${GATEWAY_URL}/" \
        -H "Content-Type: application/json" \
        -d "${payload}")
    
    if echo "${response}" | jq -e '.result' > /dev/null 2>&1; then
        print_success "Success"
        echo -e "${CYAN}Response:${NC}"
        echo "${response}" | jq '.'
    elif echo "${response}" | jq -e '.error' > /dev/null 2>&1; then
        print_error "RPC Error"
        echo "${response}" | jq '.'
    else
        print_error "Invalid response"
        echo "${response}"
    fi
    echo ""
}

show_metrics() {
    print_step "Fetching gateway metrics..."
    
    metrics=$(curl -s "${GATEWAY_URL}/metrics")
    
    print_success "Current Metrics:"
    echo "${metrics}" | jq '.'
    echo ""
}

print_header

check_gateway

print_info "Gateway URL: ${GATEWAY_URL}"
print_info "Dashboard: ${GATEWAY_URL}/dashboard"
print_info "Metrics: ${GATEWAY_URL}/metrics"
echo ""

# Test 1: Get latest slot
make_rpc_call "getSlot" "[]" "Test 1: Get current slot"

# Test 2: Get latest blockhash (cached in balanced mode)
make_rpc_call "getLatestBlockhash" '[]' "Test 2: Get latest blockhash (cacheable)"

# Test 3: Make the same call again to test caching
sleep 1
make_rpc_call "getLatestBlockhash" '[]' "Test 3: Repeat blockhash request (should hit cache)"

# Test 4: Get version
make_rpc_call "getVersion" '[]' "Test 4: Get Solana version"

# Test 5: Get supply
make_rpc_call "getSupply" '[]' "Test 5: Get supply"

# Show final metrics
show_metrics

print_success "Demo completed!"
print_info "Open the dashboard to see live logs: ${GATEWAY_URL}/dashboard"
echo ""

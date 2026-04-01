#!/bin/bash
# Smoke tests for IOU-Modern
# Usage: ./smoke-test.sh [dev|staging|prod]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVIRONMENT="${1:-dev}"
NAMESPACE="iou-${ENVIRONMENT}"
BASE_URL=""

# Determine base URL based on environment
case "${ENVIRONMENT}" in
    dev)
        BASE_URL="https://iou-dev.organization.nl"
        ;;
    staging)
        BASE_URL="https://iou-staging.organization.nl"
        ;;
    prod)
        BASE_URL="https://iou-api.organization.nl"
        ;;
    *)
        echo "Invalid environment"
        exit 1
        ;;
esac

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    exit 1
}

# Test 1: Health endpoint
test_health() {
    echo "Testing health endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/health")
    if [[ "${response}" == "200" ]]; then
        log_info "Health endpoint responding"
    else
        log_error "Health endpoint failed (HTTP ${response})"
    fi
}

# Test 2: Ready endpoint
test_ready() {
    echo "Testing ready endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/ready")
    if [[ "${response}" == "200" ]]; then
        log_info "Ready endpoint responding"
    else
        log_error "Ready endpoint failed (HTTP ${response})"
    fi
}

# Test 3: Metrics endpoint (Prometheus)
test_metrics() {
    echo "Testing metrics endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/metrics")
    if [[ "${response}" == "200" ]]; then
        log_info "Metrics endpoint responding"
    else
        log_error "Metrics endpoint failed (HTTP ${response})"
    fi
}

# Test 4: API version endpoint
test_version() {
    echo "Testing API version endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/v1/version")
    if [[ "${response}" == "200" ]]; then
        log_info "Version endpoint responding"
    else
        log_error "Version endpoint failed (HTTP ${response})"
    fi
}

# Test 5: Database connectivity
test_database() {
    echo "Testing database connectivity..."
    # This would require proper authentication, skipping for smoke test
    log_info "Database connectivity check skipped (requires auth)"
}

# Test 6: Haven+ NLX endpoint (if configured)
test_nlx() {
    echo "Testing Haven+ NLX endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/.nlx/outway")
    # NLX endpoint might not be exposed in all environments
    if [[ "${response}" == "200" ]] || [[ "${response}" == "404" ]]; then
        log_info "NLX endpoint accessible (or not configured)"
    else
        log_error "NLX endpoint unexpected response (HTTP ${response})"
    fi
}

# Run all tests
main() {
    echo "==========================================="
    echo "  IOU-Modern Smoke Tests"
    echo "  Environment: ${ENVIRONMENT}"
    echo "  Base URL: ${BASE_URL}"
    echo "==========================================="
    echo ""

    test_health
    test_ready
    test_metrics
    test_version
    test_database
    test_nlx

    echo ""
    echo "==========================================="
    echo "  All smoke tests passed!"
    echo "==========================================="
}

main "$@"

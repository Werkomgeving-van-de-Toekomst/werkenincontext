#!/bin/bash
# Haven+ Stack Deployment Script
# Usage: ./deploy.sh [dev|staging|prod]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVIRONMENT="${1:-dev}"
HELM_RELEASE_NAME="iou-api-${ENVIRONMENT}"
HELM_CHART_DIR="${SCRIPT_DIR}/../helm/iou-api"
NAMESPACE="iou-${ENVIRONMENT}"

# Haven+ colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Validate Haven+ environment
validate_environment() {
    case "${ENVIRONMENT}" in
        dev|staging|prod)
            log_info "Deploying to environment: ${ENVIRONMENT}"
            ;;
        *)
            log_error "Invalid environment. Use: dev, staging, or prod"
            ;;
    esac
}

# Check Haven+ prerequisites
check_prerequisites() {
    log_info "Checking Haven+ prerequisites..."

    command -v helm >/dev/null 2>&1 || log_error "helm not found. Install Helm 3.x"
    command -v kubectl >/dev/null 2>&1 || log_error "kubectl not found. Install kubectl"
    command -v kustomize >/dev/null 2>&1 || log_error "kustomize not found. Install kustomize"

    # Check cluster connection
    if ! kubectl cluster-info >/dev/null 2>&1; then
        log_error "Cannot connect to Kubernetes cluster"
    fi

    log_info "Prerequisites check passed"
}

# Create namespace if it doesn't exist
create_namespace() {
    log_info "Creating namespace: ${NAMESPACE}"
    kubectl create namespace "${NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -

    # Apply Haven+ namespace labels
    kubectl label namespace "${NAMESPACE}" \
        haven.common-ground.nl/environment="${ENVIRONMENT}" \
        haven.common-ground.nl/organization="organization" \
        --dry-run=client -o yaml | kubectl apply -f -
}

# Deploy base manifests with Kustomize
deploy_base() {
    log_info "Deploying base manifests with Kustomize..."
    kustomize build "${SCRIPT_DIR}/../k8s/overlays/${ENVIRONMENT}" | kubectl apply -f -
}

# Deploy Helm chart
deploy_helm_chart() {
    log_info "Deploying Helm chart: ${HELM_RELEASE_NAME}"

    # Determine values file
    VALUES_FILE="${HELM_CHART_DIR}/values-${ENVIRONMENT}.yaml"
    if [[ ! -f "${VALUES_FILE}" ]]; then
        log_warn "Values file not found: ${VALUES_FILE}, using defaults"
        VALUES_FILE="${HELM_CHART_DIR}/values.yaml"
    fi

    # Upgrade or install
    if helm status "${HELM_RELEASE_NAME}" -n "${NAMESPACE}" >/dev/null 2>&1; then
        log_info "Upgrading existing release..."
        helm upgrade "${HELM_RELEASE_NAME}" "${HELM_CHART_DIR}" \
            --namespace "${NAMESPACE}" \
            --values "${VALUES_FILE}" \
            --wait \
            --timeout 10m \
            --atomic
    else
        log_info "Installing new release..."
        helm install "${HELM_RELEASE_NAME}" "${HELM_CHART_DIR}" \
            --namespace "${NAMESPACE}" \
            --values "${VALUES_FILE}" \
            --wait \
            --timeout 10m \
            --create-namespace
    fi
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."

    # Wait for rollout
    kubectl rollout status deployment/"${HELM_RELEASE_NAME}-iou-api" \
        -n "${NAMESPACE}" --timeout=5m

    # Run smoke tests
    if [[ -f "${SCRIPT_DIR}/smoke-test.sh" ]]; then
        log_info "Running smoke tests..."
        "${SCRIPT_DIR}/smoke-test.sh" "${ENVIRONMENT}"
    else
        log_warn "Smoke test script not found, skipping"
    fi
}

# Show Haven+ compliance info
show_haven_info() {
    log_info "Haven+ Deployment Summary"
    echo "  Environment: ${ENVIRONMENT}"
    echo "  Namespace: ${NAMESPACE}"
    echo "  Release: ${HELM_RELEASE_NAME}"
    echo "  Haven+ Version: 1.0"
}

# Main deployment flow
main() {
    validate_environment
    check_prerequisites
    create_namespace
    deploy_base
    deploy_helm_chart
    verify_deployment
    show_haven_info
}

main "$@"

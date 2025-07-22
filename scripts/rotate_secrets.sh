#!/usr/bin/env bash
# Rotates NexusShell CI/CD secrets stored in HashiCorp Vault.
set -euo pipefail

if [[ -z "${VAULT_ADDR:-}" || -z "${VAULT_TOKEN:-}" ]]; then
  echo "VAULT_ADDR and VAULT_TOKEN must be set" >&2
  exit 1
fi

vault login -no-print "$VAULT_TOKEN"

# Rotate GitHub App OIDC token
vault write -force github/rotate-token path=nexusshell/ci

echo "Secrets rotated at $(date -u +%FT%TZ)" 
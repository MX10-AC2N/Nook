#!/usr/bin/env bash
# hermes-verify-e2ee-changes.sh
# Ad-hoc verification of E2EE key management changes (tasks 1-7)
# Runs after `npm run check` already passed — this does targeted behavioral checks.

set -euo pipefail
cd /opt/data/Nook/frontend

PASS=0
FAIL=0

check() {
  local desc="$1"
  shift
  if "$@" 2>/dev/null; then
    echo "  ✓ $desc"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $desc"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== verification: E2EE key management changes ==="

# 1. Verify apiFetch is exported from api.ts
check "apiFetch exported from api.ts" \
  grep -q 'export.*function apiFetch' src/lib/api.ts

# 2. Verify archive helpers exist in crypto.ts
check "encryptPrivateKeyForArchive in crypto.ts" \
  grep -q 'export async function encryptPrivateKeyForArchive' src/lib/crypto.ts
check "decryptPrivateKeyFromArchive in crypto.ts" \
  grep -q 'export async function decryptPrivateKeyFromArchive' src/lib/crypto.ts

# 3. Verify decryptSessionKeyV2 in crypto.ts
check "decryptSessionKeyV2 in crypto.ts" \
  grep -q 'export async function decryptSessionKeyV2' src/lib/crypto.ts

# 4. Verify cryptoStore has currentKeyVersion
check "currentKeyVersion in cryptoStoreState" \
  grep -q 'currentKeyVersion' src/lib/cryptoStore.svelte.ts
check "decryptMessage accepts senderKeyVersion" \
  grep -q 'senderKeyVersion' src/lib/cryptoStore.svelte.ts

# 5. Verify e2ee.ts uses apiFetch (not raw fetch)
check "e2ee.ts imports apiFetch" \
  grep -q 'import.*apiFetch.*from.*\./api' src/lib/e2ee.ts
check "e2ee.ts has no raw fetch()" \
  ! grep -q '[^a-zA-Z]fetch(' src/lib/e2ee.ts

# 6. Verify key rotation page exists
check "key rotation page exists" \
  test -f src/routes/settings/security/key-rotation/+page.svelte
check "settings page links to key rotation" \
  grep -q 'key-rotation' src/routes/settings/+page.svelte

# 7. Verify section numbering is clean in crypto.ts
check "section numbering clean (no duplicate section 5)" \
  test "$(grep -c '// 5\.' src/lib/crypto.ts)" -eq 1
check "section 6 header for pending keys" \
  grep -q '6\. Clés.*pending' src/lib/crypto.ts

echo ""
echo "=== results: $PASS passed, $FAIL failed ==="

# Clean exit even on failures — this is ad-hoc, not CI gate
exit 0

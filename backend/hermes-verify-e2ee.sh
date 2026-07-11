#!/usr/bin/env bash
# Ad-hoc verification for E2EE fix t_23eb9ad6
# Confirms: rotate_key archive PK fix, assertion fix, store_message_keys FK fix
set -euo pipefail

echo "=== E2EE FIX VERIFICATION ==="
echo ""

BACKEND="/opt/data/Nook/backend"
RUST_HOME="/tmp/rust-home/bin"

EXIT=0
for t in test_rotate_key_increments_version test_rotate_key_archives_previous \
         test_get_key_history_after_rotation test_get_member_public_keys_ok \
         test_store_message_keys_persists test_store_message_keys_upsert; do
  echo -n "  $t ... "
  if PATH="$RUST_HOME:$PATH" cargo test --manifest-path "$BACKEND/Cargo.toml" "$t" 2>&1 | grep -q "test result: ok"; then
    echo "PASS"
  else
    echo "FAIL"
    EXIT=1
  fi
done

echo ""
echo "  === INDIVIDUAL: $([ "$EXIT" -eq 0 ] && echo 'ALL PASSED' || echo 'SOME FAILED') ==="

# Full suite
E2EE=$(PATH="$RUST_HOME:$PATH" cargo test --manifest-path "$BACKEND/Cargo.toml" e2ee 2>&1)
OK=$(echo "$E2EE" | grep "test e2ee::tests::" | grep -c "ok$")
TOTAL=$(echo "$E2EE" | grep -c "^test e2ee::tests::")
echo "  === FULL SUITE: $OK/$TOTAL passing ==="

[ "$OK" -eq "$TOTAL" ] || exit 1
exit $EXIT

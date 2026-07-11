#!/usr/bin/env bash
# Ad-hoc verification: E2EE test fixes in backend/src/e2ee.rs
# Run cargo test on each of the previously-failing test names individually

set -euo pipefail

BACKEND="/opt/data/Nook/backend"
RUST_HOME="/tmp/rust-home/bin"
PASS=0
FAIL=0

echo "=== E2EE verification -- fix t_23eb9ad6 ==="
echo ""

TESTS=(
  "test_rotate_key_increments_version"
  "test_rotate_key_archives_previous"
  "test_get_key_history_after_rotation"
  "test_get_member_public_keys_ok"
  "test_store_message_keys_persists"
  "test_store_message_keys_upsert"
)

for t in "${TESTS[@]}"; do
  echo -n "  $t ... "
  if PATH="$RUST_HOME:$PATH" cargo test --manifest-path "$BACKEND/Cargo.toml" "$t" 2>&1 | grep -q "test result:"; then
    echo "PASS"
    : $((PASS += 1))
  else
    echo "FAIL"
    : $((FAIL += 1))
  fi
done

echo ""
echo "  Previously-failing: $PASS pass, $FAIL fail"
echo ""

# Full e2ee suite
E2EE=$(PATH="$RUST_HOME:$PATH" cargo test --manifest-path "$BACKEND/Cargo.toml" e2ee 2>&1)
TOTAL=$(echo "$E2EE" | grep -c "^test e2ee::tests::")
OK=$(echo "$E2EE" | grep -c "^test e2ee::tests::.* ok")
echo "  Full e2ee suite: $OK/$TOTAL passing"

if [ "$OK" -eq "$TOTAL" ] 2>/dev/null; then
  [ "$FAIL" -eq 0 ] && echo "  Status: ALL GOOD" && exit 0
fi
echo "  Status: FAILURES REMAINING"
exit 1

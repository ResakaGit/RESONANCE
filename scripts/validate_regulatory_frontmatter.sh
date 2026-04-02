#!/usr/bin/env bash
# validate_regulatory_frontmatter.sh
#
# Validates YAML frontmatter in all RESONANCE regulatory documents.
# Required fields: document_id, title, version, date, status, author,
#                  approved_by, review_date, review_status
#
# Usage: ./scripts/validate_regulatory_frontmatter.sh
# Exit 0 if all pass, exit 1 if any fail.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG_DIR="$REPO_ROOT/docs/regulatory"

REQUIRED_FIELDS="document_id title version date status author approved_by review_date review_status"

total=0
pass=0
fail=0
approved=0
pending=0
revision_requested=0
rejected=0

# Collect all regulatory .md files, excluding index/log files
files=$(find "$REG_DIR" -name "*.md" \
    -not -name "AUDIT_CHECKLIST.md" \
    -not -name "REVIEW_LOG.md" | sort)

for f in $files; do
    total=$((total + 1))
    relpath="${f#$REPO_ROOT/}"
    missing=""

    # Extract frontmatter (between first two --- lines)
    frontmatter=$(awk '/^---$/ { count++; if (count == 2) exit; next } count == 1 { print }' "$f")

    if [ -z "$frontmatter" ]; then
        echo "FAIL: $relpath — no YAML frontmatter found"
        fail=$((fail + 1))
        continue
    fi

    for field in $REQUIRED_FIELDS; do
        if ! echo "$frontmatter" | grep -q "^${field}:"; then
            missing="$missing $field"
        fi
    done

    if [ -n "$missing" ]; then
        echo "FAIL: $relpath — missing fields:$missing"
        fail=$((fail + 1))
    else
        echo "PASS: $relpath"
        pass=$((pass + 1))
    fi

    # Count review_status disposition
    rs=$(echo "$frontmatter" | grep "^review_status:" | sed 's/^review_status: *//')
    case "$rs" in
        APPROVED)           approved=$((approved + 1)) ;;
        PENDING)            pending=$((pending + 1)) ;;
        REVISION_REQUESTED) revision_requested=$((revision_requested + 1)) ;;
        REJECTED)           rejected=$((rejected + 1)) ;;
    esac
done

echo ""
echo "=== Summary ==="
echo "Total documents:     $total"
echo "Passed validation:   $pass"
echo "Failed validation:   $fail"
echo ""
echo "=== Review Status ==="
echo "APPROVED:            $approved"
echo "PENDING:             $pending"
echo "REVISION_REQUESTED:  $revision_requested"
echo "REJECTED:            $rejected"

if [ "$fail" -gt 0 ]; then
    exit 1
else
    echo ""
    echo "All documents passed frontmatter validation."
    exit 0
fi

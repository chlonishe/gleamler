#!/usr/bin/env bash
set -euo pipefail

TARGET="$1"
LIB="$2"

echo "=== Checking $TARGET: $LIB ==="

if [ ! -f "$LIB" ]; then
  echo "ERROR: Artifact $LIB does not exist!"
  exit 1
fi

file "$LIB" || true

case "$TARGET" in
  *linux*)
    readelf -h "$LIB" | grep -E "Machine:|Class:" || true
    readelf -s "$LIB" | grep -E "nif_init" || {
      echo "ERROR: nif_init not found in symbol table"
      exit 1
    }
    ;;
  *darwin*)
    nm "$LIB" | grep -E "nif_init" || {
      echo "ERROR: nif_init not found"
      exit 1
    }
    ;;
  *windows*)
    if command -v dumpbin &> /dev/null; then
      dumpbin /EXPORTS "$LIB" | grep -i "nif_init" || exit 1
    elif command -v llvm-nm &> /dev/null; then
      llvm-nm "$LIB" | grep "nif_init" || exit 1
    elif command -v nm &> /dev/null; then
      nm "$LIB" | grep "nif_init" || exit 1
    else
      echo "Warning: No dumpbin or nm found on Windows, skipping symbol inspection"
    fi
    ;;
esac

echo "=== OK: $TARGET ==="

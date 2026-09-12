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
    if objdump -p "$LIB" 2>/dev/null | grep -E "nif_init" >/dev/null; then
      echo "Found nif_init in PE export table"
    elif strings "$LIB" 2>/dev/null | grep -E "nif_init" >/dev/null; then
      echo "Found nif_init in binary exports"
    else
      echo "ERROR: nif_init not found in Windows DLL"
      exit 1
    fi
    ;;
esac

echo "=== OK: $TARGET ==="

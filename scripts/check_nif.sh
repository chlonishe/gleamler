#!/usr/bin/env bash
set -euo pipefail

TARGET="$1"
LIB="$2"

echo "=== Checking $TARGET ==="
file "$LIB"

case "$TARGET" in
  *linux*)
    readelf -h "$LIB" | grep -E "Machine:|Class:"
    readelf -s "$LIB" | grep -w "nif_init" || {
      echo "ERROR: nif_init not found in symbol table"
      exit 1
    }
    readelf -d "$LIB" | grep -E "NEEDED|SONAME" || true
    ;;
  *darwin*)
    nm "$LIB" | grep -w "nif_init" || {
      echo "ERROR: nif_init not found"
      exit 1
    }
    lipo -info "$LIB" || true
    ;;
  *windows*)
    llvm-nm "$LIB" | grep -w "nif_init" || {
      echo "ERROR: nif_init not found"
      exit 1
    }
    ;;
esac

echo "=== OK: $TARGET ==="

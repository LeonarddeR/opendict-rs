#!/usr/bin/env bash
# Populates examples/expo-test/dicts/ with bundled fixtures and any local
# dictionaries from tests/dicts/. Safe to re-run.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DEST="$ROOT/examples/expo-test/dicts"
FIXTURES="$ROOT/tests/fixtures"
LOCAL_DICTS="$ROOT/tests/dicts"

mkdir -p "$DEST"

echo "▸ Copying small fixtures..."
mkdir -p "$DEST/test-mdx"
cp "$FIXTURES/test.mdx" "$DEST/test-mdx/"

mkdir -p "$DEST/test-stardict"
cp "$FIXTURES/testdict.ifo" "$FIXTURES/testdict.idx" "$FIXTURES/testdict.dict" "$FIXTURES/testdict.syn" "$DEST/test-stardict/"

# Returns the dir under $1 that actually contains a .mdx or .ifo file.
# Walks down at most a couple of levels to handle dicts packaged in nested folders.
find_dict_root() {
  local root="$1"
  # Top level first
  if ls "$root"/*.mdx "$root"/*.ifo 2>/dev/null | head -1 >/dev/null; then
    echo "$root"
    return
  fi
  # Search up to 3 levels deep
  local found
  found=$(find "$root" -maxdepth 3 -type f \( -name '*.mdx' -o -name '*.ifo' \) 2>/dev/null | head -1)
  if [ -n "$found" ]; then
    dirname "$found"
  fi
}

if [ -d "$LOCAL_DICTS" ]; then
  echo "▸ Copying local dictionaries from tests/dicts/..."
  for category in mdict stardict; do
    if [ -d "$LOCAL_DICTS/$category" ]; then
      for dict_dir in "$LOCAL_DICTS/$category"/*/; do
        [ -d "$dict_dir" ] || continue
        name=$(basename "$dict_dir")
        actual=$(find_dict_root "$dict_dir")
        if [ -z "$actual" ]; then
          echo "    ⚠  $category/$name — no .mdx/.ifo found, skipping"
          continue
        fi
        echo "    $category/$name"
        rm -rf "$DEST/$name"
        mkdir -p "$DEST/$name"
        # Copy only files (not subdirs) — dict files are siblings
        find "$actual" -maxdepth 1 -type f -exec cp {} "$DEST/$name/" \;
      done
    fi
  done
fi

echo ""
echo "✓ Dicts staged in: $DEST"
ls -1 "$DEST" | grep -v '^README' || true
echo ""
echo "Next: ./scripts/push-dicts.sh   (after the simulator/emulator is running)"

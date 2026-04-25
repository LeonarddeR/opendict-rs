#!/usr/bin/env bash
# Pushes everything in examples/expo-test/dicts/ to the running simulator
# (iOS) and/or emulator (Android), into the app's documents directory.
# Run after the app has been installed at least once via `npm run ios` /
# `npm run android`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SRC="$ROOT/examples/expo-test/dicts"
IOS_BUNDLE_ID="com.opendict.expotest"
ANDROID_PACKAGE="com.opendict.expotest"

if [ ! -d "$SRC" ] || [ -z "$(ls -A "$SRC" 2>/dev/null | grep -v '^README')" ]; then
  echo "No dicts in $SRC. Run setup-dicts.sh first."
  exit 1
fi

# ── iOS ──

if xcrun simctl list devices booted 2>/dev/null | grep -q Booted; then
  echo "▸ Pushing dicts to iOS simulator..."
  if APP_DATA=$(xcrun simctl get_app_container booted "$IOS_BUNDLE_ID" data 2>/dev/null); then
    DOCS="$APP_DATA/Documents/dicts"
    rm -rf "$DOCS"
    mkdir -p "$DOCS"
    for d in "$SRC"/*/; do
      [ -d "$d" ] || continue
      name=$(basename "$d")
      cp -R "$d" "$DOCS/$name"
      echo "    ✓ $name"
    done
  else
    echo "    iOS app not installed yet — run \`npm run ios\` first."
  fi
else
  echo "▸ No iOS simulator booted, skipping."
fi

# ── Android ──

if command -v adb >/dev/null 2>&1 && adb devices 2>/dev/null | grep -qE 'emulator-|device$'; then
  echo "▸ Pushing dicts to Android emulator..."
  REMOTE="/storage/emulated/0/Android/data/$ANDROID_PACKAGE/files/dicts"
  adb shell "rm -rf $REMOTE && mkdir -p $REMOTE" >/dev/null 2>&1 || true
  for d in "$SRC"/*/; do
    [ -d "$d" ] || continue
    name=$(basename "$d")
    adb push "$d" "$REMOTE/$name" >/dev/null 2>&1 && echo "    ✓ $name"
  done
else
  echo "▸ No Android emulator detected, skipping."
fi

echo ""
echo "Done. Reload the app to pick up the new dicts."

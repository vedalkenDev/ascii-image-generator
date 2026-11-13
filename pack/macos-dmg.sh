#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

app_name="Cosmic Archive ASCII"
bin_name="CosmicArchiveASCII"
dist="$root/dist"
app="$dist/${app_name}.app"
arm="target/aarch64-apple-darwin/release/ascii-image-generator"
intel="target/x86_64-apple-darwin/release/ascii-image-generator"

if [[ ! -x "$arm" || ! -x "$intel" ]]; then
  echo "missing release binaries; build aarch64-apple-darwin and x86_64-apple-darwin first" >&2
  exit 1
fi

rm -rf "$dist"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

lipo -create -output "$app/Contents/MacOS/$bin_name" "$arm" "$intel"
chmod +x "$app/Contents/MacOS/$bin_name"

cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>CosmicArchiveASCII</string>
  <key>CFBundleIdentifier</key>
  <string>dev.vedalken.ascii-image-generator</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Cosmic Archive ASCII</string>
  <key>CFBundleDisplayName</key>
  <string>Cosmic Archive ASCII</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

hdiutil create \
  -volname "$app_name" \
  -srcfolder "$app" \
  -ov \
  -format UDZO \
  "$dist/Cosmic-Archive-ASCII-macos.dmg"

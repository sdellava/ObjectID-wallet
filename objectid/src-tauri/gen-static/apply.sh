# Copies all files tracked by git (/gen-static) to the generated resources (/gen)
set -euo pipefail

copy_file() {
  local source="$1"
  local target="$2"

  if [[ ! -f "$source" ]]; then
    echo "skip $source"
    return
  fi

  mkdir -p "$(dirname "$target")"
  cp -v "$source" "$target"
}

copy_file ./android/app/proguard-rules.pro ../gen/android/app/proguard-rules.pro
cp -R ./android/app/src/main/res/. ../gen/android/app/src/main/res/
copy_file ./android/app/src/main/java/com/impierce/identity_wallet/MainActivity.kt ../gen/android/app/src/main/java/com/impierce/identity_wallet/MainActivity.kt
copy_file ./android/app/src/main/AndroidManifest.xml ../gen/android/app/src/main/AndroidManifest.xml
copy_file ./apple/ExportOptions.plist ../gen/apple/ExportOptions.plist
copy_file ./apple/objectid_iOS/Info.plist ../gen/apple/objectid_iOS/Info.plist
copy_file ./apple/unime_iOS/Info.plist ../gen/apple/unime_iOS/Info.plist

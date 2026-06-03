# Copies the release artifacts (iOS, Android) from the generated resources to the /gen root folder to be uploaded to the app stores.

cp -v ../objectid/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab ../objectid/src-tauri/gen/
cp -v ../objectid/src-tauri/gen/apple/build/arm64/ObjectID.ipa ../objectid/src-tauri/gen/

# Update Tauri guide

Check latest versions of tauri packages and update them accordingly (https://crates.io/)  
Cargo.toml:  
```toml
[build-dependencies]
tauri-build = { version = "2.0.5", features = [] }

[dependencies]
# Tauri
tauri = { version = "2.2.5", features = ["tray-icon", "image-png", "config-json5", "devtools"] }
tauri-plugin-shell = "2.2.1"
tauri-plugin-dialog = "2.2.0"
tauri-plugin-clipboard-manager = "2.1.11"
tauri-plugin-os = "2.2.0"
tauri-plugin-single-instance = "2.2.1"
tauri-plugin-notification = "2.2.1"
tauri-plugin-log = "2.2.1"
tauri-plugin-window-state = "2.2.1"
tauri-plugin-websocket = "2"
```

Run:
```sh
cargo update
```

> Make sure to update the npm tauri plugin dependencies to have the same version as the rust plugins. (desktop/angular)

## Update WIX installer template

> If the migration functionality is not needed anymore remove the template, this will cause tauri to use its default template and not call the migration script.

1. Get the latest [main.wxs](https://github.com/tauri-apps/tauri/blob/dev/tooling/bundler/src/bundle/windows/templates/main.wxs) template from the repository.
2. Replace the contents of `templates/wix/main_original.wxs` with the repository version. (The file is kept only for reference)
3. Replace the contents of `templates/wix/main.wsx` and add the fallowing lines at the end of the file, inside the `Product` tag. 
```xml
    <!-- Service fragments -->
    <CustomActionRef Id='MigrationPropertySet' />
    <CustomActionRef Id='Migration' />
    <!-- Uncommenting the next line will cause the installer to check if the old service is running and fail. Without it, it will automatically stop and remove the old service without notifying the user. -->
    <!-- <CustomActionRef Id='CheckServiceStatus' /> -->
    <!-- End Service fragments -->
```

# BUILD — Warp Lite (macOS)

Hướng dẫn build `Warp Lite.app` (fork của Warp) và đóng gói `.dmg` để phân phối.

- Bin target: `warp-nine` (`app/src/bin/oss.rs`) — tên nội bộ, **không** đổi.
- App hiển thị: **Warp Lite** (`CFBundleName`), identifier `dev.warp.WarpLite`.
- Metadata bundle (tên, identifier, icon) nằm ở `[package.metadata.bundle.bin.warp-nine]` trong `app/Cargo.toml`.

---

## 0. Yêu cầu

- macOS + Xcode/Command Line Tools (Apple clang).
- `cargo install cargo-bundle`.
- Đóng gói DMG dùng `hdiutil` (có sẵn trên macOS).

### Feature bắt buộc: `gui,local_fs`

- `gui` → bật GUI desktop.
- `local_fs` → bật file search trong repo. **Thiếu `local_fs`**, `FileSearchModel::get_repo_contents` biên dịch nhánh stub trả rỗng → `@`-menu và command-palette file search **im lặng không có kết quả**. `local_fs` không nằm trong `default`/`gui` nên **phải truyền tay**.

### Bẫy môi trường trên máy này (Android NDK clang)

Android NDK clang đứng trước Xcode trên `PATH` (`~/Library/Android/sdk/ndk/.../bin`, symlink sang `/Volumes/ExternalSSD/Android/sdk`). Nó làm:
- bindgen trong `crates/warpui/build.rs` không tìm thấy `simd/simd.h`;
- `build.rs` gọi `clang --print-search-dirs` ra đường dẫn `clang/19/lib/darwin` không tồn tại → link lỗi `ld: library 'clang_rt.osx' not found`.

**Fix (env-only, không sửa `build.rs`):** chạy khối sau trong cùng shell trước khi build.

```bash
export PATH="$(echo "$PATH" | tr ':' '\n' | grep -vi 'Android/sdk' | paste -sd ':' -)"
export LIBCLANG_PATH="/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib"
export SDKROOT="$(xcrun --show-sdk-path)"
export BINDGEN_EXTRA_CLANG_ARGS="-isysroot $SDKROOT"
export CC=/usr/bin/clang CXX=/usr/bin/clang++
```

> Nếu `warpui` từng build bằng toolchain sai: `cargo clean -p warpui` để build script phát lại đường dẫn Apple.

---

## 1. Build DEBUG (test nhanh)

Chạy **từ thư mục `app/`** (gốc repo không có `[package]`). Áp dụng khối env ở trên trước.

```bash
cd app
cargo bundle --bin warp-nine --features gui,local_fs
```

- Output: `target/debug/bundle/osx/Warp Lite.app`
- Nhanh (~2–3 phút incremental). Dùng để test thay đổi.

Mở thử:

```bash
open "target/debug/bundle/osx/Warp Lite.app"
```

> **Lưu ý debug:** debug build bật `DEBUG_FLAGS` và `debug_assert!`. Nếu một menu action thiếu description, `app_menus.rs` sẽ panic lúc khởi động (release không dính vì `debug_assert!` là no-op). Log panic ở `~/Library/Logs/warp-nine.log`.

---

## 2. Build RELEASE (phân phối)

```bash
cd app
cargo bundle --release --bin warp-nine --features gui,local_fs
```

- Output: `target/release/bundle/osx/Warp Lite.app`
- Chậm (~10 phút, biên dịch toàn bộ warp lib ở chế độ optimized).

### Re-sign ad-hoc (bắt buộc)

Chữ ký ad-hoc do `cargo bundle` sinh ra có thể lỗi seal (`spctl`/`codesign` báo "code has no resources but signature indicates they must be present"). Ký lại cho sạch:

```bash
APP="target/release/bundle/osx/Warp Lite.app"
codesign --force --deep -s - "$APP"
codesign --verify --deep --strict "$APP" && echo "SIGNATURE OK"
```

---

## 3. Đóng gói DMG

Chạy từ **gốc repo**. Tạo staging có app + symlink `/Applications` (kéo-thả để cài), rồi nén UDZO.

```bash
APP="target/release/bundle/osx/Warp Lite.app"
STAGE="$(mktemp -d)"
DMG="dist/Warp Lite-arm64.dmg"

mkdir -p dist
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"

rm -f "$DMG"
hdiutil create -volname "Warp Lite" -srcfolder "$STAGE" -ov -format UDZO "$DMG"
rm -rf "$STAGE"

# Kiểm tra
hdiutil verify "$DMG"
shasum -a 256 "$DMG"
```

- Output: `dist/Warp Lite-arm64.dmg` (~41 MB).

---

## 4. Cài đặt & Gatekeeper

App chỉ **ad-hoc signed**, chưa Developer ID + notarize → lần đầu macOS chặn "unidentified developer".

- Chuột phải vào app → **Open** → **Open**, hoặc
- `xattr -dr com.apple.quarantine "/Applications/Warp Lite.app"`

---

## 5. Giới hạn hiện tại

- **Chỉ arm64** (Apple Silicon). Build Intel: thêm `--target x86_64-apple-darwin`. Gộp universal: `lipo -create <arm64-bin> <x86_64-bin> -output <universal-bin>`.
- Chưa notarize (cần tài khoản Apple Developer).
- Icon tối đa **512px** — `cargo bundle` trên máy này không hỗ trợ slot icns 1024 ("No matching IconType"). File icon nguồn: `app/channels/oss/icon/no-padding/{512x512.png, icon.ico}`.

---

## Tóm tắt lệnh

```bash
# (một lần mỗi shell) áp env fix ở Mục 0, rồi:

# DEBUG
cd app && cargo bundle --bin warp-nine --features gui,local_fs

# RELEASE + re-sign + DMG
cd app && cargo bundle --release --bin warp-nine --features gui,local_fs && cd ..
codesign --force --deep -s - "target/release/bundle/osx/Warp Lite.app"
STAGE="$(mktemp -d)"; mkdir -p dist
cp -R "target/release/bundle/osx/Warp Lite.app" "$STAGE/"; ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "Warp Lite" -srcfolder "$STAGE" -ov -format UDZO "dist/Warp Lite-arm64.dmg"; rm -rf "$STAGE"
```

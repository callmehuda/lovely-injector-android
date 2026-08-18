# Lovely (Android port)

Lovely is a runtime Lua injector for [LÖVE 2D](https://love2d.org/) games. This fork is **Android-only** — it ships just the native Android library (`liblovely.so`) and the shared `lovely-core` patch engine.

## How it works

When the game loads a Lua chunk via `luaL_loadbufferx` / `luaL_loadbuffer`, Lovely intercepts the call, applies any registered patches in memory, and forwards the patched buffer to the real Lua loader. The game's installed files on disk are never modified, so mods can be installed, updated, and removed at will without touching the game itself.

## Default mod directory

On Android, mods are read from:

```
/sdcard/Documents/Balatro/Mods
```

This is the canonical user-facing path (`/sdcard/...` is a symlink to `/storage/emulated/0/...`), reachable from any file manager. The path can always be overridden at runtime by setting `LOVELY_MOD_DIR` in the environment before launching the game — the env var is checked first, the constant above is just the fallback default.

## Building

The Android `.so` is built with the Android NDK + a Rust nightly toolchain. The CI workflow in [`.github/workflows/android-build.yml`](.github/workflows/android-build.yml) produces binaries for both `arm64-v8a` and `armeabi-v7a` on every push.

### Local build (one ABI)

```bash
rustup toolchain install nightly
rustup target add aarch64-linux-android --toolchain nightly

export ANDROID_NDK_HOME=/path/to/Android/sdk/ndk/<version>
export CC_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang
export CXX_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang++
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$CC_aarch64_linux_android
export AR_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar
export RANLIB_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ranlib

cargo build --release --lib -p lovely-android --target aarch64-linux-android
```

Output: `target/aarch64-linux-android/release/liblovely.so`.

### CI build

Every push triggers `.github/workflows/android-build.yml`, which:

1. Sets up Rust nightly + the NDK (`r26d`).
2. Builds `liblovely.so` for `aarch64-linux-android` and `armv7-linux-androideabi` in parallel.
3. Uploads each ABI as a separate artifact, then merges them into a single `liblovely-android-all-abis` artifact.

You can also trigger the workflow manually from the GitHub Actions UI ("Run workflow" button).

## Installation (end user)

1. Download the latest `liblovely-android-all-abis` artifact from the Actions tab.
2. Unzip and copy the relevant `liblovely.so` (under `arm64-v8a/` for modern phones, `armeabi-v7a/` for older 32-bit devices) into your game's APK `lib/<abi>/` folder.
3. Put your mods in `/sdcard/Documents/Balatro/Mods/` — create the directory if it doesn't exist.
4. Launch the game.

## Patch format

Each `lovely.toml` (or `lovely/*.toml`) defines one or more patches:

- **`[patches.pattern]`** — wildcard line matcher (`?` = one char, `*` = any). Inject payload before/after/at the matched line.
- **`[patches.regex]`** — full regex engine with capture group interpolation.
- **`[patches.copy]`** — append/prepend one or more files' contents onto the target.
- **`[patches.module]`** — inject a new require-able Lua module into the game's `package.preload` table.

See the upstream [Lovely README](https://github.com/ethangreen-dev/lovely-injector) for the full patch format documentation.

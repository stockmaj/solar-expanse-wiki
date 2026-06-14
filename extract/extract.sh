#!/usr/bin/env bash
# Regenerate the Solar Expanse wiki from a local game install.
#
# Pipeline:
#   1. AssetRipper dumps the Unity binary assets to a Unity project tree.
#   2. parse-locale (Rust) reads StreamingAssets/Languages/en-US.csv → locale.json.
#   3. parse-stats  (Rust) reads MyScene.unity → stats.json.
#   4. gen-pages    (Rust) reads both → emits all wiki Markdown pages.
#
# Override the game install with SOLAR_EXPANSE_DATA=<path>.
# Skip AssetRipper (re-use the previous dump) with FAST=1.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WIKI_ROOT="$(cd "$HERE/.." && pwd)"
CACHE="$HERE/cache"
TOOLS="$HERE/tools"
PORT="${ASSETRIPPER_PORT:-7700}"

ASSETRIPPER_VERSION="${ASSETRIPPER_VERSION:-1.3.14}"
ASSETRIPPER_URL="https://github.com/AssetRipper/AssetRipper/releases/download/${ASSETRIPPER_VERSION}/AssetRipper_mac_arm64.zip"
ASSETRIPPER_BIN="$TOOLS/AssetRipper.GUI.Free"

DEFAULT_GAME_DATA="/Users/$(whoami)/Applications/Sikarugir/Steam Wine.app/Contents/SharedSupport/prefix/drive_c/Program Files (x86)/Steam/steamapps/common/Solar Expanse/Solar Expanse_Data"
GAME_DATA="${SOLAR_EXPANSE_DATA:-$DEFAULT_GAME_DATA}"

log() { printf '\033[1;36m[extract]\033[0m %s\n' "$*"; }

require() { command -v "$1" >/dev/null 2>&1 || { echo "missing dependency: $1" >&2; exit 1; }; }
require curl
require jq
require unzip

if [[ ! -d "$GAME_DATA" ]]; then
    echo "Solar Expanse_Data not found at: $GAME_DATA" >&2
    echo "Set SOLAR_EXPANSE_DATA to override." >&2
    exit 1
fi

mkdir -p "$CACHE" "$TOOLS"

ensure_assetripper() {
    if [[ -x "$ASSETRIPPER_BIN" ]]; then return; fi
    log "downloading AssetRipper $ASSETRIPPER_VERSION"
    local zip="$TOOLS/AssetRipper.zip"
    curl -fsSL -o "$zip" "$ASSETRIPPER_URL"
    unzip -q -o "$zip" -d "$TOOLS"
    rm -f "$zip"
    xattr -dr com.apple.quarantine "$TOOLS" 2>/dev/null || true
    chmod +x "$ASSETRIPPER_BIN"
}

ensure_cargo() {
    if command -v cargo >/dev/null 2>&1; then return; fi
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
        return
    fi
    echo "cargo not found. Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
    exit 1
}

run_assetripper() {
    if [[ "${FAST:-0}" == "1" && -f "$CACHE/project/ExportedProject/Assets/Scenes/MyScene.unity" ]]; then
        log "FAST=1 set, reusing existing dump at $CACHE/project"
        return
    fi
    ensure_assetripper

    log "starting AssetRipper headless on port $PORT"
    rm -rf "$CACHE/project" && mkdir -p "$CACHE/project"

    "$ASSETRIPPER_BIN" --headless --port "$PORT" --log-path "$CACHE/assetripper.log" \
        > "$CACHE/assetripper.stdout" 2>&1 &
    local pid=$!
    trap "kill $pid 2>/dev/null || true" EXIT

    # Wait for the HTTP server
    for _ in {1..30}; do
        if curl -sf "http://localhost:$PORT/" >/dev/null 2>&1; then break; fi
        sleep 0.5
    done

    log "loading game folder"
    curl -fsS -X POST "http://localhost:$PORT/LoadFolder" \
        --data-urlencode "path=$GAME_DATA" \
        -o /dev/null

    log "exporting Unity project (this takes ~30 s)"
    curl -fsS -X POST "http://localhost:$PORT/Export/UnityProject" \
        --data-urlencode "path=$CACHE/project" \
        -o /dev/null

    # Tail the log until the export reports completion
    until grep -q "Finished exporting Unity project\|Finished post-export" "$CACHE/assetripper.log" 2>/dev/null; do
        sleep 1
    done

    kill $pid 2>/dev/null || true
    trap - EXIT
    log "AssetRipper finished"
}

run_pipeline() {
    ensure_cargo
    log "building Rust binaries"
    (cd "$HERE" && cargo build --release --quiet)
    local bindir="$HERE/target/release"

    local en_us="$GAME_DATA/StreamingAssets/Languages/en-US.csv"
    local scene="$CACHE/project/ExportedProject/Assets/Scenes/MyScene.unity"

    log "parse-locale"
    "$bindir/parse-locale" "$en_us" "$CACHE/locale.json"

    log "parse-stats"
    "$bindir/parse-stats" "$scene" "$CACHE/stats.json"

    log "parse-sirenix"
    "$bindir/parse-sirenix" "$CACHE/sirenix-dump.json" "$CACHE/sirenix.json"

    log "parse-terraformation-config"
    "$bindir/parse-terraformation-config" \
        "$CACHE/project/ExportedProject/Assets/MonoBehaviour/TerraformationConfig.asset" \
        "$CACHE/terraformation.json"

    log "extract-icons"
    # extract-icons writes one subdir per category (resources/, research/,
    # planet-types/) under the given root, so the root is `docs/images` —
    # not `docs/images/resources`, which would produce a doubled
    # `images/resources/resources/` path.
    "$bindir/extract-icons" "$CACHE/project/ExportedProject" "$WIKI_ROOT/docs/images"

    local proj_settings="$CACHE/project/ExportedProject/ProjectSettings/ProjectSettings.asset"
    local game_version="unknown"
    if [[ -f "$proj_settings" ]]; then
        # Unity stores the game version as `  bundleVersion: 0.26.5.15.14 BETA`.
        game_version="$(awk -F': ' '/^[[:space:]]*bundleVersion:/{print $2; exit}' "$proj_settings")"
        [[ -z "$game_version" ]] && game_version="unknown"
    fi
    log "game version: $game_version"

    log "gen-pages"
    "$bindir/gen-pages" "$CACHE/locale.json" "$CACHE/stats.json" "$CACHE/sirenix.json" "$CACHE/terraformation.json" "$WIKI_ROOT/docs" "$game_version"

    log "done"
}

check_sirenix_dump() {
    local dump="$GAME_DATA/StreamingAssets/sirenix-dump.json"
    local mod_src="$HERE/bepinex-mod/bin/Release/netstandard2.1/SolarExpanseWikiDumper.dll"
    local mod_dst="$GAME_DATA/../BepInEx/plugins/SolarExpanseWikiDumper.dll"

    if [[ -f "$dump" ]]; then
        cp "$dump" "$CACHE/sirenix-dump.json"
        log "sirenix dump available ($(du -h "$dump" | cut -f1))"
        return
    fi

    if ! command -v dotnet >/dev/null 2>&1; then
        echo "dotnet not installed; install with: brew install dotnet" >&2
        echo "Then re-run this script." >&2
        exit 1
    fi

    if [[ ! -f "$mod_src" ]]; then
        log "building BepInEx dumper mod"
        (cd "$HERE/bepinex-mod" && dotnet build -c Release --nologo --verbosity quiet)
    fi
    if [[ ! -f "$mod_dst" ]]; then
        log "installing dumper into $(dirname "$mod_dst")"
        cp "$mod_src" "$mod_dst"
    fi

    cat <<EOF >&2

  No sirenix-dump.json found at:
    $dump

  The BepInEx mod is installed but has not run yet.  Launch Solar Expanse,
  start or load any game scenario, wait a few seconds once the game world
  is visible, then quit.  The mod fires on gameplay-scene load (not the
  main menu), so you must get into an actual game.

  After that, re-run this script to continue.
EOF
    exit 2
}

run_assetripper
check_sirenix_dump
run_pipeline

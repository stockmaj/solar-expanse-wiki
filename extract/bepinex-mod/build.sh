#!/usr/bin/env bash
# Build the BepInEx mod and copy it into the game's plugins folder.
#
# Override the game install with SOLAR_EXPANSE_GAME=<path-to-Solar Expanse>.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_GAME="/Users/$(whoami)/Applications/Sikarugir/Steam Wine.app/Contents/SharedSupport/prefix/drive_c/Program Files (x86)/Steam/steamapps/common/Solar Expanse"
GAME="${SOLAR_EXPANSE_GAME:-$DEFAULT_GAME}"

if [[ ! -d "$GAME/BepInEx/plugins" ]]; then
    echo "BepInEx plugins dir not found at: $GAME/BepInEx/plugins" >&2
    echo "Set SOLAR_EXPANSE_GAME to override." >&2
    exit 1
fi

dotnet build "$HERE" -c Release --nologo --verbosity quiet
DLL=$(find "$HERE/bin/Release" -name "SolarExpanseWikiDumper.dll" | head -1)
cp "$DLL" "$GAME/BepInEx/plugins/SolarExpanseWikiDumper.dll"
echo "installed → $GAME/BepInEx/plugins/SolarExpanseWikiDumper.dll"

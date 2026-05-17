# Solar Expanse Wiki Dumper

A BepInEx mod that dumps every Sirenix Odin-serialized ScriptableObject from
Solar Expanse into a single JSON file. The rest of the wiki extraction
pipeline (Rust) consumes that JSON.

## Why this exists

The game stores its `SpacecraftType`, `LaunchVehicleType`, `ContractDefinition`,
`ResourceDefinition`, `CompanyDefinition`, and several other key data classes
using **Sirenix Odin Serializer**. AssetRipper can read Unity's standard
serialization but cannot decode Odin's binary blob, so for those classes it
emits an empty 14-line stub. This mod reads the data at runtime when the game's
own loader has already deserialized everything for us.

## Build

```bash
./build.sh
```

That runs `dotnet build` and copies the resulting DLL into the game's
`BepInEx/plugins/` folder. Pre-reqs:

- .NET SDK (`brew install dotnet`)
- BepInEx 5 already installed in the game (you already have it — `FleetTracker`,
  `LifeSupportTracker`, etc. are in `BepInEx/plugins/`)

## Run

1. Launch Solar Expanse.
2. Wait ~5 seconds after the main menu finishes loading.
3. The mod writes `sirenix-dump.json` (and a `sirenix-dump.flag` marker) into
   `Solar Expanse_Data/StreamingAssets/`.
4. Quit the game.

On subsequent launches the mod sees the marker file and no-ops. To force a fresh
dump, delete `sirenix-dump.flag` and launch again.

## What gets dumped

See `DumpTypes` in `Plugin.cs`. Add a C# class name to that set when you need to
expose a new section to the wiki. The mod uses reflection over public and
`[SerializeField]`-decorated fields, so adding a type to `DumpTypes` is the only
change needed for most cases.

Output format (truncated):

```json
{
  "SpacecraftType": [
    {
      "name": "spacecraft_chem_small",
      "engineType": "Chemical",
      "availableDeltaV": 4500.0,
      "mass": 5000.0,
      "cargoCapacity": 8000.0,
      "fuelCapacity": 15000.0,
      "reusability": 0.0,
      "thrust": 100000.0
    }
  ],
  "LaunchVehicleType": [...]
}
```

## Removing the mod

`rm "$GAME/BepInEx/plugins/SolarExpanseWikiDumper.dll"`. The dump file in
`StreamingAssets/` is harmless to leave behind, but you can delete it too.

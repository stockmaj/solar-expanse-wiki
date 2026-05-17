# Solar Expanse Wiki

An unofficial, community-maintained reference for **[Solar Expanse](https://store.steampowered.com/app/1369700/)**.

The wiki itself lives in [`docs/`](docs/) and is served at GitHub Pages.
This top-level README is for contributors who want to fix or extend the wiki.

## How the wiki is built

Almost every page in `docs/` is **generated** from the game's own files by the
pipeline in [`extract/`](extract/) — not hand-written. If you spot a wrong
number or missing column, the fix usually goes in the extractor / generator,
not the page.

The pipeline:

```
AssetRipper  ──► extracts Unity scene + asset YAML into extract/cache/project/
BepInEx mod  ──► dumps Sirenix-serialized ScriptableObjects to sirenix-dump.json
parse-locale ──► CSV localization → locale.json
parse-stats  ──► Unity scene YAML → stats.json (per-body orbital data)
parse-sirenix ─► sirenix-dump.json → sirenix.json (per-spacecraft stats etc.)
gen-pages    ──► joins all three → writes Markdown into docs/
```

To regenerate locally:

```bash
./extract/extract.sh
```

See [`extract/README.md`](extract/README.md) for details on each stage.

## Contributing

Most fixes go through the build pipeline, not direct page edits. See
[`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md).

For changes that **are** hand-written (CONTRIBUTING.md itself, the landing
page's prose intro, descriptions inside the table-rendering functions),
direct PRs to those files are fine.

## License

- Wiki content (`docs/`): **CC-BY-SA 4.0** — copy, remix, share, even commercially, with attribution and same-license derivatives. Same license as Wikipedia.
- Build pipeline (`extract/`, `bepinex-mod/`): **MIT**.

See [LICENSE](LICENSE) for full text.

## Credits

**Solar Expanse** © Maciej Miąsik / TJ Entertainment. This wiki is unofficial
fan documentation; all game text and data are property of their respective
holders and are presented here for reference under fair-use principles for
non-commercial community use.

# Contributing

Thanks for thinking about contributing! A few things to know first.

## Most pages are generated

Almost every page in this wiki is **generated** by the pipeline in
[`../extract/`](../extract/), not hand-written. That means:

- ✗ Direct edits to a generated page (e.g. `spacecraft/README.md`,
  `celestial-bodies/planets.md`) will be **overwritten** the next time the
  pipeline runs.
- ✓ Fixes should go into the generator. If a number is wrong, the table is
  missing a column, or a category label is off, the change belongs in
  [`extract/src/bin/gen_pages.rs`](../extract/src/bin/gen_pages.rs),
  [`extract/src/bin/parse_sirenix.rs`](../extract/src/bin/parse_sirenix.rs), or
  one of the other parsers.

A page header tells you whether it's generated — the top of the README at
the repo root explains the pipeline.

## Hand-written pages (direct edits OK)

A few files are not generated and accept direct PRs:

- `docs/CONTRIBUTING.md` (this file)
- The descriptive prose at the top of each section's README (it currently lives
  inside the generator's `format!` strings — open a PR against `gen_pages.rs`
  for those)

## How to PR a fix

1. Fork this repo on GitHub.
2. Make your change on a feature branch.
3. If you're changing the pipeline, run `./extract/extract.sh` locally to
   regenerate the pages and confirm the diff looks right. Include both the
   pipeline change and the regenerated output in your PR.
4. Open a PR against `main`.

You don't need a Solar Expanse install for tiny prose/typo fixes, but most
pipeline changes require one — the pipeline reads files from your game
install directly.

## Reporting bugs / missing data

If a stat is wrong or a section is missing, open an issue — easier than a PR
when the underlying problem isn't obvious yet. Include:

- The page URL and the cell / line in question
- What you see vs. what the game shows in-game
- Your game version (visible in the main menu) so we can tell whether the
  data has drifted

## License

By contributing, you agree your contributions are licensed under the same
terms as the rest of the repo:

- **Wiki content** under [CC-BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
- **Build pipeline code** under MIT

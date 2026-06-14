use anyhow::{anyhow, Context, Result};
use image::imageops::FilterType;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const TARGET_SIZE: u32 = 24;

/// A category of icons to crop from the game's sprite atlas — drives the
/// per-prefix loop in `main()`.  Each category names the MonoBehaviour prefix
/// (e.g. `id_resource_`), the YAML field that carries the Sprite GUID, the
/// subdirectory under the output root, and an optional prefix-strip for the
/// output filename so e.g. `id_resource_HEL3.asset` → `hel3.png`.
struct Category {
    monob_prefix: &'static str,
    sprite_field: &'static str,
    out_subdir: &'static str,
    strip_prefix: &'static str,
}

const CATEGORIES: &[Category] = &[
    Category {
        monob_prefix: "id_resource_",
        sprite_field: "sprite2",
        out_subdir: "resources",
        strip_prefix: "id_resource_",
    },
    Category {
        monob_prefix: "research_",
        sprite_field: "sprite",
        out_subdir: "research",
        strip_prefix: "",
    },
    Category {
        monob_prefix: "planet_",
        sprite_field: "sprite",
        out_subdir: "planet-types",
        strip_prefix: "",
    },
];

/// Crop sprite icons from each sprite's own texture atlas for each category in
/// `CATEGORIES` and write them as small PNGs under `<output dir>/<subdir>/`.
/// Each sprite YAML records its source texture via `m_RD.texture`; atlases are
/// loaded on first use and cached so large textures aren't reopened repeatedly.
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!("usage: extract-icons <ExportedProject root> <output dir>"));
    }
    let export = PathBuf::from(&args[1]);
    let out_dir = PathBuf::from(&args[2]);
    fs::create_dir_all(&out_dir)?;

    let sprite_dir = export.join("Assets/Sprite");
    let monob_dir = export.join("Assets/MonoBehaviour");
    let texture_dir = export.join("Assets/Texture2D");

    // Build guid → Sprite-asset-path map (shared across categories).
    let mut guid_to_sprite: HashMap<String, PathBuf> = HashMap::new();
    for entry in fs::read_dir(&sprite_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "meta") {
            if let Some(guid) = read_meta_guid(&path)? {
                let asset_path = path.with_extension(""); // drop .meta
                guid_to_sprite.insert(guid, asset_path);
            }
        }
    }

    // Build guid → texture-PNG-path map so each sprite can load its own atlas.
    let mut guid_to_texture: HashMap<String, PathBuf> = HashMap::new();
    for entry in fs::read_dir(&texture_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "meta") {
            if let Some(guid) = read_meta_guid(&path)? {
                let texture_path = path.with_extension(""); // drop .meta
                guid_to_texture.insert(guid, texture_path);
            }
        }
    }

    // Cache of loaded atlas images keyed by texture GUID.
    let mut texture_cache: HashMap<String, image::DynamicImage> = HashMap::new();

    // Walk MonoBehaviour/ once, dispatching each matching file to its
    // category's output dir.  Skipping non-matching files keeps the loop O(n).
    let mut written_per_cat: HashMap<&str, u32> = HashMap::new();
    for cat in CATEGORIES {
        fs::create_dir_all(out_dir.join(cat.out_subdir))?;
        written_per_cat.insert(cat.out_subdir, 0);
    }

    for entry in fs::read_dir(&monob_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("asset") {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Find which category this file belongs to (first match wins).
        let cat = match CATEGORIES.iter().find(|c| name.starts_with(c.monob_prefix)) {
            Some(c) => c,
            None => continue,
        };
        let out_name: String = if cat.strip_prefix.is_empty() {
            name.to_ascii_lowercase()
        } else {
            name.strip_prefix(cat.strip_prefix)
                .unwrap_or(name)
                .to_ascii_lowercase()
        };
        if out_name.is_empty() || out_name == "empty" {
            continue;
        }

        let text = fs::read_to_string(&path)?;
        let sprite_guid = match parse_sprite_guid(&text, cat.sprite_field) {
            Some(g) => g,
            None => continue,
        };
        let sprite_asset_path = match guid_to_sprite.get(&sprite_guid) {
            Some(p) => p,
            None => continue,
        };
        let sprite_yaml = fs::read_to_string(sprite_asset_path)?;
        let rect = match parse_rect(&sprite_yaml) {
            Some(r) => r,
            None => continue,
        };

        // Each sprite's m_RD.texture names the atlas it lives in.  Load it on
        // first use and cache it — different categories use different atlases
        // (e.g. research icons are in ResearchIconsAtlas.png, not
        // solar_expanse_icons.png).
        let texture_guid = match parse_sprite_guid(&sprite_yaml, "texture") {
            Some(g) => g,
            None => continue,
        };
        if !texture_cache.contains_key(&texture_guid) {
            let tex_path = match guid_to_texture.get(&texture_guid) {
                Some(p) => p.clone(),
                None => continue,
            };
            let img = image::open(&tex_path)
                .with_context(|| format!("opening texture {}", tex_path.display()))?;
            texture_cache.insert(texture_guid.clone(), img);
        }
        let atlas = match texture_cache.get(&texture_guid) {
            Some(a) => a,
            None => continue,
        };
        let atlas_h = atlas.height();

        // Unity stores sprite rects with Y measured from the bottom of the
        // texture.  PNG coordinates run top-down, so we flip Y.
        let x = rect.x.max(0.0) as u32;
        let w = rect.width.max(1.0) as u32;
        let h = rect.height.max(1.0) as u32;
        let y_from_top = (atlas_h as f64 - rect.y as f64 - rect.height as f64)
            .max(0.0) as u32;

        let cropped = atlas.crop_imm(x, y_from_top, w, h);
        let resized = cropped.resize(TARGET_SIZE, TARGET_SIZE, FilterType::Lanczos3);
        let out_path = out_dir.join(cat.out_subdir).join(format!("{out_name}.png"));
        resized.save(&out_path)
            .with_context(|| format!("writing {}", out_path.display()))?;
        *written_per_cat.entry(cat.out_subdir).or_insert(0) += 1;
    }

    for cat in CATEGORIES {
        let n = written_per_cat.get(cat.out_subdir).copied().unwrap_or(0);
        eprintln!(
            "wrote {} {} icons to {}/ ({} × {} px each)",
            n,
            cat.out_subdir,
            out_dir.join(cat.out_subdir).display(),
            TARGET_SIZE,
            TARGET_SIZE
        );
    }
    Ok(())
}

fn read_meta_guid(p: &Path) -> Result<Option<String>> {
    let s = fs::read_to_string(p)?;
    for line in s.lines() {
        if let Some(rest) = line.trim().strip_prefix("guid:") {
            return Ok(Some(rest.trim().to_string()));
        }
    }
    Ok(None)
}

fn parse_sprite_guid(asset_yaml: &str, field_name: &str) -> Option<String> {
    let needle = format!("{field_name}:");
    for line in asset_yaml.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(&needle) {
            // Stop matching if the line is a nested-key match (rest should
            // start with whitespace or `{`).
            let rest = rest.trim_start();
            if !rest.starts_with('{') {
                continue;
            }
            // value looks like `{fileID: 21300000, guid: adc58c..., type: 2}`
            for kv in rest.trim_start_matches('{').split(',') {
                let kv = kv.trim().trim_end_matches('}');
                if let Some(g) = kv.strip_prefix("guid:") {
                    return Some(g.trim().to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
fn parse_sprite2_guid(asset_yaml: &str) -> Option<String> {
    parse_sprite_guid(asset_yaml, "sprite2")
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn parse_rect(asset_yaml: &str) -> Option<Rect> {
    let mut in_rect = false;
    let (mut x, mut y, mut w, mut h) = (None, None, None, None);
    for line in asset_yaml.lines() {
        let stripped = line.trim_end();
        if stripped.trim_start() == "m_Rect:" {
            in_rect = true;
            continue;
        }
        if !in_rect {
            continue;
        }
        if !line.starts_with("    ") {
            // Dedented out of the m_Rect block
            break;
        }
        let trim = line.trim();
        if let Some(v) = trim.strip_prefix("x:") {
            x = v.trim().parse().ok();
        } else if let Some(v) = trim.strip_prefix("y:") {
            y = v.trim().parse().ok();
        } else if let Some(v) = trim.strip_prefix("width:") {
            w = v.trim().parse().ok();
        } else if let Some(v) = trim.strip_prefix("height:") {
            h = v.trim().parse().ok();
        }
    }
    Some(Rect {
        x: x?,
        y: y?,
        width: w?,
        height: h?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sprite2_guid_handles_inline_dict() {
        let yaml = "  sprite2: {fileID: 21300000, guid: adc58c1864171cb47aed1bc9183ff9a2, type: 2}";
        assert_eq!(
            parse_sprite2_guid(yaml).as_deref(),
            Some("adc58c1864171cb47aed1bc9183ff9a2")
        );
    }

    #[test]
    fn parse_sprite_guid_handles_single_sprite_field() {
        let yaml = "  sprite: {fileID: 21300000, guid: 05492b93a04f15543b966a910f7ef85d, type: 2}";
        assert_eq!(
            parse_sprite_guid(yaml, "sprite").as_deref(),
            Some("05492b93a04f15543b966a910f7ef85d")
        );
    }

    #[test]
    fn parse_sprite_guid_does_not_confuse_sprite_with_sprite2() {
        // Looking for `sprite:` should NOT match `sprite2:` (or vice versa).
        let yaml = "  sprite2: {fileID: 21300000, guid: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, type: 2}\n  sprite: {fileID: 21300000, guid: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, type: 2}";
        assert_eq!(
            parse_sprite_guid(yaml, "sprite").as_deref(),
            Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert_eq!(
            parse_sprite_guid(yaml, "sprite2").as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
    }

    #[test]
    fn parse_sprite_guid_skips_unset_field() {
        // A `sprite: {fileID: 0}` ref means "no sprite assigned" — we must
        // NOT return that as a real guid (the loop should keep scanning or
        // give up).
        let yaml = "  sprite: {fileID: 0}";
        assert_eq!(parse_sprite_guid(yaml, "sprite"), None);
    }

    #[test]
    fn parse_sprite_guid_extracts_texture_from_sprite_yaml() {
        // Sprite YAML has texture: in m_RD (with a guid) and again in
        // m_AtlasRD (fileID: 0, no guid).  Must return the first real guid.
        let yaml = "\
  m_RD:\n\
    serializedVersion: 3\n\
    texture: {fileID: 2800000, guid: 996a5f8d66e59be4c9e157635e05709a, type: 3}\n\
    alphaTexture: {fileID: 0}\n\
  m_AtlasRD:\n\
    serializedVersion: 3\n\
    texture: {fileID: 0}\n";
        assert_eq!(
            parse_sprite_guid(yaml, "texture").as_deref(),
            Some("996a5f8d66e59be4c9e157635e05709a")
        );
    }

    #[test]
    fn parse_rect_pulls_xywh_from_m_rect_block() {
        let yaml = "Sprite:\n  m_Rect:\n    serializedVersion: 2\n    x: 1054.1\n    y: 547.5\n    width: 196.8\n    height: 186.8\n  m_Offset: {x: 0, y: 0}\n";
        let r = parse_rect(yaml).expect("should parse");
        assert!((r.x - 1054.1).abs() < 0.01);
        assert!((r.y - 547.5).abs() < 0.01);
        assert!((r.width - 196.8).abs() < 0.01);
        assert!((r.height - 186.8).abs() < 0.01);
    }

    #[test]
    fn read_meta_guid_finds_guid_line() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "fileFormatVersion: 2\nguid: deadbeef1234\nNativeFormatImporter:\n").unwrap();
        assert_eq!(
            read_meta_guid(tmp.path()).unwrap().as_deref(),
            Some("deadbeef1234")
        );
    }
}

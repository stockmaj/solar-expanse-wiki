use anyhow::{anyhow, Context, Result};
use image::imageops::FilterType;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const TARGET_SIZE: u32 = 24;

/// Map each resource id (e.g. "steel", "water", "hel3") to a small PNG icon
/// cropped out of the game's resource sprite atlas.
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
    let atlas_path = export.join("Assets/Texture2D/solar_expanse_icons.png");
    if !atlas_path.exists() {
        return Err(anyhow!(
            "atlas PNG not found: {} — did AssetRipper export Texture2D/?",
            atlas_path.display()
        ));
    }

    let atlas = image::open(&atlas_path)
        .with_context(|| format!("opening atlas {}", atlas_path.display()))?;
    let atlas_h = atlas.height();

    // Build guid → Sprite-asset-path map.
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

    let mut written = 0;
    for entry in fs::read_dir(&monob_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("asset") {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let resource_id = match name.strip_prefix("id_resource_") {
            // Lowercase so the icon filename matches the locale's lowercase id
            // (game-side files use mixed case for a handful of resources, e.g.
            // `id_resource_HEL3.asset` while the locale carries `hel3`).
            Some(s) => s.to_ascii_lowercase(),
            None => continue,
        };
        if resource_id == "empty" {
            continue;
        }

        let text = fs::read_to_string(&path)?;
        let sprite_guid = match parse_sprite2_guid(&text) {
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

        // Unity stores sprite rects with Y measured from the bottom of the
        // texture.  PNG coordinates run top-down, so we flip Y.
        let x = rect.x.max(0.0) as u32;
        let w = rect.width.max(1.0) as u32;
        let h = rect.height.max(1.0) as u32;
        let y_from_top = (atlas_h as f64 - rect.y as f64 - rect.height as f64)
            .max(0.0) as u32;

        let cropped = atlas.crop_imm(x, y_from_top, w, h);
        let resized = cropped.resize(TARGET_SIZE, TARGET_SIZE, FilterType::Lanczos3);
        let out_path = out_dir.join(format!("{resource_id}.png"));
        resized.save(&out_path)
            .with_context(|| format!("writing {}", out_path.display()))?;
        written += 1;
    }

    eprintln!(
        "wrote {} resource icons to {} ({} × {} px each)",
        written,
        out_dir.display(),
        TARGET_SIZE,
        TARGET_SIZE
    );
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

fn parse_sprite2_guid(asset_yaml: &str) -> Option<String> {
    for line in asset_yaml.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("sprite2:") {
            // value looks like `{fileID: 21300000, guid: adc58c..., type: 2}`
            for kv in rest.trim_start().trim_start_matches('{').split(',') {
                let kv = kv.trim().trim_end_matches('}');
                if let Some(g) = kv.strip_prefix("guid:") {
                    return Some(g.trim().to_string());
                }
            }
        }
    }
    None
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

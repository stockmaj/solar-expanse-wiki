use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const GUID_SOLAR_BODY: &str = "ae426645999213f30e86342528c32628";
const GUID_NBODY: &str = "bb2f3ea79264b8a4ef2ff2823a0af4d7";
const GUID_ORBIT_UNIVERSAL: &str = "34376944fc28cccd32412aa34287d642";

#[derive(Default, Debug)]
struct YamlObject {
    file_id: i64,
    type_tag: String,
    fields: BTreeMap<String, String>,
}

#[derive(Default, Serialize)]
struct Body {
    name: String,
    parent: Option<String>,
    mass_1e24_kg: Option<f64>,
    radius_km: Option<f64>,
    semi_major_axis_au: Option<f64>,
    eccentricity: Option<f64>,
    inclination_deg: Option<f64>,
    perihelion_au: Option<f64>,
    longitude_deg: Option<f64>,
    omega_lc_deg: Option<f64>,
    omega_uc_deg: Option<f64>,
    body_type: Option<i64>,
    orbit_data_source: Option<&'static str>,
}

#[derive(Serialize)]
struct Stats {
    source: String,
    bodies: Vec<Body>,
}

fn parse_objects_from_yaml(yaml: &str) -> Vec<YamlObject> {
    let mut objects: Vec<YamlObject> = Vec::new();
    let mut current: Option<YamlObject> = None;

    for line in yaml.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            if let Some(obj) = current.take() {
                objects.push(obj);
            }
            let mut tag = String::new();
            let mut file_id: i64 = 0;
            if let Some(amp_idx) = rest.find('&') {
                tag = rest[..amp_idx].trim().to_string();
                file_id = rest[amp_idx + 1..].trim().parse().unwrap_or(0);
            }
            current = Some(YamlObject {
                file_id,
                type_tag: tag,
                fields: BTreeMap::new(),
            });
            continue;
        }
        let obj = match current.as_mut() {
            Some(o) => o,
            None => continue,
        };
        if let Some((key, value)) = parse_top_level_field(line) {
            obj.fields.insert(key, value);
        }
    }
    if let Some(obj) = current.take() {
        objects.push(obj);
    }
    objects
}

fn parse_top_level_field(line: &str) -> Option<(String, String)> {
    if !line.starts_with("  ") || line.starts_with("    ") {
        return None;
    }
    let trimmed = &line[2..];
    let colon = trimmed.find(':')?;
    let key = trimmed[..colon].to_string();
    let value = trimmed[colon + 1..].trim().to_string();
    Some((key, value))
}

fn guid_from_pptr(value: &str) -> Option<String> {
    let inner = value.trim().strip_prefix('{')?.strip_suffix('}')?;
    for pair in inner.split(',') {
        let mut it = pair.splitn(2, ':');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        if k == "guid" {
            return Some(v.to_string());
        }
    }
    None
}

fn pptr_file_id(value: &str) -> Option<i64> {
    let inner = value.trim().strip_prefix('{')?.strip_suffix('}')?;
    for pair in inner.split(',') {
        let mut it = pair.splitn(2, ':');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        if k == "fileID" {
            return v.parse().ok();
        }
    }
    None
}

fn parse_f64(s: &str) -> Option<f64> {
    s.parse().ok()
}

fn parse_i64(s: &str) -> Option<i64> {
    s.parse().ok()
}

fn extract_from_yaml(yaml: &str) -> Vec<Body> {
    let objects = parse_objects_from_yaml(yaml);

    let mut gameobject_name: BTreeMap<i64, String> = BTreeMap::new();
    let mut component_owner: BTreeMap<i64, i64> = BTreeMap::new();
    for obj in &objects {
        if obj.type_tag == "!u!1" {
            if let Some(name) = obj.fields.get("m_Name") {
                gameobject_name.insert(obj.file_id, name.clone());
            }
        }
    }

    let mut bodies: BTreeMap<i64, Body> = BTreeMap::new();
    let mut nbody_owner: BTreeMap<i64, i64> = BTreeMap::new();

    for obj in &objects {
        if obj.type_tag != "!u!114" {
            continue;
        }
        let script = match obj.fields.get("m_Script") {
            Some(s) => s,
            None => continue,
        };
        let guid = match guid_from_pptr(script) {
            Some(g) => g,
            None => continue,
        };
        let go_pptr = match obj.fields.get("m_GameObject") {
            Some(g) => g,
            None => continue,
        };
        let go_id = match pptr_file_id(go_pptr) {
            Some(id) if id != 0 => id,
            _ => continue,
        };
        component_owner.insert(obj.file_id, go_id);

        let body = bodies.entry(go_id).or_insert_with(|| Body {
            name: gameobject_name.get(&go_id).cloned().unwrap_or_default(),
            ..Default::default()
        });

        match guid.as_str() {
            GUID_SOLAR_BODY => {
                body.body_type = obj.fields.get("bodyType").and_then(|v| parse_i64(v));
                body.mass_1e24_kg = obj.fields.get("mass_1E24").and_then(|v| parse_f64(v));
                body.radius_km = obj.fields.get("radiusKm").and_then(|v| parse_f64(v));
                body.semi_major_axis_au = obj.fields.get("a").and_then(|v| parse_f64(v));
                body.eccentricity = obj.fields.get("ecc").and_then(|v| parse_f64(v));
                body.inclination_deg = obj.fields.get("inclination").and_then(|v| parse_f64(v));
                body.perihelion_au = obj.fields.get("p").and_then(|v| parse_f64(v));
                body.longitude_deg = obj.fields.get("longitude").and_then(|v| parse_f64(v));
                body.omega_lc_deg = obj.fields.get("omega_lc").and_then(|v| parse_f64(v));
                body.omega_uc_deg = obj.fields.get("omega_uc").and_then(|v| parse_f64(v));
                body.orbit_data_source = Some("SolarBody");
            }
            GUID_NBODY => {
                nbody_owner.insert(obj.file_id, go_id);
                let mass = obj.fields.get("mass").and_then(|v| parse_f64(v));
                if body.mass_1e24_kg.unwrap_or(0.0) == 0.0 {
                    body.mass_1e24_kg = mass;
                }
            }
            GUID_ORBIT_UNIVERSAL => {
                let p = obj.fields.get("p").and_then(|v| parse_f64(v));
                let ecc = obj.fields.get("eccentricity").and_then(|v| parse_f64(v));
                let inc = obj.fields.get("inclination").and_then(|v| parse_f64(v));
                let center = obj.fields.get("centerNbody").and_then(|v| pptr_file_id(v));
                if body.orbit_data_source.is_none() {
                    body.perihelion_au = p;
                    body.eccentricity = ecc;
                    body.inclination_deg = inc;
                    body.omega_lc_deg = obj.fields.get("omega_lc").and_then(|v| parse_f64(v));
                    body.omega_uc_deg = obj.fields.get("omega_uc").and_then(|v| parse_f64(v));
                    if let (Some(p), Some(e)) = (p, ecc) {
                        if e < 1.0 {
                            body.semi_major_axis_au = Some(p / (1.0 - e));
                        }
                    }
                    body.orbit_data_source = Some("OrbitUniversal");
                }
                if let Some(c) = center {
                    body.parent = Some(format!("__nbody_fid_{c}"));
                }
            }
            _ => {}
        }
    }

    for body in bodies.values_mut() {
        if let Some(parent_ref) = body.parent.clone() {
            if let Some(nbody_fid) = parent_ref.strip_prefix("__nbody_fid_") {
                let nbody_fid: i64 = nbody_fid.parse().unwrap_or(0);
                if let Some(go_id) = nbody_owner.get(&nbody_fid) {
                    body.parent = gameobject_name.get(go_id).cloned();
                } else {
                    body.parent = None;
                }
            }
        }
    }

    let mut result: Vec<Body> = bodies
        .into_values()
        .filter(|b| !b.name.is_empty())
        .collect();
    result.sort_by(|a, b| {
        a.semi_major_axis_au
            .unwrap_or(0.0)
            .partial_cmp(&b.semi_major_axis_au.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.name.cmp(&b.name))
    });
    result
}

fn extract_from_scene(scene_path: &Path) -> Result<Vec<Body>> {
    let yaml = fs::read_to_string(scene_path)
        .with_context(|| format!("reading {}", scene_path.display()))?;
    Ok(extract_from_yaml(&yaml))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mercury_yaml() -> &'static str {
        "\
%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!1 &27
GameObject:
  serializedVersion: 6
  m_Component:
  - component: {fileID: 100}
  m_Name: Mercury
--- !u!114 &100
MonoBehaviour:
  m_GameObject: {fileID: 27}
  m_Enabled: 1
  m_Script: {fileID: 11500000, guid: ae426645999213f30e86342528c32628, type: 3}
  ecc: 0.2056
  a: 0.3870993
  p: 0.30750022
  omega_uc: 48.331
  omega_lc: 29.124
  inclination: 7.005
  longitude: 174.79
  mass_1E24: 0.3301
  radiusKm: 2439.7
  bodyType: 0
"
    }

    #[test]
    fn parses_block_headers_without_trailing_space() {
        let objs = parse_objects_from_yaml("--- !u!1 &42\nGameObject:\n  m_Name: Foo\n");
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].type_tag, "!u!1");
        assert_eq!(objs[0].file_id, 42);
        assert_eq!(objs[0].fields.get("m_Name").map(|s| s.as_str()), Some("Foo"));
    }

    #[test]
    fn extracts_planet_from_solar_body() {
        let bodies = extract_from_yaml(mercury_yaml());
        assert_eq!(bodies.len(), 1, "expected one body, got {:?}", bodies.iter().map(|b| &b.name).collect::<Vec<_>>());
        let mercury = &bodies[0];
        assert_eq!(mercury.name, "Mercury");
        assert_eq!(mercury.body_type, Some(0));
        assert!((mercury.semi_major_axis_au.unwrap() - 0.3870993).abs() < 1e-5);
        assert!((mercury.eccentricity.unwrap() - 0.2056).abs() < 1e-5);
        assert!((mercury.inclination_deg.unwrap() - 7.005).abs() < 1e-5);
        assert!((mercury.mass_1e24_kg.unwrap() - 0.3301).abs() < 1e-5);
        assert!((mercury.radius_km.unwrap() - 2439.7).abs() < 1e-3);
        assert_eq!(mercury.orbit_data_source.as_deref(), Some("SolarBody"));
    }

    #[test]
    fn parses_pptr_fields() {
        assert_eq!(pptr_file_id("{fileID: 27}"), Some(27));
        assert_eq!(pptr_file_id("{fileID: -806885394, guid: abc, type: 3}"), Some(-806885394));
        assert_eq!(
            guid_from_pptr("{fileID: 11500000, guid: ae426645999213f30e86342528c32628, type: 3}").as_deref(),
            Some("ae426645999213f30e86342528c32628"),
        );
    }

    #[test]
    fn joins_moon_to_parent_via_orbit_universal() {
        let yaml = "\
--- !u!1 &10
GameObject:
  m_Name: Earth
--- !u!1 &20
GameObject:
  m_Name: Moon
--- !u!114 &101
MonoBehaviour:
  m_GameObject: {fileID: 10}
  m_Script: {fileID: 11500000, guid: bb2f3ea79264b8a4ef2ff2823a0af4d7, type: 3}
  mass: 5.972
--- !u!114 &201
MonoBehaviour:
  m_GameObject: {fileID: 20}
  m_Script: {fileID: 11500000, guid: bb2f3ea79264b8a4ef2ff2823a0af4d7, type: 3}
  mass: 0.07342
--- !u!114 &202
MonoBehaviour:
  m_GameObject: {fileID: 20}
  m_Script: {fileID: 11500000, guid: 34376944fc28cccd32412aa34287d642, type: 3}
  evolveMode: 0
  p: 2.57
  p_inspector: 2.57
  eccentricity: 0
  inclination: 5.145
  omega_uc: 0
  omega_lc: 0
  centerNbody: {fileID: 101}
";
        let bodies = extract_from_yaml(yaml);
        let moon = bodies.iter().find(|b| b.name == "Moon").expect("Moon body missing");
        assert_eq!(moon.parent.as_deref(), Some("Earth"));
        assert!((moon.mass_1e24_kg.unwrap() - 0.07342).abs() < 1e-5);
        assert_eq!(moon.orbit_data_source.as_deref(), Some("OrbitUniversal"));
        // semi_major_axis_au = p / (1 - e), with e=0 ⇒ 2.57
        assert!((moon.semi_major_axis_au.unwrap() - 2.57).abs() < 1e-5);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!("usage: parse-stats <MyScene.unity> <out.json>"));
    }
    let scene = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let bodies = extract_from_scene(&scene)?;
    let stats = Stats {
        source: scene.display().to_string(),
        bodies,
    };

    serde_json::to_writer_pretty(fs::File::create(&output)?, &stats)?;
    eprintln!("wrote {} ({} bodies)", output.display(), stats.bodies.len());
    Ok(())
}

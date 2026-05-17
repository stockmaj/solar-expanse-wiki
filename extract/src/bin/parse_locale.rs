use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize)]
struct NameDesc {
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Corporation {
    id: String,
    name: String,
    description: String,
    traits: String,
}

#[derive(Serialize)]
struct ResourceEntry {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct Facility {
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct CelestialBody {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct ResearchEntry {
    id: String,
    category: &'static str,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Locale {
    source: String,
    total_keys: usize,
    celestial_bodies: Vec<CelestialBody>,
    spacecraft: Vec<NameDesc>,
    launch_vehicles: Vec<NameDesc>,
    cargo: Vec<NameDesc>,
    research: Vec<ResearchEntry>,
    corporations: Vec<Corporation>,
    contracts: Vec<NameDesc>,
    resources: Vec<ResourceEntry>,
    facilities: Vec<Facility>,
    habitability_scales: BTreeMap<String, Vec<String>>,
}

fn load_keys(path: &PathBuf) -> Result<BTreeMap<String, String>> {
    let mut keys = BTreeMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("opening {}", path.display()))?;
    for result in reader.records() {
        let record = result?;
        let mut it = record.iter();
        let key = match it.next() {
            Some(k) => k.to_string(),
            None => continue,
        };
        let value: Vec<&str> = it.collect();
        keys.insert(key, value.join(","));
    }
    Ok(keys)
}

fn pair_name_description(keys: &BTreeMap<String, String>, prefix: &str) -> Vec<NameDesc> {
    let mut out: Vec<NameDesc> = Vec::new();
    for (k, v) in keys {
        if !k.starts_with(prefix) || k.ends_with("_Description") {
            continue;
        }
        let desc = keys.get(&format!("{k}_Description")).cloned().unwrap_or_default();
        out.push(NameDesc {
            id: k.clone(),
            name: v.clone(),
            description: desc,
        });
    }
    out
}

fn celestial_bodies(keys: &BTreeMap<String, String>) -> Vec<CelestialBody> {
    keys.iter()
        .filter_map(|(k, v)| {
            k.strip_prefix("CelestialBodiesNames.").map(|id| CelestialBody {
                id: id.to_string(),
                name: v.clone(),
            })
        })
        .collect()
}

fn research(keys: &BTreeMap<String, String>) -> Vec<ResearchEntry> {
    let mut out = Vec::new();
    for (k, v) in keys {
        if let Some(base) = k.strip_suffix("_Title").filter(|s| s.starts_with("research_")) {
            let fluff = keys.get(&format!("{base}_fluff")).cloned().unwrap_or_default();
            let category = if base.contains("_category_") { "category" } else { "topic" };
            out.push(ResearchEntry {
                id: base.to_string(),
                category,
                name: v.clone(),
                description: fluff,
            });
        }
    }
    out
}

fn corporations(keys: &BTreeMap<String, String>) -> Vec<Corporation> {
    const PREFIX: &str = "Game.UI.CustomizationScreen.CorporationInfo.Item";
    let mut indices: Vec<i32> = keys
        .keys()
        .filter_map(|k| {
            k.strip_prefix(PREFIX)
                .and_then(|rest| rest.strip_suffix(".Name"))
                .and_then(|n| n.parse().ok())
        })
        .collect();
    indices.sort();
    indices
        .into_iter()
        .map(|i| Corporation {
            id: format!("Item{i}"),
            name: keys.get(&format!("{PREFIX}{i}.Name")).cloned().unwrap_or_default(),
            description: keys
                .get(&format!("{PREFIX}{i}.Description"))
                .cloned()
                .unwrap_or_default(),
            traits: keys
                .get(&format!("{PREFIX}{i}.TraitsList"))
                .cloned()
                .unwrap_or_default(),
        })
        .collect()
}

fn contracts(keys: &BTreeMap<String, String>) -> Vec<NameDesc> {
    let mut out = Vec::new();
    for (k, v) in keys {
        if let Some(base) = k
            .strip_suffix("_Title")
            .filter(|s| s.starts_with("contract_"))
        {
            out.push(NameDesc {
                id: base.to_string(),
                name: v.clone(),
                description: keys.get(&format!("{base}_fluff")).cloned().unwrap_or_default(),
            });
        }
    }
    out
}

fn resources(keys: &BTreeMap<String, String>) -> Vec<ResourceEntry> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for k in keys.keys() {
        let ident = k
            .strip_prefix("ToolTip_id_resource_")
            .or_else(|| k.strip_prefix("id_resource_"));
        if let Some(ident) = ident {
            if !seen.insert(ident.to_string()) {
                continue;
            }
            let name = keys
                .get(&format!("ToolTip_id_resource_{ident}"))
                .or_else(|| keys.get(&format!("id_resource_{ident}")))
                .cloned()
                .unwrap_or_else(|| ident.to_string());
            out.push(ResourceEntry {
                id: ident.to_string(),
                name,
            });
        }
    }
    out
}

fn facilities(keys: &BTreeMap<String, String>) -> Vec<Facility> {
    // Two key shapes exist for facility text:
    //   build_<id>             → short display name (e.g. "OUTPOST")
    //   build_<id>_Description → long blurb
    //   ToolTip_build_<id>     → tooltip text (fallback when no _Description exists)
    let mut out: Vec<Facility> = Vec::new();
    for (k, v) in keys {
        if k.starts_with("ToolTip_") || k.ends_with("_Description") || k.ends_with("_fluff") {
            continue;
        }
        let id = match k.strip_prefix("build_") {
            Some(rest) => rest.to_string(),
            None => continue,
        };
        if v.is_empty() {
            continue;
        }
        let desc = keys
            .get(&format!("build_{id}_Description"))
            .or_else(|| keys.get(&format!("ToolTip_build_{id}")))
            .cloned()
            .unwrap_or_default();
        out.push(Facility {
            id,
            name: v.clone(),
            description: desc,
        });
    }
    out
}

fn habitability_scales(keys: &BTreeMap<String, String>) -> BTreeMap<String, Vec<String>> {
    let base = "Tooltip.UIBasicInfoPlanetCharacteristic";
    let mut out = BTreeMap::new();
    for attr in ["Temperature", "Atmosphere", "Gravitation", "Radiation"] {
        let labels: Vec<String> = (0..10)
            .filter_map(|i| keys.get(&format!("{base}.{attr}{i}")).cloned())
            .filter(|s| !s.is_empty())
            .collect();
        if !labels.is_empty() {
            out.insert(attr.to_lowercase(), labels);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn load(csv: &str) -> BTreeMap<String, String> {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.as_file().write_all(csv.as_bytes()).unwrap();
        tmp.as_file().sync_all().unwrap();
        load_keys(&tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn celestial_bodies_extracts_name_under_prefix() {
        let keys = load("CelestialBodiesNames.Mercury,Mercury\nCelestialBodiesNames.Moon,Luna\nUI.Other,nope\n");
        let bodies = celestial_bodies(&keys);
        let names: Vec<&str> = bodies.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"Mercury"));
        assert!(names.contains(&"Luna"));
        assert_eq!(bodies.iter().find(|b| b.id == "Moon").unwrap().name, "Luna");
        assert_eq!(bodies.len(), 2);
    }

    #[test]
    fn pair_name_description_pairs_id_with_description() {
        let keys = load(
            "spacecraft_iris,Iris\nspacecraft_iris_Description,Simple probe craft.\nspacecraft_zeus,Zeus\n",
        );
        let items = pair_name_description(&keys, "spacecraft_");
        let iris = items.iter().find(|x| x.id == "spacecraft_iris").unwrap();
        assert_eq!(iris.name, "Iris");
        assert_eq!(iris.description, "Simple probe craft.");
        let zeus = items.iter().find(|x| x.id == "spacecraft_zeus").unwrap();
        assert_eq!(zeus.name, "Zeus");
        assert_eq!(zeus.description, "");
    }

    #[test]
    fn corporations_walks_indexed_items() {
        let keys = load(
            "Game.UI.CustomizationScreen.CorporationInfo.Item0.Name,SoleX\n\
             Game.UI.CustomizationScreen.CorporationInfo.Item0.Description,A private firm.\n\
             Game.UI.CustomizationScreen.CorporationInfo.Item0.TraitsList,● Reusable rockets\n\
             Game.UI.CustomizationScreen.CorporationInfo.Item2.Name,ESA\n\
             Game.UI.CustomizationScreen.CorporationInfo.Item2.Description,European agency.\n",
        );
        let corps = corporations(&keys);
        let names: Vec<&str> = corps.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["SoleX", "ESA"]);
        assert_eq!(corps[0].traits, "● Reusable rockets");
        assert_eq!(corps[1].traits, "");
    }

    #[test]
    fn research_distinguishes_categories_from_topics() {
        let keys = load(
            "research_category_chem_Title,Chemical Propulsion\n\
             research_category_chem_fluff,Burn stuff.\n\
             research_chem_main1_Title,Solid Propellant\n\
             research_chem_main1_fluff,Light fire.\n",
        );
        let r = research(&keys);
        let cat = r.iter().find(|x| x.name == "Chemical Propulsion").unwrap();
        let topic = r.iter().find(|x| x.name == "Solid Propellant").unwrap();
        assert_eq!(cat.category, "category");
        assert_eq!(topic.category, "topic");
        assert_eq!(cat.description, "Burn stuff.");
    }

    #[test]
    fn resources_dedupes_id_resource_and_tooltip_id_resource() {
        let keys = load("id_resource_water,Water\nToolTip_id_resource_water,Water\nid_resource_iron,Iron\n");
        let r = resources(&keys);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn habitability_scales_collects_temperature_ladder() {
        let keys = load(
            "Tooltip.UIBasicInfoPlanetCharacteristic.Temperature1,Extremely Cold\n\
             Tooltip.UIBasicInfoPlanetCharacteristic.Temperature2,Cold\n\
             Tooltip.UIBasicInfoPlanetCharacteristic.Temperature3,Temperate\n",
        );
        let scales = habitability_scales(&keys);
        assert_eq!(scales["temperature"], vec!["Extremely Cold", "Cold", "Temperate"]);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!("usage: parse-locale <en-US.csv> <out.json>"));
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let keys = load_keys(&input)?;
    let locale = Locale {
        source: input.display().to_string(),
        total_keys: keys.len(),
        celestial_bodies: celestial_bodies(&keys),
        spacecraft: pair_name_description(&keys, "spacecraft_"),
        launch_vehicles: pair_name_description(&keys, "lv_"),
        cargo: pair_name_description(&keys, "cargo_"),
        research: research(&keys),
        corporations: corporations(&keys),
        contracts: contracts(&keys),
        resources: resources(&keys),
        facilities: facilities(&keys),
        habitability_scales: habitability_scales(&keys),
    };

    serde_json::to_writer_pretty(std::fs::File::create(&output)?, &locale)?;
    eprintln!(
        "wrote {} ({} keys → {} celestial bodies, {} spacecraft, {} research, {} contracts)",
        output.display(),
        locale.total_keys,
        locale.celestial_bodies.len(),
        locale.spacecraft.len(),
        locale.research.len(),
        locale.contracts.len(),
    );
    Ok(())
}

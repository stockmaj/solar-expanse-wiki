use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
struct Sirenix {
    #[serde(default)]
    facilities: Vec<FacilityStat>,
    #[serde(default)]
    research: Vec<ResearchStat>,
    #[serde(default)]
    resources: Vec<ResourceStat>,
}

#[derive(Deserialize, Clone)]
struct FacilityStat {
    id: String,
    descriptor: String,
    facility_type: String,
    #[serde(default)]
    build_cost: Vec<ResourceCost>,
}

#[derive(Deserialize, Clone)]
struct ResearchStat {
    id: String,
    action: String,
    #[serde(default)]
    bonus_kind: Option<String>,
    #[serde(default)]
    bonus_amount: f64,
    #[serde(default)]
    bonus_components: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct ResourceStat {
    id: String,
}

#[derive(Deserialize, Clone)]
struct ResourceCost {
    resource_id: String,
    amount: f64,
}

#[derive(Deserialize, Default)]
struct Locale {
    #[serde(default)]
    facilities: Vec<LocaleEntry>,
    #[serde(default)]
    research: Vec<LocaleEntry>,
    #[serde(default)]
    resources: Vec<LocaleEntry>,
}

#[derive(Deserialize, Clone)]
struct LocaleEntry {
    id: String,
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalculatorData {
    facilities: Vec<CalcFacility>,
    resources: Vec<CalcResource>,
    reductions: Vec<CalcReduction>,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcFacility {
    id: String,
    name: String,
    category: String,
    build_cost: Vec<CalcCost>,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcCost {
    resource: String,
    amount: f64,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcResource {
    id: String,
    name: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcReduction {
    id: String,
    name: String,
    kind: String,
    percent: f64,
    affects: Vec<String>,
    affects_all: bool,
}

fn smart_title_case(s: &str) -> String {
    let alpha: String = s.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.is_empty() || alpha.chars().any(|c| c.is_lowercase()) {
        return s.to_string();
    }
    let lower = s.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut capitalize_next = true;
    for c in lower.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                for u in c.to_uppercase() {
                    out.push(u);
                }
                capitalize_next = false;
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
            capitalize_next = c.is_whitespace() || c == '/' || c == '-' || c == '(';
        }
    }
    out
}

fn build_calculator_data(sirenix: &Sirenix, locale: &Locale) -> CalculatorData {
    let facility_name = |id: &str| -> Option<&str> {
        // Sirenix facility ids carry a `build_` prefix that the locale strips.
        let key = id.strip_prefix("build_").unwrap_or(id);
        locale
            .facilities
            .iter()
            .find(|f| f.id == key)
            .map(|f| f.name.as_str())
    };

    let mut facilities: Vec<CalcFacility> = Vec::new();
    for f in &sirenix.facilities {
        if f.facility_type == "FacilitySegment" {
            continue;
        }
        if f.descriptor == "Orbital" {
            continue;
        }
        let raw_name = match facility_name(&f.id) {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };
        let name = smart_title_case(raw_name);
        let build_cost = f
            .build_cost
            .iter()
            .map(|c| CalcCost {
                resource: c.resource_id.clone(),
                amount: c.amount,
            })
            .collect();
        facilities.push(CalcFacility {
            id: f.id.clone(),
            name,
            category: f.facility_type.clone(),
            build_cost,
        });
    }

    let used_resources: BTreeSet<&str> = facilities
        .iter()
        .flat_map(|f| f.build_cost.iter().map(|c| c.resource.as_str()))
        .collect();

    let resource_name = |id: &str| -> Option<&str> {
        locale
            .resources
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.as_str())
    };

    let mut resources: Vec<CalcResource> = Vec::new();
    for r in &sirenix.resources {
        if !used_resources.contains(r.id.as_str()) {
            continue;
        }
        let name = resource_name(&r.id).unwrap_or(&r.id).to_string();
        resources.push(CalcResource {
            id: r.id.clone(),
            name,
        });
    }

    let relevant_kinds = ["BuildCost", "PowerProduction", "ReduceCrewRequirements"];
    let research_name = |id: &str| -> Option<&str> {
        locale
            .research
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.as_str())
    };

    let mut reductions: Vec<CalcReduction> = Vec::new();
    for r in &sirenix.research {
        if r.action != "UnlockBonus" {
            continue;
        }
        let Some(kind) = r.bonus_kind.as_deref() else {
            continue;
        };
        if !relevant_kinds.contains(&kind) {
            continue;
        }
        let name = research_name(&r.id).unwrap_or(&r.id).to_string();
        // Some research entries use the literal sentinel ["All"] instead of an
        // empty list to mean "applies to every facility of this kind". Treat
        // both the same.
        let is_all_sentinel = r.bonus_components.len() == 1 && r.bonus_components[0] == "All";
        let affects_all = r.bonus_components.is_empty() || is_all_sentinel;
        let affects = if affects_all { Vec::new() } else { r.bonus_components.clone() };
        reductions.push(CalcReduction {
            id: r.id.clone(),
            name,
            kind: kind.to_string(),
            percent: r.bonus_amount.abs(),
            affects,
            affects_all,
        });
    }

    CalculatorData {
        facilities,
        resources,
        reductions,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "usage: {} <sirenix.json> <locale.json> <out.json>",
            args.first().map(String::as_str).unwrap_or("gen-calculator-data")
        );
        std::process::exit(2);
    }
    let sirenix_path = Path::new(&args[1]);
    let locale_path = Path::new(&args[2]);
    let out_path = Path::new(&args[3]);

    let sirenix_text = fs::read_to_string(sirenix_path)
        .with_context(|| format!("reading {}", sirenix_path.display()))?;
    let sirenix: Sirenix = serde_json::from_str(&sirenix_text)
        .with_context(|| format!("parsing {}", sirenix_path.display()))?;

    let locale_text = fs::read_to_string(locale_path)
        .with_context(|| format!("reading {}", locale_path.display()))?;
    let locale: Locale = serde_json::from_str(&locale_text)
        .with_context(|| format!("parsing {}", locale_path.display()))?;

    let data = build_calculator_data(&sirenix, &locale);

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&data)?;
    fs::write(out_path, json).with_context(|| format!("writing {}", out_path.display()))?;

    println!(
        "wrote {} facilities, {} resources, {} reductions -> {}",
        data.facilities.len(),
        data.resources.len(),
        data.reductions.len(),
        out_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_json<T: for<'de> Deserialize<'de>>(v: serde_json::Value) -> T {
        serde_json::from_value(v).expect("fixture should parse")
    }

    fn locale_with(
        facilities: Vec<(&str, &str)>,
        research: Vec<(&str, &str)>,
        resources: Vec<(&str, &str)>,
    ) -> Locale {
        Locale {
            facilities: facilities
                .into_iter()
                .map(|(id, name)| LocaleEntry {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: String::new(),
                })
                .collect(),
            research: research
                .into_iter()
                .map(|(id, name)| LocaleEntry {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: String::new(),
                })
                .collect(),
            resources: resources
                .into_iter()
                .map(|(id, name)| LocaleEntry {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn facility_with_multiple_resources_passes_through() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_alloymine",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Mining",
                "build_cost": [
                    {"resource_id": "metal", "amount": 125.0},
                    {"resource_id": "chips", "amount": 50.0}
                ]
            }],
            "resources": [
                {"id": "metal"},
                {"id": "chips"}
            ]
        }));
        let locale = locale_with(
            vec![("alloymine", "EXOTIC ALLOY EXTRACTOR")],
            vec![],
            vec![("metal", "Metals"), ("chips", "Electronics")],
        );

        let data = build_calculator_data(&sirenix, &locale);

        assert_eq!(data.facilities.len(), 1);
        let f = &data.facilities[0];
        assert_eq!(f.id, "build_alloymine");
        assert_eq!(f.name, "Exotic Alloy Extractor");
        assert_eq!(f.category, "Mining");
        assert_eq!(
            f.build_cost,
            vec![
                CalcCost { resource: "metal".to_string(), amount: 125.0 },
                CalcCost { resource: "chips".to_string(), amount: 50.0 }
            ]
        );
    }

    #[test]
    fn facility_segment_is_excluded() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_intership_segment",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "FacilitySegment",
                "build_cost": [{"resource_id": "metal", "amount": 10.0}]
            }],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("intership_segment", "SHIP SEGMENT")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert!(data.facilities.is_empty());
    }

    #[test]
    fn orbital_descriptor_facility_is_excluded() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_payload_engine",
                "descriptor": "Orbital",
                "placement": "Orbit",
                "facility_type": "Module",
                "build_cost": [{"resource_id": "metal", "amount": 10.0}]
            }],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("payload_engine", "PAYLOAD ENGINE")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert!(data.facilities.is_empty());
    }

    #[test]
    fn buildcost_research_becomes_reduction_with_abs_percent() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "research": [{
                "id": "research_lifesup_10",
                "action": "UnlockBonus",
                "bonus_kind": "BuildCost",
                "bonus_amount": -35.0,
                "bonus_components": ["build_outpost", "build_habitat"]
            }]
        }));
        let locale = locale_with(
            vec![],
            vec![("research_lifesup_10", "Regolith Shielding")],
            vec![],
        );

        let data = build_calculator_data(&sirenix, &locale);

        assert_eq!(data.reductions.len(), 1);
        let r = &data.reductions[0];
        assert_eq!(r.id, "research_lifesup_10");
        assert_eq!(r.name, "Regolith Shielding");
        assert_eq!(r.kind, "BuildCost");
        assert_eq!(r.percent, 35.0);
        assert_eq!(r.affects, vec!["build_outpost", "build_habitat"]);
        assert!(!r.affects_all);
    }

    #[test]
    fn empty_bonus_components_marks_affects_all() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "research": [{
                "id": "research_global",
                "action": "UnlockBonus",
                "bonus_kind": "PowerProduction",
                "bonus_amount": 10.0,
                "bonus_components": []
            }]
        }));
        let locale = locale_with(
            vec![],
            vec![("research_global", "Better Power")],
            vec![],
        );

        let data = build_calculator_data(&sirenix, &locale);

        assert_eq!(data.reductions.len(), 1);
        assert!(data.reductions[0].affects_all);
        assert!(data.reductions[0].affects.is_empty());
    }

    #[test]
    fn all_sentinel_bonus_components_marks_affects_all() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "research": [{
                "id": "research_robotics_crew1",
                "action": "UnlockBonus",
                "bonus_kind": "ReduceCrewRequirements",
                "bonus_amount": 5.0,
                "bonus_components": ["All"]
            }]
        }));
        let locale = locale_with(
            vec![],
            vec![("research_robotics_crew1", "Exoskeletons")],
            vec![],
        );

        let data = build_calculator_data(&sirenix, &locale);

        assert_eq!(data.reductions.len(), 1);
        assert!(data.reductions[0].affects_all);
        assert!(data.reductions[0].affects.is_empty());
    }

    #[test]
    fn irrelevant_bonus_kind_is_excluded() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "research": [{
                "id": "research_engine",
                "action": "UnlockBonus",
                "bonus_kind": "ComponentExhaustV",
                "bonus_amount": 10.0,
                "bonus_components": []
            }]
        }));
        let locale = locale_with(
            vec![],
            vec![("research_engine", "Better Engine")],
            vec![],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert!(data.reductions.is_empty());
    }

    #[test]
    fn unused_resource_is_dropped() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_alloymine",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Mining",
                "build_cost": [{"resource_id": "metal", "amount": 125.0}]
            }],
            "resources": [
                {"id": "metal"},
                {"id": "unobtainium"}
            ]
        }));
        let locale = locale_with(
            vec![("alloymine", "EXOTIC ALLOY EXTRACTOR")],
            vec![],
            vec![("metal", "Metals"), ("unobtainium", "Unobtainium")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert_eq!(data.resources.len(), 1);
        assert_eq!(data.resources[0].id, "metal");
        assert_eq!(data.resources[0].name, "Metals");
    }

    #[test]
    fn facility_with_missing_locale_name_is_skipped() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [
                {
                    "id": "build_alloymine",
                    "descriptor": "Ground",
                    "placement": "Surface",
                    "facility_type": "Mining",
                    "build_cost": [{"resource_id": "metal", "amount": 125.0}]
                },
                {
                    "id": "build_ghost",
                    "descriptor": "Ground",
                    "placement": "Surface",
                    "facility_type": "Mining",
                    "build_cost": [{"resource_id": "metal", "amount": 1.0}]
                }
            ],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("alloymine", "EXOTIC ALLOY EXTRACTOR")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert_eq!(data.facilities.len(), 1);
        assert_eq!(data.facilities[0].id, "build_alloymine");
        assert_eq!(data.facilities[0].name, "Exotic Alloy Extractor");
    }
}

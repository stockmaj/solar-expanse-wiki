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
    #[serde(default)]
    crew_transports: Vec<CrewTransportStat>,
    #[serde(default)]
    spacecraft: Vec<SpacecraftStat>,
    #[serde(default)]
    space_modules: Vec<SpaceModuleStat>,
}

#[derive(Deserialize, Clone)]
struct SpaceModuleStat {
    id: String,
    mass: f64,
    #[serde(default)]
    special_ability: String,
    #[serde(default)]
    is_locked: bool,
    #[serde(default)]
    can_be_load_as_cargo: bool,
}

#[derive(Deserialize, Clone)]
struct CrewTransportStat {
    id: String,
    capacity: i64,
    mass: f64,
    is_locked: bool,
}

#[derive(Deserialize, Clone)]
struct SpacecraftStat {
    id: String,
    cargo_capacity: f64,
    #[serde(default)]
    can_be_built_by_player: bool,
}

#[derive(Deserialize, Clone)]
struct FacilityStat {
    id: String,
    descriptor: String,
    facility_type: String,
    #[serde(default)]
    build_cost: Vec<ResourceCost>,
    #[serde(default)]
    workers_required: i64,
    #[serde(default)]
    energy_consumption: f64,
    #[serde(default)]
    produces: Vec<ResourceCost>,
    #[serde(default)]
    build_time_days: f64,
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
    #[serde(default)]
    spacecraft: Vec<LocaleEntry>,
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
    crew_transports: Vec<CalcCrewTransport>,
    spacecraft: Vec<CalcSpacecraft>,
    space_modules: Vec<CalcSpaceModule>,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcSpaceModule {
    id: String,
    name: String,
    /// Role tag for grouping in the UI (Mining / Power / Probe / …).
    category: String,
    /// Dry mass in tons — what gets lifted.
    mass: f64,
    is_locked: bool,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcSpacecraft {
    id: String,
    name: String,
    /// Cargo tons per trip.
    cargo_capacity: f64,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcCrewTransport {
    id: String,
    name: String,
    /// Humans per loaded module.
    capacity: i64,
    /// Dry mass per module, in tons. Each carried human adds 1 t on top.
    mass: f64,
    is_locked: bool,
}

#[derive(Serialize, Debug, PartialEq)]
struct CalcFacility {
    id: String,
    name: String,
    category: String,
    build_cost: Vec<CalcCost>,
    workers_required: i64,
    energy_consumption: f64,
    power_production: f64,
    build_time_days: f64,
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
        let power_production = f
            .produces
            .iter()
            .find(|p| p.resource_id == "energy")
            .map(|p| p.amount)
            .unwrap_or(0.0);
        facilities.push(CalcFacility {
            id: f.id.clone(),
            name,
            category: f.facility_type.clone(),
            build_cost,
            workers_required: f.workers_required,
            energy_consumption: f.energy_consumption,
            power_production,
            build_time_days: f.build_time_days,
        });
    }

    // Disambiguate `_big` variants that share a display name with the basic
    // facility. The game gives the late-game variant the same in-game name;
    // appending "(Advanced)" lets the calculator distinguish them.
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for f in &facilities {
        *name_counts.entry(f.name.clone()).or_insert(0) += 1;
    }
    for f in facilities.iter_mut() {
        if f.id.ends_with("_big") && name_counts.get(&f.name).copied().unwrap_or(0) > 1 {
            f.name = format!("{} (Advanced)", f.name);
        }
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

    let crew_transports: Vec<CalcCrewTransport> = sirenix
        .crew_transports
        .iter()
        .map(|c| CalcCrewTransport {
            id: c.id.clone(),
            name: crew_display_name(&c.id),
            capacity: c.capacity,
            mass: c.mass,
            is_locked: c.is_locked,
        })
        .collect();

    // Interplanetary spacecraft with a meaningful cargo capacity. The dump
    // uses 999999 as a sentinel for payload containers (and a stray
    // `_ForAsteroidImpact` test entry); those aren't real ships. The
    // `can_be_built_by_player` flag in the dump is unreliable — false for
    // Hermes, Centaur, Atlas, which players absolutely use — so we ignore it
    // and trust the cargo-capacity bound + a missing-name skip instead.
    let spacecraft_name: std::collections::HashMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    let mut spacecraft: Vec<CalcSpacecraft> = sirenix
        .spacecraft
        .iter()
        .filter(|s| s.cargo_capacity > 0.0 && s.cargo_capacity < 99_000.0)
        .filter_map(|s| {
            let name = spacecraft_name.get(s.id.as_str()).copied()?;
            if name.is_empty() { return None; }
            Some(CalcSpacecraft {
                id: s.id.clone(),
                name: name.to_string(),
                cargo_capacity: s.cargo_capacity,
            })
        })
        .collect();
    spacecraft.sort_by(|a, b| {
        a.cargo_capacity
            .partial_cmp(&b.cargo_capacity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });

    // Spacecraft modules a player ships to a colony (mining, refinery,
    // probes, habitats, power, etc.). Cargo cost is the module's own dry
    // mass, not its build_cost — they're built on Earth before launch.
    // Filter: skip legacy `id_SpaceModule_*` sentinels, contract-only items,
    // any zero-mass entry, crew transports (already captured separately),
    // and anything not loadable as cargo.
    let mut space_modules: Vec<CalcSpaceModule> = sirenix
        .space_modules
        .iter()
        .filter(|m| {
            m.id.starts_with("module_")
                && m.id != "module_contractitem"
                && m.mass > 0.0
                && m.special_ability != "CrewTransport"
                && m.can_be_load_as_cargo
        })
        .map(|m| CalcSpaceModule {
            id: m.id.clone(),
            name: humanize_module_id(&m.id),
            category: module_category(&m.special_ability),
            mass: m.mass,
            is_locked: m.is_locked,
        })
        .collect();
    space_modules.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.mass.partial_cmp(&b.mass).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.id.cmp(&b.id))
    });

    CalculatorData {
        facilities,
        resources,
        reductions,
        crew_transports,
        spacecraft,
        space_modules,
    }
}

/// Strip the `module_` prefix and title-case the rest. Specific id overrides
/// for the modules whose default humanisation reads awkwardly.
fn humanize_module_id(id: &str) -> String {
    match id {
        "module_basemining" => "Base Mining Module".into(),
        "module_icemining" => "Ice Mining Module".into(),
        "module_metalmining" => "Metal Mining Module".into(),
        "module_raremining" => "Rare Metal Mining Module".into(),
        "module_ground_probe" => "Ground Probe".into(),
        "module_space_probe" => "Space Probe".into(),
        "module_fuel" => "Fuel Refinery Module".into(),
        "module_hubel_telescope" => "Hubel Telescope".into(),
        "module_space_construction_cargo" => "Space Construction Module".into(),
        "module_construction" => "Construction Module".into(),
        "module_habitat" => "Habitat Module".into(),
        "module_power" => "Power Module".into(),
        _ => humanize_id(id.strip_prefix("module_").unwrap_or(id)),
    }
}

/// Map the raw `specialAbilityFacilityNew` enum to a player-facing category.
fn module_category(ability: &str) -> String {
    match ability {
        "Mining" => "Mining Module",
        "Probe" => "Probe",
        "Refiner" => "Refinery Module",
        "ConstructionEquipment" => "Construction Module",
        "CrewCapacity" => "Habitat Module",
        "EnergyProduction" | "EnergyProduction, EnergyStorage" => "Power Module",
        "InstallationModule" => "Installation Module",
        _ => "Module",
    }
    .to_string()
}

/// Short display names for the three crew-transport modules. Game's raw ids
/// — `module_crew_compartment` etc. — are unwieldy in tight UI spots like
/// dropdown rows; these are the names shown in the calculator.
fn crew_display_name(id: &str) -> String {
    match id {
        "module_crew_compartment" => "Crew Small".to_string(),
        "module_crew_medium" => "Crew Med".to_string(),
        "module_crew_large" => "Crew Large".to_string(),
        _ => humanize_id(id),
    }
}

/// snake_case id → Title Case display name (e.g. `module_crew_compartment`
/// → `Module Crew Compartment`).
fn humanize_id(id: &str) -> String {
    let cleaned = id.replace('_', " ");
    let mut out = String::with_capacity(cleaned.len());
    let mut cap = true;
    for c in cleaned.chars() {
        if c.is_whitespace() {
            out.push(c);
            cap = true;
        } else if cap && c.is_alphabetic() {
            for u in c.to_uppercase() { out.push(u); }
            cap = false;
        } else {
            out.push(c);
        }
    }
    out
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
            spacecraft: Vec::new(),
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

    #[test]
    fn facility_exposes_workers_and_energy_fields() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_alloymine",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Mining",
                "workers_required": 5,
                "energy_consumption": 0.5,
                "build_cost": [{"resource_id": "metal", "amount": 125.0}]
            }],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("alloymine", "EXOTIC ALLOY EXTRACTOR")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        let f = &data.facilities[0];
        assert_eq!(f.workers_required, 5);
        assert_eq!(f.energy_consumption, 0.5);
        assert_eq!(f.power_production, 0.0);
    }

    #[test]
    fn facility_with_energy_in_produces_reports_power_production() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_power_chemical",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Power",
                "workers_required": 100,
                "energy_consumption": 0.0,
                "produces": [{"resource_id": "energy", "amount": 400.0}],
                "build_cost": [{"resource_id": "steel", "amount": 400.0}]
            }],
            "resources": [{"id": "steel"}]
        }));
        let locale = locale_with(
            vec![("power_chemical", "CHEMICAL REACTOR")],
            vec![],
            vec![("steel", "Alloy")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        let f = &data.facilities[0];
        assert_eq!(f.power_production, 400.0);
        assert_eq!(f.energy_consumption, 0.0);
        assert_eq!(f.workers_required, 100);
    }

    #[test]
    fn facility_without_energy_in_produces_has_zero_power_production() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_alloysmelting",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Other",
                "workers_required": 10,
                "energy_consumption": 2.5,
                "produces": [{"resource_id": "alloy", "amount": 0.0}],
                "build_cost": [{"resource_id": "metal", "amount": 100.0}]
            }],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("alloysmelting", "ALLOY SMELTING")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        let f = &data.facilities[0];
        assert_eq!(f.power_production, 0.0);
    }

    #[test]
    fn big_variant_gets_advanced_suffix_when_basic_shares_name() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [
                {
                    "id": "build_carbonmine",
                    "descriptor": "Ground",
                    "placement": "Surface",
                    "facility_type": "Mining",
                    "build_cost": [{"resource_id": "metal", "amount": 125.0}]
                },
                {
                    "id": "build_carbonmine_big",
                    "descriptor": "Ground",
                    "placement": "Surface",
                    "facility_type": "Mining",
                    "build_cost": [{"resource_id": "metal", "amount": 1250.0}]
                }
            ],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![
                ("carbonmine", "CARBON MINE"),
                ("carbonmine_big", "CARBON MINE"),
            ],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        let names: Vec<&str> = data.facilities.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["Carbon Mine", "Carbon Mine (Advanced)"]);
    }

    #[test]
    fn crew_transports_pass_through_with_humanized_name() {
        let sirenix: Sirenix = from_json(serde_json::json!({
            "crew_transports": [
                { "id": "module_crew_compartment", "capacity": 5, "mass": 5.0, "is_locked": false },
                { "id": "module_crew_large", "capacity": 100, "mass": 60.0, "is_locked": true }
            ]
        }));
        let locale = locale_with(vec![], vec![], vec![]);

        let data = build_calculator_data(&sirenix, &locale);

        assert_eq!(data.crew_transports.len(), 2);
        assert_eq!(data.crew_transports[0].id, "module_crew_compartment");
        assert_eq!(data.crew_transports[0].name, "Crew Small");
        assert_eq!(data.crew_transports[0].capacity, 5);
        assert_eq!(data.crew_transports[0].mass, 5.0);
        assert!(!data.crew_transports[0].is_locked);
        assert_eq!(data.crew_transports[1].name, "Crew Large");
        assert!(data.crew_transports[1].is_locked);
    }

    #[test]
    fn big_variant_keeps_plain_name_when_basic_is_filtered_out() {
        // If the small variant gets excluded (e.g. missing locale), the _big
        // variant shouldn't be renamed since there's nothing to disambiguate from.
        let sirenix: Sirenix = from_json(serde_json::json!({
            "facilities": [{
                "id": "build_carbonmine_big",
                "descriptor": "Ground",
                "placement": "Surface",
                "facility_type": "Mining",
                "build_cost": [{"resource_id": "metal", "amount": 1250.0}]
            }],
            "resources": [{"id": "metal"}]
        }));
        let locale = locale_with(
            vec![("carbonmine_big", "CARBON MINE")],
            vec![],
            vec![("metal", "Metals")],
        );

        let data = build_calculator_data(&sirenix, &locale);
        assert_eq!(data.facilities.len(), 1);
        assert_eq!(data.facilities[0].name, "Carbon Mine");
    }
}

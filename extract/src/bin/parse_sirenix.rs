use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Debug, Default, PartialEq)]
struct ResourceCost {
    resource_id: String,
    amount: f64,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Spacecraft {
    id: String,
    engine_module: Option<String>,
    engine_type: String,
    mass: f64,
    cargo_capacity: f64,
    fuel_capacity: f64,
    reusability: f64,
    needs_launch_vehicle: bool,
    built_in_orbit: bool,
    can_be_built_by_player: bool,
    build_cost: Vec<ResourceCost>,
    build_time_days: f64,
    launch_cost: f64,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct LaunchVehicle {
    id: String,
    max_payload: f64,
    max_fuel_load: f64,
    exhaust_velocity: f64,
    reusability: f64,
    can_send_human: bool,
    is_locked: bool,
    build_cost: Vec<ResourceCost>,
    build_time_days: f64,
    launch_cost: f64,
    maintenance_cost_per_day: f64,
}

#[derive(Serialize)]
struct Sirenix {
    spacecraft: Vec<Spacecraft>,
    launch_vehicles: Vec<LaunchVehicle>,
}

fn parse_spacecraft(v: &Value) -> Option<Spacecraft> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("spacecraft_") {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") || lower.contains("test") {
        return None;
    }

    let f = |path: &[&str]| -> f64 { lookup_f64(v, path).unwrap_or(0.0) };
    let b = |path: &[&str]| -> bool { lookup_bool(v, path).unwrap_or(false) };

    let engine_module = v
        .pointer("/hull/engine/spaceComponent/name")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    let engine_type = v
        .get("engineType")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    let build_cost = parse_build_cost(v.pointer("/hull/priceBase/listResources"));

    Some(Spacecraft {
        id,
        engine_module,
        engine_type,
        mass: f(&["mass"]),
        cargo_capacity: f(&["cargoCapacity"]),
        fuel_capacity: f(&["fuelCapacity"]),
        reusability: f(&["reusability"]),
        needs_launch_vehicle: b(&["needLaunchVehicleToGoToMoon"]),
        built_in_orbit: b(&["orbitSC"]),
        can_be_built_by_player: b(&["canByBuildByUser"]),
        build_cost,
        build_time_days: f(&["timeToBuildInDays"]),
        launch_cost: f(&["costLaunch"]),
    })
}

fn parse_launch_vehicle(v: &Value) -> Option<LaunchVehicle> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("lv_") {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") || lower.contains("test") || lower.contains("fake") {
        return None;
    }
    let f = |path: &[&str]| -> f64 { lookup_f64(v, path).unwrap_or(0.0) };
    let b = |path: &[&str]| -> bool { lookup_bool(v, path).unwrap_or(false) };
    Some(LaunchVehicle {
        id,
        max_payload: f(&["maxPayload"]),
        max_fuel_load: f(&["maxFuelLoad"]),
        exhaust_velocity: f(&["exhaustV"]),
        reusability: f(&["reusability"]),
        can_send_human: b(&["canSendHuman"]),
        is_locked: b(&["isLocked"]),
        build_cost: parse_build_cost(v.pointer("/priceBase/listResources")),
        build_time_days: f(&["timeToBuildInDays"]),
        launch_cost: f(&["costLaunch"]),
        maintenance_cost_per_day: f(&["maintenanceCostPerDay"]),
    })
}

fn parse_build_cost(list: Option<&Value>) -> Vec<ResourceCost> {
    let arr = match list.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };
    arr.iter()
        .filter_map(|row| {
            let id = row
                .pointer("/resourceDefinitionIDSave/id")
                .and_then(|v| v.as_str())?
                .strip_prefix("id_resource_")
                .unwrap_or_else(|| {
                    // fall back to raw id if prefix missing
                    row.pointer("/resourceDefinitionIDSave/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                })
                .to_string();
            let amount = row.get("price")?.as_f64()?;
            Some(ResourceCost {
                resource_id: id,
                amount,
            })
        })
        .collect()
}

fn lookup_f64(v: &Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for &k in path {
        cur = cur.get(k)?;
    }
    cur.as_f64()
}

fn lookup_bool(v: &Value, path: &[&str]) -> Option<bool> {
    let mut cur = v;
    for &k in path {
        cur = cur.get(k)?;
    }
    cur.as_bool()
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!("usage: parse-sirenix <sirenix-dump.json> <out.json>"));
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let raw: Value = serde_json::from_str(
        &fs::read_to_string(&input).with_context(|| format!("reading {}", input.display()))?,
    )
    .with_context(|| format!("parsing {}", input.display()))?;

    let mut spacecraft: Vec<Spacecraft> = raw
        .get("SpacecraftType")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_spacecraft).collect())
        .unwrap_or_default();
    spacecraft.sort_by(|a, b| a.id.cmp(&b.id));

    let mut launch_vehicles: Vec<LaunchVehicle> = raw
        .get("LaunchVehicleType")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_launch_vehicle).collect())
        .unwrap_or_default();
    launch_vehicles.sort_by(|a, b| a.id.cmp(&b.id));

    let out = Sirenix {
        spacecraft,
        launch_vehicles,
    };
    serde_json::to_writer_pretty(fs::File::create(&output)?, &out)?;
    eprintln!(
        "wrote {} ({} spacecraft, {} launch vehicles after filtering)",
        output.display(),
        out.spacecraft.len(),
        out.launch_vehicles.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iris_fixture() -> Value {
        serde_json::json!({
            "$name": "Spacecraft1Iris",
            "$type": "SpacecraftType",
            "engineType": "chemical",
            "mass": 1,
            "cargoCapacity": 2,
            "fuelCapacity": 20,
            "reusability": 0,
            "needLaunchVehicleToGoToMoon": true,
            "orbitSC": false,
            "canByBuildByUser": true,
            "costLaunch": 1000,
            "timeToBuildInDays": 30,
            "id": "spacecraft_chem_small",
            "hull": {
                "engine": {
                    "category": "Engine",
                    "spaceComponent": { "$ref": true, "type": "SpaceComponent", "name": "eng_chemsmall" },
                    "count": 1
                },
                "priceBase": {
                    "listResources": [
                        {
                            "resourceDefinitionIDSave": { "id": "id_resource_steel" },
                            "price": 20
                        }
                    ],
                    "buildCost": 0
                }
            }
        })
    }

    fn cheat_fixture() -> Value {
        serde_json::json!({
            "$name": "id_Spacecraft_Cheat",
            "engineType": "none",
            "id": "id_Spacecraft_Cheat",
        })
    }

    #[test]
    fn parses_iris_into_spacecraft() {
        let sc = parse_spacecraft(&iris_fixture()).expect("Iris should parse");
        assert_eq!(sc.id, "spacecraft_chem_small");
        assert_eq!(sc.engine_module.as_deref(), Some("eng_chemsmall"));
        assert_eq!(sc.engine_type, "chemical");
        assert_eq!(sc.mass, 1.0);
        assert_eq!(sc.cargo_capacity, 2.0);
        assert_eq!(sc.fuel_capacity, 20.0);
        assert_eq!(sc.reusability, 0.0);
        assert!(sc.needs_launch_vehicle);
        assert!(!sc.built_in_orbit);
        assert!(sc.can_be_built_by_player);
        assert_eq!(sc.build_time_days, 30.0);
        assert_eq!(sc.launch_cost, 1000.0);
        assert_eq!(sc.build_cost.len(), 1);
        assert_eq!(sc.build_cost[0].resource_id, "steel");
        assert_eq!(sc.build_cost[0].amount, 20.0);
    }

    #[test]
    fn skips_cheat_and_test_entries() {
        assert!(parse_spacecraft(&cheat_fixture()).is_none());

        let test_v = serde_json::json!({
            "id": "spacecraft_testship",
            "engineType": "chemical",
        });
        assert!(parse_spacecraft(&test_v).is_none(), "test should be skipped");
    }

    #[test]
    fn skips_entries_without_spacecraft_prefix() {
        let bogus = serde_json::json!({ "id": "foo_bar", "engineType": "chemical" });
        assert!(parse_spacecraft(&bogus).is_none());
    }

    fn albatross_fixture() -> Value {
        serde_json::json!({
            "$name": "lv_chem_seadragon",
            "$type": "LaunchVehicleType",
            "canSendHuman": true,
            "maxPayload": 1800,
            "maxFuelLoad": 10000,
            "reusability": 0,
            "costLaunch": 4600,
            "exhaustV": 4.4,
            "isLocked": true,
            "maintenanceCostPerDay": 20,
            "timeToBuildInDays": 180,
            "priceBase": {
                "listResources": [
                    { "resourceDefinitionIDSave": { "id": "id_resource_metal" }, "price": 800 }
                ],
                "buildCost": 0
            },
            "id": "lv_chem_seadragon"
        })
    }

    #[test]
    fn parses_launch_vehicle_into_albatross() {
        let lv = parse_launch_vehicle(&albatross_fixture()).expect("Albatross should parse");
        assert_eq!(lv.id, "lv_chem_seadragon");
        assert_eq!(lv.max_payload, 1800.0);
        assert_eq!(lv.max_fuel_load, 10000.0);
        assert_eq!(lv.exhaust_velocity, 4.4);
        assert_eq!(lv.reusability, 0.0);
        assert!(lv.can_send_human);
        assert!(lv.is_locked);
        assert_eq!(lv.build_time_days, 180.0);
        assert_eq!(lv.launch_cost, 4600.0);
        assert_eq!(lv.maintenance_cost_per_day, 20.0);
        assert_eq!(lv.build_cost.len(), 1);
        assert_eq!(lv.build_cost[0].resource_id, "metal");
        assert_eq!(lv.build_cost[0].amount, 800.0);
    }

    #[test]
    fn skips_non_lv_and_test_launch_vehicles() {
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_Rocket_Cheat"})).is_none());
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_LV_launch_spin_Fake"})).is_none());
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_Rocket_RocketType5"})).is_none());
    }

    #[test]
    fn build_cost_strips_id_resource_prefix() {
        let v = serde_json::json!([
            { "resourceDefinitionIDSave": { "id": "id_resource_steel" }, "price": 12.5 },
            { "resourceDefinitionIDSave": { "id": "id_resource_water" }, "price": 5.0 },
        ]);
        let cost = parse_build_cost(Some(&v));
        assert_eq!(cost.len(), 2);
        assert_eq!(cost[0].resource_id, "steel");
        assert_eq!(cost[1].resource_id, "water");
        assert_eq!(cost[0].amount, 12.5);
    }
}

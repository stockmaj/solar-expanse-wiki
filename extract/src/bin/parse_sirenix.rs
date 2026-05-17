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
    // The fuel resource the rocket burns: `id_resource_fuel` for chemical,
    // `id_resource_hydrogen` for nuclear-thermal (LH2 reaction mass), or
    // `None` for entries where the field is unset in the dump.  Used to
    // categorize the launch-vehicles page into chemical vs. nuclear tables.
    fuel_type_on_start: Option<String>,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Research {
    id: String,
    work_hours: f64,
    branch: String,           // researchType.name (Engineering / Physics / Biotech)
    subbranch: String,        // researchSubType.name with SubBranch_ stripped
    prereqs: Vec<String>,     // ids of required research
    action: String,           // UnlockFacility / UnlockSpacecraftType / UnlockVehicleType / UnlockBonus / UnlockContract / None
    unlock_target: Option<String>,    // for non-bonus actions: the build_xxx / spacecraft_xxx / lv_xxx / contract_xxx id
    bonus_kind: Option<String>,       // for UnlockBonus: e.g. "ComponentExhaustV"
    bonus_amount: f64,
    bonus_components: Vec<String>,    // e.g. ["eng_chem"]
    show_in_tree: bool,
    contract_unlocks: Vec<String>,    // every contract id this research unlocks (from unlockData + unlockDataList)
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Facility {
    id: String,
    descriptor: String,        // "Ground" or "Orbital"
    placement: String,         // possiblePlacement enum
    facility_type: String,     // "Production", "Mining", etc.
    build_cost: Vec<ResourceCost>,
    maintenance_per_day: f64,
    workers_required: i64,
    energy_consumption: f64,
    research_prereq: Option<String>,  // research id that locks this facility
    is_obsolete: bool,
    can_be_scrapped: bool,
    can_be_turned_off: bool,
    /// Resources this facility outputs per day. Pulled from real structured
    /// data, not description text. Sources, in priority order:
    ///   * `facilityType == "Power"` with `energyProductionData.energyProduction > 0`
    ///     → produces synthetic `energy` resource. (Power facilities have a
    ///     bogus placeholder `refinerData` of metal→alloy that must be ignored.)
    ///   * `resourcesToMine[]` → mining facilities; rate is unknown statically
    ///     (depends on deposit), recorded as 0.
    ///   * `refinerData.output[]` → refiners / production / fuel plants.
    ///   * `byproducts[]` → side outputs (e.g. CO2 from carbon power plant).
    produces: Vec<ResourceCost>,
    /// Resources this facility consumes per day. Sources:
    ///   * `facilityType == "Power"` → `energyProductionData.input[]` (e.g.
    ///     uranium for nuclear, HEL3 for fusion). refinerData IS IGNORED for
    ///     Power facilities (it carries a placeholder metal→alloy default).
    ///   * Non-power → `refinerData.input[]`.
    consumes: Vec<ResourceCost>,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct SpaceComponent {
    id: String,
    category: String,       // Engine / Tank / Cargo / Crew / PowerSupply
    thrust: f64,
    exhaust_v: f64,
    mass: f64,
    power: f64,
    fuel_capacity: f64,
    cargo_capacity: f64,
    life_support_max: f64,
    fuel_type: Option<String>,
    is_locked: bool,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Resource {
    id: String,
    resource_type: String,    // Normal / Energy / Human
    market_price_base: f64,
    show_on_ui: bool,
    can_be_left_on_object: bool,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct ContractObjective {
    kind: String,                 // Possession / BuildFacility / MarketsOffers / etc.
    quantity: f64,
    target: Option<String>,       // facility id, resource id, etc.
}

/// Per-corporation starting state for a single pre-built save scenario
/// (StartGameColonization / StartGameExpansion / StartGameRaceBeyond).
/// The values are read out of `StartGameData[].companyDataSave[]` — these
/// are the canonical pre-rolled saves shipped with the game.
#[derive(Serialize, Debug, Default, PartialEq)]
struct CorpStart {
    /// e.g. "NASA", "Solex", "Roscosmos". Matches CompanyDefinition.id.
    company_id: String,
    /// Pioneer (normal) starting money. Apply DifficultyConfig multipliers
    /// (1.25x Explorer, 1.0x Pioneer, 0.75x Veteran) for the other tiers.
    starting_money: f64,
    /// Ids of research already completed when the scenario starts.
    /// References `ResearchDefinition.id` (e.g. "research_chem_main1").
    completed_research: Vec<String>,
    /// Count of launch vehicles in the company's starting fleet.
    starting_launch_vehicles: i64,
    /// Count of spacecraft in the company's starting fleet.
    starting_spacecraft: i64,
}

/// One pre-built save scenario, listing every playable corp's starting state.
/// `scenario_id` matches the `$name` field in StartGameData entries.
#[derive(Serialize, Debug, Default, PartialEq)]
struct ScenarioStart {
    scenario_id: String, // e.g. "StartGameColonization", "StartGameExpansion", "StartGameRaceBeyond"
    corp_starts: Vec<CorpStart>,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Contract {
    id: String,
    is_locked: bool,
    is_final: bool,
    objectives: Vec<ContractObjective>,
    money_reward: f64,                    // sum of Money-type rewards
    unlock_rewards: Vec<String>,          // ids of contracts / research / SC / LV unlocked on completion
    facility_grants: Vec<String>,         // facility ids granted on completion
    spacecraft_grants: Vec<String>,
    launch_vehicle_grants: Vec<String>,
    resource_grants: Vec<ResourceCost>,
    /// In-game date string when this contract first becomes offerable.
    /// Format in the dump is `MM/DD/YYYY` (e.g. `01/01/2050`). Empty in the
    /// dump means "always available from game start"; we normalize that to
    /// `None`.
    date_start_active: Option<String>,
    /// Duration (in years) the contract remains offerable before it
    /// disappears. `0` means "never expires".
    years_to_expire: f64,
}

/// One row from the dump's `StartGameEpoch` table. Each entry pins down the
/// starting date and roster of playable companies for one of the game's five
/// pre-set timelines (Prelude / Early Exploration / Colonization / The
/// Expansion / Race Beyond). Companion data — per-corp starting research
/// and fleet — lives in `StartGameData` (see `ScenarioStart`).
#[derive(Serialize, Debug, Default, PartialEq)]
struct Epoch {
    /// Raw scene id, e.g. `StartGameEpoch_Colonization`. Display names are
    /// hand-mapped in `gen_pages.rs` because no locale strings exist.
    id: String,
    /// Raw date string, formatted `DD.MM.YYYY HH:MM:SS`. Year extraction is
    /// done at render time.
    start_date_string: String,
    /// `true` for the three "future" epochs that the player has to unlock
    /// via story progression in a Prelude/Early-Exploration save.
    is_locked: bool,
    /// Corp ids (e.g. `NASA`, `Solex`) the player may choose at this epoch.
    /// Matches `CompanyDefinition.id`.
    possible_player_companies: Vec<String>,
}

#[derive(Serialize)]
struct Sirenix {
    spacecraft: Vec<Spacecraft>,
    launch_vehicles: Vec<LaunchVehicle>,
    research: Vec<Research>,
    facilities: Vec<Facility>,
    space_components: Vec<SpaceComponent>,
    resources: Vec<Resource>,
    contracts: Vec<Contract>,
    scenario_starts: Vec<ScenarioStart>,
    epochs: Vec<Epoch>,
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
    // The game has two LV id namespaces: `lv_*` (specialty rockets) and
    // `id_Rocket_*` (the core campaign rockets — Sparrow, Falcon, Eagle, etc.).
    let is_lv = id.starts_with("lv_") || id.starts_with("id_Rocket_");
    if !is_lv {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") || lower.contains("test") || lower.contains("fake") || lower.contains("forcyclemision") {
        return None;
    }
    let f = |path: &[&str]| -> f64 { lookup_f64(v, path).unwrap_or(0.0) };
    let b = |path: &[&str]| -> bool { lookup_bool(v, path).unwrap_or(false) };
    let fuel_type_on_start = v
        .pointer("/fuelTypeOnStart/name")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
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
        fuel_type_on_start,
    })
}

fn parse_research(v: &Value) -> Option<Research> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("research_") {
        return None;
    }

    let branch = v
        .pointer("/researchType/name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let subbranch = v
        .pointer("/researchSubType/name")
        .and_then(|x| x.as_str())
        .map(|s| s.strip_prefix("SubBranch_").unwrap_or(s).to_string())
        .unwrap_or_default();

    let prereqs: Vec<String> = v
        .get("requirementsResearch")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let action = v
        .pointer("/unlockData/actionUnlock")
        .and_then(|x| x.as_str())
        .unwrap_or("None")
        .to_string();
    let parameter1 = v
        .pointer("/unlockData/parameter1")
        .and_then(|x| x.as_str())
        .unwrap_or("");

    let unlock_target = match action.as_str() {
        "UnlockFacility" | "UnlockSpacecraftType" | "UnlockVehicleType" | "UnlockContract" => {
            if parameter1.is_empty() { None } else { Some(parameter1.to_string()) }
        }
        _ => None,
    };

    let bonus_kind = v
        .pointer("/unlockData/bonus")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty() && *s != "None")
        .map(|s| s.to_string());
    let bonus_amount = v
        .pointer("/unlockData/bonusParameter")
        .and_then(|x| x.as_f64())
        .unwrap_or(0.0);
    let bonus_components: Vec<String> = v
        .pointer("/unlockData/id_ComponentOrOther")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let show_in_tree = v
        .get("showInTree")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);

    // Collect every contract this research unlocks — from the primary `unlockData`
    // entry and from any `unlockDataList[]` entries.  A research item can carry
    // multiple unlock actions (e.g. unlock a facility AND unlock a contract that
    // depends on that facility); we want all of them so contract depth can chain
    // through research.
    let mut contract_unlocks: Vec<String> = Vec::new();
    if action == "UnlockContract" && !parameter1.is_empty() {
        contract_unlocks.push(parameter1.to_string());
    }
    if let Some(list) = v.get("unlockDataList").and_then(|x| x.as_array()) {
        for entry in list {
            let act = entry
                .get("actionUnlock")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            if act != "UnlockContract" {
                continue;
            }
            if let Some(p1) = entry
                .get("parameter1")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
            {
                contract_unlocks.push(p1.to_string());
            }
        }
    }

    Some(Research {
        id,
        work_hours: lookup_f64(v, &["workHourToComplete"]).unwrap_or(0.0),
        branch,
        subbranch,
        prereqs,
        action,
        unlock_target,
        bonus_kind,
        bonus_amount,
        bonus_components,
        show_in_tree,
        contract_unlocks,
    })
}

fn parse_facility(v: &Value, descriptor: &str) -> Option<Facility> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("build_") {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") || lower.contains("test") {
        return None;
    }

    let f = |path: &[&str]| -> f64 { lookup_f64(v, path).unwrap_or(0.0) };
    let i = |path: &[&str]| -> i64 {
        let mut cur = v;
        for &k in path {
            match cur.get(k) {
                Some(x) => cur = x,
                None => return 0,
            }
        }
        cur.as_i64().unwrap_or(0)
    };
    let b = |path: &[&str]| -> bool { lookup_bool(v, path).unwrap_or(false) };

    let placement = v
        .get("possiblePlacement")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let facility_type = v
        .get("facilityType")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    // lockByHelpNotUse → research_xxx that unlocks this facility
    let research_prereq = v
        .pointer("/lockByHelpNotUse/name")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    // ── produces / consumes ────────────────────────────────────────────────
    // The raw dump exposes resource I/O on several disjoint fields and the
    // shapes are not interchangeable. In particular every power facility
    // ships with a placeholder `refinerData` of `metal+volatile → alloy` —
    // game runtime ignores it for Power-type facilities, and so must we, or
    // every reactor would falsely advertise as an alloy refinery.
    //
    // Sources, in order of trust:
    //   • Power → energyProductionData (produce synthetic `energy`, consume `input[]`)
    //   • Mining → resourcesToMine[]   (rate unknown statically; recorded as 0)
    //   • Other  → refinerData.input[] / refinerData.output[]
    //   • Any     → byproducts[]       (side outputs like CO2 from carbon plant)
    let mut produces: Vec<ResourceCost> = Vec::new();
    let mut consumes: Vec<ResourceCost> = Vec::new();
    let is_power = facility_type == "Power";

    if is_power {
        let prod = lookup_f64(v, &["energyProductionData", "energyProduction"]).unwrap_or(0.0);
        if prod > 0.0 {
            produces.push(ResourceCost {
                resource_id: "energy".to_string(),
                amount: prod,
            });
        }
        for entry in v
            .pointer("/energyProductionData/input")
            .and_then(|x| x.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(rc) = parse_resource_cost_io(entry) {
                consumes.push(rc);
            }
        }
    } else {
        // resourcesToMine[] → simple list of `{name: "id_resource_xxx"}`.
        for entry in v
            .get("resourcesToMine")
            .and_then(|x| x.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(rid) = entry.get("name").and_then(|x| x.as_str()) {
                produces.push(ResourceCost {
                    resource_id: normalize_resource_id(rid),
                    amount: 0.0, // mining rate isn't a static constant
                });
            }
        }
        // refinerData.{input,output} → standard `{resource:{name}, ratePerDay}`.
        for entry in v
            .pointer("/refinerData/output")
            .and_then(|x| x.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(rc) = parse_resource_cost_io(entry) {
                produces.push(rc);
            }
        }
        for entry in v
            .pointer("/refinerData/input")
            .and_then(|x| x.as_array())
            .into_iter()
            .flatten()
        {
            if let Some(rc) = parse_resource_cost_io(entry) {
                consumes.push(rc);
            }
        }
    }
    // byproducts[] applies regardless of facility type — `{resource:{name}, rate}`.
    for entry in v
        .get("byproducts")
        .and_then(|x| x.as_array())
        .into_iter()
        .flatten()
    {
        let rid = entry
            .pointer("/resource/name")
            .and_then(|x| x.as_str());
        let amount = entry.get("rate").and_then(|x| x.as_f64()).unwrap_or(0.0);
        if let Some(rid) = rid {
            produces.push(ResourceCost {
                resource_id: normalize_resource_id(rid),
                amount,
            });
        }
    }

    // Stable sort & dedup so output is deterministic regardless of dump order.
    produces.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));
    produces.dedup_by(|a, b| a.resource_id == b.resource_id);
    consumes.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));
    consumes.dedup_by(|a, b| a.resource_id == b.resource_id);

    Some(Facility {
        id,
        descriptor: descriptor.to_string(),
        placement,
        facility_type,
        build_cost: parse_build_cost(v.pointer("/price/listResources")),
        maintenance_per_day: f(&["maintenanceCostPerDay"]),
        workers_required: i(&["needWorkersToWork"]),
        energy_consumption: f(&["energyConsumption"]),
        research_prereq,
        is_obsolete: b(&["isObsolete"]),
        can_be_scrapped: b(&["canBeScrapped"]),
        can_be_turned_off: b(&["canBeTurnedOff"]),
        produces,
        consumes,
    })
}

/// Parse `{resource: {name: "id_resource_xxx"}, ratePerDay: 1.5}` → ResourceCost.
fn parse_resource_cost_io(entry: &Value) -> Option<ResourceCost> {
    let rid = entry.pointer("/resource/name").and_then(|x| x.as_str())?;
    let amount = entry.get("ratePerDay").and_then(|x| x.as_f64()).unwrap_or(0.0);
    Some(ResourceCost {
        resource_id: normalize_resource_id(rid),
        amount,
    })
}

/// Strip `id_resource_` prefix and lowercase. The raw dump mixes case for
/// hel3 (`id_resource_HEL3` in facility data, `id_resource_hel3` in
/// ResourceDefinition); we normalize both to `hel3`.
fn normalize_resource_id(raw: &str) -> String {
    raw.strip_prefix("id_resource_").unwrap_or(raw).to_ascii_lowercase()
}

fn parse_space_component(v: &Value) -> Option<SpaceComponent> {
    let id = v.get("id")?.as_str()?.to_string();
    if id.is_empty() {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") || lower.contains("placeholder") {
        return None;
    }

    let f = |path: &[&str]| -> f64 { lookup_f64(v, path).unwrap_or(0.0) };

    let category = v
        .get("category")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let fuel_type = v
        .pointer("/fuelType/name")
        .and_then(|x| x.as_str())
        .map(|s| s.strip_prefix("id_resource_").unwrap_or(s).to_string());

    Some(SpaceComponent {
        id,
        category,
        thrust: f(&["thrust"]),
        exhaust_v: f(&["exhaustV"]),
        mass: f(&["mass"]),
        power: f(&["power"]),
        fuel_capacity: f(&["fuelCapacity"]),
        cargo_capacity: f(&["cargoCapacity"]),
        life_support_max: f(&["lifeSupportMax"]),
        fuel_type,
        is_locked: lookup_bool(v, &["isLocked"]).unwrap_or(false),
    })
}

fn parse_resource(v: &Value) -> Option<Resource> {
    let raw_id = v.get("id")?.as_str()?;
    if raw_id.is_empty() {
        return None;
    }
    let id = raw_id.strip_prefix("id_resource_").unwrap_or(raw_id).to_string();
    if id.contains("empty") || id.contains("cheat") {
        return None;
    }
    Some(Resource {
        id,
        resource_type: v.get("resourceType").and_then(|x| x.as_str()).unwrap_or("Normal").to_string(),
        market_price_base: lookup_f64(v, &["marketClearingPriceBase"]).unwrap_or(0.0),
        show_on_ui: lookup_bool(v, &["showOnUI"]).unwrap_or(true),
        can_be_left_on_object: lookup_bool(v, &["canBeLeftOnObject"]).unwrap_or(false),
    })
}

fn parse_contract(v: &Value) -> Option<Contract> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("contract_") {
        return None;
    }
    let lower = id.to_ascii_lowercase();
    if lower.contains("cheat") {
        return None;
    }

    let mut money_reward = 0.0;
    let mut unlock_rewards: Vec<String> = Vec::new();
    let mut facility_grants: Vec<String> = Vec::new();
    let mut spacecraft_grants: Vec<String> = Vec::new();
    let mut launch_vehicle_grants: Vec<String> = Vec::new();
    let mut resource_grants: Vec<ResourceCost> = Vec::new();

    if let Some(rewards) = v.get("rewards").and_then(|x| x.as_array()) {
        for r in rewards {
            let kind = r.get("rewardType").and_then(|x| x.as_str()).unwrap_or("");
            let amount = r.get("amount").and_then(|x| x.as_f64()).unwrap_or(0.0);
            match kind {
                "Money" => money_reward += amount,
                "Resource" => {
                    if let Some(rid) = r
                        .pointer("/resourceDefinition/name")
                        .and_then(|x| x.as_str())
                    {
                        resource_grants.push(ResourceCost {
                            resource_id: rid.strip_prefix("id_resource_").unwrap_or(rid).to_string(),
                            amount,
                        });
                    }
                }
                _ => {}
            }
            // Facility / spacecraft / LV grants come in separate fields on the reward struct
            if let Some(f) = r.pointer("/facilityBaseDescriptor/name").and_then(|x| x.as_str()) {
                if !f.is_empty() {
                    facility_grants.push(f.to_string());
                }
            }
            if let Some(sc) = r.pointer("/spaceCraftType/name").and_then(|x| x.as_str()) {
                if !sc.is_empty() {
                    spacecraft_grants.push(sc.to_string());
                }
            }
            if let Some(lv) = r.pointer("/launchVehicleType/name").and_then(|x| x.as_str()) {
                if !lv.is_empty() {
                    launch_vehicle_grants.push(lv.to_string());
                }
            }
            if let Some(target) = r
                .pointer("/unlockData/parameter1")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
            {
                unlock_rewards.push(target.to_string());
            }
        }
    }

    let mut objectives: Vec<ContractObjective> = Vec::new();
    if let Some(obj_arr) = v.get("objectives").and_then(|x| x.as_array()) {
        for o in obj_arr {
            let kind = o
                .get("objectiveType")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if kind.is_empty() {
                continue;
            }
            let quantity = o.get("howMuch").and_then(|x| x.as_f64()).unwrap_or(0.0);
            // Target is whichever of productItem / marketsOffersObjectiveData.rd is populated.
            let target = o
                .pointer("/productItem/name")
                .and_then(|x| x.as_str())
                .or_else(|| o.pointer("/marketsOffersObjectiveData/rd/name").and_then(|x| x.as_str()))
                .or_else(|| o.pointer("/changeDepositParametersObjectiveData/rd/name").and_then(|x| x.as_str()))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            objectives.push(ContractObjective {
                kind,
                quantity,
                target,
            });
        }
    }

    // dateStartActive: "MM/DD/YYYY" — empty string means "always available",
    // which we normalize to None so the renderer can show a "—" cell.
    let date_start_active = v
        .get("dateStartActive")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let years_to_expire = v
        .get("yearsToExpire")
        .and_then(|x| x.as_f64())
        .unwrap_or(0.0);

    Some(Contract {
        id,
        is_locked: lookup_bool(v, &["isLocked"]).unwrap_or(false),
        is_final: lookup_bool(v, &["isFinalContract"]).unwrap_or(false),
        objectives,
        money_reward,
        unlock_rewards,
        facility_grants,
        spacecraft_grants,
        launch_vehicle_grants,
        resource_grants,
        date_start_active,
        years_to_expire,
    })
}

/// Parse a single `StartGameEpoch` entry. The dump has five entries — one
/// per epoch — and each is keyed by `id` (e.g. `StartGameEpoch_Colonization`).
/// We extract just the player-relevant fields; the `startDate` ticks integer
/// and serializationData blob aren't needed because `startDateString` already
/// carries the human-readable date.
fn parse_epoch(v: &Value) -> Option<Epoch> {
    let id = v.get("id")?.as_str()?.to_string();
    if !id.starts_with("StartGameEpoch_") {
        return None;
    }
    let start_date_string = v
        .get("startDateString")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let is_locked = lookup_bool(v, &["isLocked"]).unwrap_or(false);
    let possible_player_companies: Vec<String> = v
        .get("possiblePlayerCompanies")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Some(Epoch {
        id,
        start_date_string,
        is_locked,
        possible_player_companies,
    })
}

/// Parse a single `StartGameData` entry from the Sirenix dump into a
/// `ScenarioStart` (per-corp starting research / funding / fleet counts).
///
/// The data lives inside `serializationData.SerializationNodes`, which is
/// a flat node stream emitted by Sirenix's serializer. We walk the stream
/// to find the `companyDataSave` array and, for each company, extract:
///   * `companyID.id`               → corp name
///   * `researchDataToSave.completeResearch[].id` → completed research ids
///   * `launchVehicles[]` / `spacecrafts[]` array counts → starting fleet
///   * `money`                      → starting funding (Pioneer/Normal base)
///
/// Returns `None` if the entry has no populated `companyDataSave` (the
/// Kepler/Trapist/PerfectSystem placeholders, and the empty SceneJSON).
fn parse_scenario_start(entry: &Value) -> Option<ScenarioStart> {
    let scenario_id = entry.get("$name").and_then(|v| v.as_str())?.to_string();
    // Skip non-scenario / placeholder entries. Pre-built saves are
    // exactly the three real epoch scenarios; everything else is a
    // test/dev artifact or an empty placeholder.
    let scenario_kind = match scenario_id.as_str() {
        "StartGameColonization" | "StartGameExpansion" | "StartGameRaceBeyond" => scenario_id.clone(),
        _ => return None,
    };

    let nodes = entry.pointer("/serializationData/SerializationNodes")?.as_array()?;

    // Find the `companyDataSave` field, then the inner StartOfArray
    // whose `Data` carries the company count.
    let mut i = 0usize;
    while i < nodes.len() {
        if name_of(&nodes[i]) == "companyDataSave" && entry_of(&nodes[i]) == "StartOfNode" {
            break;
        }
        i += 1;
    }
    if i >= nodes.len() {
        return None;
    }
    // Next non-empty node should be StartOfArray with count.
    let mut j = i + 1;
    while j < nodes.len() && entry_of(&nodes[j]) != "StartOfArray" {
        j += 1;
    }
    if j >= nodes.len() {
        return None;
    }
    let n_companies: usize = data_of(&nodes[j]).parse().unwrap_or(0);
    let mut idx = j + 1;
    let mut corp_starts: Vec<CorpStart> = Vec::new();
    for _ in 0..n_companies {
        // Skip ahead to next StartOfNode at depth 0 inside the array.
        while idx < nodes.len() && entry_of(&nodes[idx]) != "StartOfNode" {
            idx += 1;
        }
        if idx >= nodes.len() {
            break;
        }
        let (corp, next) = walk_company(nodes, idx);
        if let Some(c) = corp {
            corp_starts.push(c);
        }
        idx = next;
    }

    // Filter out WorldGovernment (the AI faction, not a playable corp).
    corp_starts.retain(|c| c.company_id != "WorldGovernment");
    corp_starts.sort_by(|a, b| a.company_id.cmp(&b.company_id));

    Some(ScenarioStart {
        scenario_id: scenario_kind,
        corp_starts,
    })
}

fn name_of(n: &Value) -> &str {
    n.get("Name").and_then(|v| v.as_str()).unwrap_or("")
}
fn entry_of(n: &Value) -> &str {
    n.get("Entry").and_then(|v| v.as_str()).unwrap_or("")
}
fn data_of(n: &Value) -> &str {
    n.get("Data").and_then(|v| v.as_str()).unwrap_or("")
}

/// Walk one `Game.CompanyDataSave` node (starting at `nodes[start]` which
/// must be a `StartOfNode`). Returns the parsed `CorpStart` and the index
/// just past the matching `EndOfNode`.
fn walk_company(nodes: &[Value], start: usize) -> (Option<CorpStart>, usize) {
    let mut depth: i32 = 0;
    let mut j = start;
    let mut company = CorpStart::default();
    let mut got_company_id = false;
    let mut in_complete_research = false;
    let mut cr_depth: i32 = -1;
    // Track depth at which each named sub-array opened, so we can grab its
    // StartOfArray count without confusing it with nested arrays.
    while j < nodes.len() {
        let n = &nodes[j];
        let e = entry_of(n);
        let nm = name_of(n);

        // The very first 'id' string (companyID.id) — captured before any other.
        if e == "String" && nm == "id" && !got_company_id {
            company.company_id = data_of(n).to_string();
            got_company_id = true;
        }
        // Each named list opens with StartOfNode whose next StartOfArray
        // carries the count. We just read it directly.
        if e == "StartOfNode" {
            if nm == "launchVehicles" {
                if let Some(next) = nodes.get(j + 1) {
                    if entry_of(next) == "StartOfArray" {
                        company.starting_launch_vehicles =
                            data_of(next).parse().unwrap_or(0);
                    }
                }
            } else if nm == "spacecrafts" {
                if let Some(next) = nodes.get(j + 1) {
                    if entry_of(next) == "StartOfArray" {
                        company.starting_spacecraft = data_of(next).parse().unwrap_or(0);
                    }
                }
            } else if nm == "completeResearch" {
                in_complete_research = true;
                cr_depth = depth;
            }
        }
        if in_complete_research && e == "String" && nm == "id" {
            // Each completed-research entry is a tiny node with just `id`.
            // We already captured the company id earlier, so any subsequent
            // 'id' string inside the completeResearch region is research.
            company.completed_research.push(data_of(n).to_string());
        }
        if e == "FloatingPoint" && nm == "money" {
            if let Ok(v) = data_of(n).parse::<f64>() {
                company.starting_money = v;
            }
        }

        // Update depth and detect closing of completeResearch and the
        // overall company node.
        if e == "StartOfNode" || e == "StartOfArray" {
            depth += 1;
        } else if e == "EndOfNode" || e == "EndOfArray" {
            depth -= 1;
            if in_complete_research && depth == cr_depth {
                in_complete_research = false;
            }
            if depth == 0 {
                j += 1;
                return (
                    if got_company_id { Some(company) } else { None },
                    j,
                );
            }
        }
        j += 1;
    }
    (
        if got_company_id { Some(company) } else { None },
        j,
    )
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

    let mut research: Vec<Research> = raw
        .get("ResearchDefinition")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_research).collect())
        .unwrap_or_default();
    research.sort_by(|a, b| {
        a.branch
            .cmp(&b.branch)
            .then(a.subbranch.cmp(&b.subbranch))
            .then(a.id.cmp(&b.id))
    });

    let mut facilities: Vec<Facility> = Vec::new();
    if let Some(arr) = raw.get("GroundFacilityDescriptor").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(f) = parse_facility(v, "Ground") {
                facilities.push(f);
            }
        }
    }
    if let Some(arr) = raw.get("SpaceModuleDescriptor").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(f) = parse_facility(v, "Orbital") {
                facilities.push(f);
            }
        }
    }
    facilities.sort_by(|a, b| a.id.cmp(&b.id));

    let mut space_components: Vec<SpaceComponent> = raw
        .get("SpaceComponent")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_space_component).collect())
        .unwrap_or_default();
    space_components.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.mass.partial_cmp(&b.mass).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.id.cmp(&b.id))
    });

    let mut resources: Vec<Resource> = raw
        .get("ResourceDefinition")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_resource).collect())
        .unwrap_or_default();
    resources.sort_by(|a, b| a.id.cmp(&b.id));

    let mut contracts: Vec<Contract> = raw
        .get("ContractDefinition")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_contract).collect())
        .unwrap_or_default();
    contracts.sort_by(|a, b| a.id.cmp(&b.id));

    // Pre-built starting saves (Colonization / Expansion / RaceBeyond).
    // Each StartGameData entry encodes one full pre-rolled save scene; we
    // pluck out just the per-company starting research / money / fleet.
    let mut scenario_starts: Vec<ScenarioStart> = raw
        .get("StartGameData")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_scenario_start).collect())
        .unwrap_or_default();
    // Order scenarios by in-game timeline (Colonization → Expansion → RaceBeyond).
    let timeline = |s: &ScenarioStart| -> u8 {
        match s.scenario_id.as_str() {
            "StartGameColonization" => 0,
            "StartGameExpansion" => 1,
            "StartGameRaceBeyond" => 2,
            _ => 99,
        }
    };
    scenario_starts.sort_by_key(timeline);

    // StartGameEpoch entries pin start dates and playable corps per epoch
    // (Prelude / Early Exploration / Colonization / The Expansion / Race
    // Beyond). Five entries total.
    let mut epochs: Vec<Epoch> = raw
        .get("StartGameEpoch")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_epoch).collect())
        .unwrap_or_default();
    let epoch_order = |e: &Epoch| -> u8 {
        match e.id.as_str() {
            "StartGameEpoch_Prelude" => 0,
            "StartGameEpoch_EarlyExploration" => 1,
            "StartGameEpoch_Colonization" => 2,
            "StartGameEpoch_TheExpansion" => 3,
            "StartGameEpoch_RaceBeyond" => 4,
            _ => 99,
        }
    };
    epochs.sort_by_key(epoch_order);

    let out = Sirenix {
        spacecraft,
        launch_vehicles,
        research,
        facilities,
        space_components,
        resources,
        contracts,
        scenario_starts,
        epochs,
    };
    serde_json::to_writer_pretty(fs::File::create(&output)?, &out)?;
    eprintln!(
        "wrote {} ({} spacecraft, {} LVs, {} research, {} facilities, {} components, {} resources, {} contracts, {} scenarios, {} epochs)",
        output.display(),
        out.spacecraft.len(),
        out.launch_vehicles.len(),
        out.research.len(),
        out.facilities.len(),
        out.space_components.len(),
        out.resources.len(),
        out.contracts.len(),
        out.scenario_starts.len(),
        out.epochs.len(),
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
        // Cheat / Fake / cyclical-mission entries should be filtered out.
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_Rocket_Cheat"})).is_none());
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_LV_launch_spin_Fake"})).is_none());
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "id_Rocket_ForCycleMision"})).is_none());
        // Things that don't look like a launch vehicle at all
        assert!(parse_launch_vehicle(&serde_json::json!({"id": "spacecraft_chem_small"})).is_none());
    }

    #[test]
    fn accepts_id_Rocket_RocketType_namespace() {
        // Core campaign rockets (Sparrow, Falcon, Eagle, ...) use this id shape.
        let r = serde_json::json!({"id": "id_Rocket_RocketType1", "maxPayload": 10});
        let parsed = parse_launch_vehicle(&r).expect("RocketType1 should parse");
        assert_eq!(parsed.max_payload, 10.0);
    }

    #[test]
    fn captures_fuel_type_for_chemical_and_nuclear() {
        // Sparrow-style chemical rocket: fuelTypeOnStart points at id_resource_fuel.
        let chem = serde_json::json!({
            "id": "id_Rocket_RocketType1",
            "fuelTypeOnStart": { "$ref": true, "type": "ResourceDefinition", "name": "id_resource_fuel" }
        });
        let c = parse_launch_vehicle(&chem).unwrap();
        assert_eq!(c.fuel_type_on_start.as_deref(), Some("id_resource_fuel"));

        // Nuclear-thermal rocket: hydrogen as reaction mass.
        let nuke = serde_json::json!({
            "id": "lv_nuke_small",
            "fuelTypeOnStart": { "name": "id_resource_hydrogen" }
        });
        let n = parse_launch_vehicle(&nuke).unwrap();
        assert_eq!(n.fuel_type_on_start.as_deref(), Some("id_resource_hydrogen"));

        // Missing field stays None (some dump entries have fuelTypeOnStart=null).
        let bare = serde_json::json!({"id": "lv_chem_seadragon"});
        assert!(parse_launch_vehicle(&bare).unwrap().fuel_type_on_start.is_none());
    }

    #[test]
    fn parses_research_with_facility_unlock() {
        let v = serde_json::json!({
            "id": "research_agriculture_1",
            "workHourToComplete": 1200000,
            "researchType": {"name": "Biotech"},
            "researchSubType": {"name": "SubBranch_Agriculture"},
            "requirementsResearch": [{"name": "research_biotech_base"}],
            "unlockData": {
                "actionUnlock": "UnlockFacility",
                "parameter1": "build_farm",
                "bonus": "None",
                "bonusParameter": 0,
                "id_ComponentOrOther": []
            },
            "showInTree": false
        });
        let r = parse_research(&v).expect("should parse");
        assert_eq!(r.id, "research_agriculture_1");
        assert_eq!(r.work_hours, 1_200_000.0);
        assert_eq!(r.branch, "Biotech");
        assert_eq!(r.subbranch, "Agriculture");
        assert_eq!(r.prereqs, vec!["research_biotech_base"]);
        assert_eq!(r.action, "UnlockFacility");
        assert_eq!(r.unlock_target.as_deref(), Some("build_farm"));
        assert!(r.bonus_kind.is_none());
        assert!(!r.show_in_tree);
    }

    #[test]
    fn parses_research_with_bonus_unlock() {
        let v = serde_json::json!({
            "id": "research_chem_main2",
            "workHourToComplete": 2160000,
            "researchType": {"name": "Engineering"},
            "researchSubType": {"name": "SubBranch_Chemical"},
            "requirementsResearch": [{"name": "research_chem_main1"}],
            "unlockData": {
                "actionUnlock": "UnlockBonus",
                "parameter1": "I",
                "bonus": "ComponentExhaustV",
                "bonusParameter": 5,
                "id_ComponentOrOther": ["eng_chem"]
            }
        });
        let r = parse_research(&v).expect("should parse");
        assert_eq!(r.subbranch, "Chemical");
        assert_eq!(r.action, "UnlockBonus");
        assert_eq!(r.bonus_kind.as_deref(), Some("ComponentExhaustV"));
        assert_eq!(r.bonus_amount, 5.0);
        assert_eq!(r.bonus_components, vec!["eng_chem"]);
        assert!(r.unlock_target.is_none());
    }

    fn scenario_fixture() -> Value {
        // A trimmed StartGameData entry that mirrors the real dump's
        // companyDataSave region: two companies (NASA and WorldGovernment),
        // with NASA having two completed researches and a single starting
        // launch vehicle / two starting spacecraft.
        // The serialization node stream is a flat list of name/entry/data
        // triples — same as `serializationData.SerializationNodes` in the
        // real dump.
        serde_json::json!({
            "$name": "StartGameColonization",
            "id": "StartGameExpansion",
            "serializationData": {
                "SerializationNodes": [
                    { "Name": "companyDataSave", "Entry": "StartOfNode", "Data": "0|List`1[CompanyDataSave]" },
                    { "Name": "", "Entry": "StartOfArray", "Data": "2" },
                    { "Name": "", "Entry": "StartOfNode", "Data": "1|CompanyDataSave" },
                    { "Name": "companyID", "Entry": "StartOfNode", "Data": "2|CompanyIDSave" },
                    { "Name": "id", "Entry": "String", "Data": "NASA" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "launchVehicles", "Entry": "StartOfNode", "Data": "3|List`1" },
                    { "Name": "", "Entry": "StartOfArray", "Data": "1" },
                    { "Name": "", "Entry": "EndOfArray", "Data": "" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "spacecrafts", "Entry": "StartOfNode", "Data": "4|List`1" },
                    { "Name": "", "Entry": "StartOfArray", "Data": "2" },
                    { "Name": "", "Entry": "EndOfArray", "Data": "" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "researchDataToSave", "Entry": "StartOfNode", "Data": "5|ResearchDataToSave" },
                    { "Name": "completeResearch", "Entry": "StartOfNode", "Data": "6|List`1" },
                    { "Name": "", "Entry": "StartOfArray", "Data": "2" },
                    { "Name": "", "Entry": "StartOfNode", "Data": "7|ResearchDefinitionSave" },
                    { "Name": "id", "Entry": "String", "Data": "research_chem_main1" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "", "Entry": "StartOfNode", "Data": "8|ResearchDefinitionSave" },
                    { "Name": "id", "Entry": "String", "Data": "research_lv_main1" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "", "Entry": "EndOfArray", "Data": "" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "money", "Entry": "FloatingPoint", "Data": "35900000" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "", "Entry": "StartOfNode", "Data": "9|CompanyDataSave" },
                    { "Name": "companyID", "Entry": "StartOfNode", "Data": "10|CompanyIDSave" },
                    { "Name": "id", "Entry": "String", "Data": "WorldGovernment" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "money", "Entry": "FloatingPoint", "Data": "500000000" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" },
                    { "Name": "", "Entry": "EndOfArray", "Data": "" },
                    { "Name": "", "Entry": "EndOfNode", "Data": "" }
                ]
            }
        })
    }

    #[test]
    fn parses_scenario_start_for_colonization() {
        let s = parse_scenario_start(&scenario_fixture()).expect("colonization should parse");
        assert_eq!(s.scenario_id, "StartGameColonization");
        // WorldGovernment must be filtered out — only NASA remains.
        assert_eq!(s.corp_starts.len(), 1);
        let nasa = &s.corp_starts[0];
        assert_eq!(nasa.company_id, "NASA");
        assert_eq!(nasa.starting_money, 35_900_000.0);
        assert_eq!(nasa.starting_launch_vehicles, 1);
        assert_eq!(nasa.starting_spacecraft, 2);
        assert_eq!(
            nasa.completed_research,
            vec!["research_chem_main1".to_string(), "research_lv_main1".to_string()]
        );
    }

    #[test]
    fn skips_placeholder_and_test_scenarios() {
        // Kepler/Trapist/PerfectSystem placeholders have $name like
        // "StartGameKepler"; only the three real epoch scenarios should parse.
        let placeholder = serde_json::json!({
            "$name": "StartGameKepler",
            "serializationData": { "SerializationNodes": [] }
        });
        assert!(parse_scenario_start(&placeholder).is_none());
        let test = serde_json::json!({
            "$name": "testStartGAme",
            "serializationData": { "SerializationNodes": [] }
        });
        assert!(parse_scenario_start(&test).is_none());
    }

    #[test]
    fn facility_power_plant_produces_energy_from_energy_production_data() {
        // Carbon power plant: real game data has facilityType=Power,
        // energyProductionData.energyProduction=350, and inputs of volatile+oxygen.
        // refinerData carries a placeholder metal→alloy default that MUST be ignored.
        let v = serde_json::json!({
            "id": "build_power_carbon",
            "possiblePlacement": "Surface",
            "facilityType": "Power",
            "energyProductionData": {
                "energyProduction": 350,
                "input": [
                    {"resource": {"name": "id_resource_volatile"}, "ratePerDay": 0.12},
                    {"resource": {"name": "id_resource_oxygen"}, "ratePerDay": 0.32}
                ]
            },
            "refinerData": {
                "input": [{"resource": {"name": "id_resource_metal"}, "ratePerDay": 1.5}],
                "output": [{"resource": {"name": "id_resource_alloy"}, "ratePerDay": 1.5}]
            },
            "byproducts": [{"resource": {"name": "id_resource_co2"}, "rate": 0.44, "state": "Gas"}]
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let produced_ids: Vec<&str> = f.produces.iter().map(|c| c.resource_id.as_str()).collect();
        assert!(produced_ids.contains(&"energy"), "expected energy in produces, got {:?}", produced_ids);
        assert!(produced_ids.contains(&"co2"), "expected co2 byproduct, got {:?}", produced_ids);
        assert!(!produced_ids.contains(&"alloy"), "must NOT inherit placeholder alloy from refinerData, got {:?}", produced_ids);
        let consumed_ids: Vec<&str> = f.consumes.iter().map(|c| c.resource_id.as_str()).collect();
        assert!(consumed_ids.contains(&"volatile"));
        assert!(consumed_ids.contains(&"oxygen"));
        assert!(!consumed_ids.contains(&"metal"), "must NOT inherit placeholder metal from refinerData, got {:?}", consumed_ids);
    }

    #[test]
    fn facility_mine_produces_mined_resource() {
        // Fissiles mine: resourcesToMine carries the produced id; refinerData empty.
        let v = serde_json::json!({
            "id": "build_uranmine",
            "facilityType": "Mining",
            "resourcesToMine": [{"name": "id_resource_uran"}],
            "refinerData": {"input": [], "output": []},
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let produced_ids: Vec<&str> = f.produces.iter().map(|c| c.resource_id.as_str()).collect();
        assert_eq!(produced_ids, vec!["uran"]);
        assert!(f.consumes.is_empty());
    }

    #[test]
    fn facility_refiner_produces_output_consumes_input() {
        // Fissile Extraction Facility (build_earthnuke): refinerData.output → uran.
        let v = serde_json::json!({
            "id": "build_earthnuke",
            "facilityType": "Other",
            "refinerData": {
                "input": [],
                "output": [{"resource": {"name": "id_resource_uran"}, "ratePerDay": 0.01}]
            },
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let produced_ids: Vec<&str> = f.produces.iter().map(|c| c.resource_id.as_str()).collect();
        assert_eq!(produced_ids, vec!["uran"]);
    }

    #[test]
    fn facility_exotic_alloy_consumes_fissiles_does_not_produce_them() {
        // Exotic Alloy Production: refinerData.input has raremetal + uran,
        // output is alloy. This is the bug — the old substring heuristic
        // mistook this for a Fissiles/Rare Metals/Metals producer.
        let v = serde_json::json!({
            "id": "build_exoticalloy",
            "facilityType": "Production",
            "refinerData": {
                "input": [
                    {"resource": {"name": "id_resource_raremetal"}, "ratePerDay": 0.09},
                    {"resource": {"name": "id_resource_uran"}, "ratePerDay": 0.01}
                ],
                "output": [{"resource": {"name": "id_resource_alloy"}, "ratePerDay": 0.08}]
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let produced_ids: Vec<&str> = f.produces.iter().map(|c| c.resource_id.as_str()).collect();
        assert_eq!(produced_ids, vec!["alloy"]);
        assert!(!produced_ids.contains(&"uran"), "exotic alloy must not be a producer of fissiles");
        assert!(!produced_ids.contains(&"raremetal"));
        let consumed_ids: Vec<&str> = f.consumes.iter().map(|c| c.resource_id.as_str()).collect();
        assert!(consumed_ids.contains(&"uran"));
        assert!(consumed_ids.contains(&"raremetal"));
    }

    #[test]
    fn facility_hel3_resource_id_lowercased() {
        // Raw dump has id_resource_HEL3 (mixed case); we normalize to lowercase
        // to match the resource ids emitted by `parse_resource`.
        let v = serde_json::json!({
            "id": "build_power_fusion",
            "facilityType": "Power",
            "energyProductionData": {
                "energyProduction": 5000,
                "input": [{"resource": {"name": "id_resource_HEL3"}, "ratePerDay": 0.42}]
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let consumed_ids: Vec<&str> = f.consumes.iter().map(|c| c.resource_id.as_str()).collect();
        assert!(consumed_ids.contains(&"hel3"), "got {:?}", consumed_ids);
    }

    #[test]
    fn parses_contract_timing_fields() {
        // ContractDefinition entries carry dateStartActive ("MM/DD/YYYY") and
        // yearsToExpire (numeric, in years). Both are surfaced verbatim.
        let v = serde_json::json!({
            "id": "contract_mars_outpost",
            "isLocked": false,
            "isFinalContract": false,
            "dateStartActive": "01/01/2050",
            "yearsToExpire": 5,
            "objectives": [],
            "rewards": []
        });
        let c = parse_contract(&v).expect("contract should parse");
        assert_eq!(c.date_start_active.as_deref(), Some("01/01/2050"));
        assert_eq!(c.years_to_expire, 5.0);
    }

    #[test]
    fn parses_contract_with_empty_date_and_zero_expiry() {
        // Many contracts have dateStartActive="" and yearsToExpire=0 in the
        // dump (always-available, never expires). Both should parse cleanly.
        let v = serde_json::json!({
            "id": "contract_anytime",
            "isLocked": false,
            "isFinalContract": false,
            "dateStartActive": "",
            "yearsToExpire": 0,
            "objectives": [],
            "rewards": []
        });
        let c = parse_contract(&v).expect("contract should parse");
        assert!(c.date_start_active.is_none() || c.date_start_active.as_deref() == Some(""));
        assert_eq!(c.years_to_expire, 0.0);
    }

    fn epoch_fixture() -> Value {
        // Trimmed real StartGameEpoch entry from the Sirenix dump.
        serde_json::json!({
            "$name": "StartGameEpoch_Colonization",
            "$type": "StartGameEpoch",
            "translationKeyPrefix": "Game.UI.CustomizationScreen.StartDateSettings.Item4",
            "startDateString": "01.01.2100 00:00:00",
            "isLocked": true,
            "possiblePlayerCompanies": [
                { "$ref": true, "type": "CompanyDefinition", "name": "CNSA" },
                { "$ref": true, "type": "CompanyDefinition", "name": "ESA" },
                { "$ref": true, "type": "CompanyDefinition", "name": "NASA" },
                { "$ref": true, "type": "CompanyDefinition", "name": "Roscosmos" }
            ],
            "id": "StartGameEpoch_Colonization"
        })
    }

    #[test]
    fn parses_epoch_fields() {
        let e = parse_epoch(&epoch_fixture()).expect("colonization epoch should parse");
        assert_eq!(e.id, "StartGameEpoch_Colonization");
        assert_eq!(e.start_date_string, "01.01.2100 00:00:00");
        assert!(e.is_locked);
        assert_eq!(e.possible_player_companies.len(), 4);
        assert!(e.possible_player_companies.contains(&"NASA".to_string()));
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

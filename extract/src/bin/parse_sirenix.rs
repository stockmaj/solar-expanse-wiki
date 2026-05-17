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
    /// Days required to construct the facility. From `timeToBuildInDays`.
    build_time_days: f64,
    /// Launch-method bonus payload, if any. Source: `bonusData` (which the dump
    /// stores as a *single object*, not an array). Surfaces only when
    /// `bonus != "None"` — every non-launch facility carries a `"None"`
    /// placeholder we must skip. Shape: `(bonus_kind, bonusParameter)`.
    /// Real values seen in dump: LaunchCost / LaunchCostOptionInPlanMission /
    /// SpaceCraftInPlanMission with parameters 10–100.
    bonus_data: Option<(String, f64)>,
    /// Facility role from `specialAbilityFacilityNew` — CrewCapacity, Refiner,
    /// Mining, SpaceMirrorOrShade, ConstructionEquipment, Bonus, Lab,
    /// BuildSpacecraft, EnergyProduction, EnergyStorage (and "None" → dropped).
    role: Option<String>,
    /// Magnitude for the role above (`specialAbilityParameter`). Meaning is
    /// role-dependent: crew count for habitats, research rate for Lab, mining
    /// rate for Mining, albedo delta for SpaceMirrorOrShade, etc.
    role_magnitude: f64,
    /// Terraforming-relevant deltas pulled from `habitabilityParametersBonus`.
    /// That field is a *dict* keyed by parameter name (temperature, composition,
    /// pressure, gravity, radiation, magneticFieldVisualization, plus a bunch
    /// of flag-like fields — extremeVolcanism etc. — that are NOT deltas).
    /// We surface only the player-facing numeric knobs, with friendly labels.
    /// Empty for everything except a handful of terraforming facilities.
    habitability_deltas: Vec<(String, f64)>,
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
/// (Early Exploration / The Expansion / Colonization Era / Race Beyond).
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
    /// Facilities the company owns at scenario load, as `(build_* id, count)`
    /// pairs sorted by id.  Walked from `objectInfoDatas[].productionItems[]`
    /// — each Facility node carries an `idProductionItemType` String — and
    /// attributed to the corp via the enclosing ObjectInfoData's
    /// `companyId.id`.  Duplicates collapse to a single entry with its count.
    #[serde(default)]
    starting_facilities: Vec<(String, u32)>,
}

/// One pre-built save scenario, listing every playable corp's starting state.
/// `scenario_id` is the `StartGameEpoch_*` id resolved via
/// `PlanetarySystem_Realistic.mapEpochToToStartData` — NOT the (sometimes
/// misnamed) `$name` of the underlying StartGameData asset.
#[derive(Serialize, Debug, Default, PartialEq)]
struct ScenarioStart {
    scenario_id: String, // e.g. "StartGameEpoch_EarlyExploration", "StartGameEpoch_TheExpansion", "StartGameEpoch_Colonization", "StartGameEpoch_RaceBeyond"
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
    /// Distinct non-None `layer` values found across this contract's
    /// objectives.  In production data the only non-None value is `"Asteroid"`
    /// — it signals that the player must have reached the asteroid belt to
    /// progress this contract, regardless of what the `unlock_rewards` chain
    /// claims (some asteroid contracts have no upstream contract at all but
    /// are still gated by getting there).  Used downstream in `gen_pages` to
    /// bump the depth/Order column for these contracts so they appear after
    /// the asteroid-belt gate (`contract_asteroid_first`).
    #[serde(default)]
    objective_layers: Vec<String>,
    /// True iff at least one of this contract's objectives carries
    /// `layer: "None"` (i.e. an objective explicitly NOT gated to any layer).
    /// In production data, the Sirenix dump defaults every objective's `layer`
    /// to `"Asteroid"`; the handful of contracts that have an explicit
    /// `"None"` objective (currently Humans on Mars and Space Dock) are the
    /// ones that bridge from non-asteroid play into the asteroid belt.  This
    /// flag lets `gen_pages` distinguish "fully asteroid-gated" contracts
    /// (no None) from those that mix Earth/Moon/Mars work with asteroid work.
    #[serde(default)]
    has_layer_none_objective: bool,
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

    // bonusData is a single object on every facility; the placeholder bonus
    // ("None") must be dropped so the column renders blank for non-launch rows.
    let bonus_data = v.get("bonusData").and_then(|bd| {
        let bonus = bd.get("bonus").and_then(|x| x.as_str()).unwrap_or("None");
        if bonus.is_empty() || bonus == "None" {
            return None;
        }
        let param = bd
            .get("bonusParameter")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0);
        Some((bonus.to_string(), param))
    });

    // specialAbilityFacilityNew + specialAbilityParameter — the role & its
    // magnitude. "None" placeholder collapses to no-role.
    let role = v
        .get("specialAbilityFacilityNew")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty() && *s != "None")
        .map(|s| s.to_string());
    let role_magnitude = f(&["specialAbilityParameter"]);

    // habitabilityParametersBonus is a dict, not an array. Only a handful of
    // numeric keys are real terraforming deltas; the rest (extremeVolcanism,
    // environmentalToxicity, cryoVolcanism, hydroCarbonLakes) are flag-like
    // and never represent a delta even when set to 1.
    let mut habitability_deltas: Vec<(String, f64)> = Vec::new();
    if let Some(hpb) = v.get("habitabilityParametersBonus") {
        // Order matters for rendering — keep player-facing parameters first,
        // then the magnetic-field knob the terraform-magnet facilities use.
        for (raw_key, label) in [
            ("temperature", "Temperature"),
            ("composition", "Atmosphere"),
            ("pressure", "Pressure"),
            ("gravity", "Gravity"),
            ("water", "Water"),
            ("radiation", "Radiation"),
            ("magneticFieldVisualization", "Magnetic field"),
        ] {
            let val = hpb.get(raw_key).and_then(|x| x.as_f64()).unwrap_or(0.0);
            if val != 0.0 {
                habitability_deltas.push((label.to_string(), val));
            }
        }
    }

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
        build_time_days: f(&["timeToBuildInDays"]),
        bonus_data,
        role,
        role_magnitude,
        habitability_deltas,
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
    let mut objective_layers: Vec<String> = Vec::new();
    let mut has_layer_none_objective = false;
    if let Some(obj_arr) = v.get("objectives").and_then(|x| x.as_array()) {
        for o in obj_arr {
            // Capture the per-objective `layer` regardless of whether the
            // objective itself is well-formed (kind may be empty / skipped).
            // We dedupe while preserving first-seen order.  "None" and empty
            // mean "no layer gating" and are dropped from objective_layers,
            // but we record their presence in has_layer_none_objective so
            // downstream code can distinguish mixed (asteroid + None) contracts
            // from fully asteroid-gated ones.
            if let Some(layer) = o.get("layer").and_then(|x| x.as_str()) {
                if layer.is_empty() || layer == "None" {
                    has_layer_none_objective = true;
                } else if !objective_layers.iter().any(|l| l == layer) {
                    objective_layers.push(layer.to_string());
                }
            }
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
        objective_layers,
        has_layer_none_objective,
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

/// Walk the dump's `PlanetarySystemDescriptor` array, locate the
/// `PlanetarySystem_Realistic` entry, and build a routing dict that maps each
/// `StartGameData $name` to the `StartGameEpoch_*` id that uses it in the Sol
/// (Realistic) timeline. Returns `None` if the descriptor or its
/// `mapEpochToToStartData` field is absent.
///
/// Sirenix encodes dictionary keys as type-qualified strings such as
/// `"StartGameEpoch_EarlyExploration (ScriptableObjectScripts.StartGameConfiguration.StartGameEpoch)"`,
/// so we strip the parenthesised type suffix and trim whitespace to recover the
/// plain epoch id. The dict values are `$ref` objects whose `name` field
/// carries the StartGameData asset $name.
fn build_scenario_routing(
    descriptors: &Value,
) -> Option<std::collections::HashMap<String, String>> {
    let arr = descriptors.as_array()?;
    let realistic = arr.iter().find(|d| {
        d.get("$name").and_then(|v| v.as_str()) == Some("PlanetarySystem_Realistic")
    })?;
    let map = realistic.get("mapEpochToToStartData")?.as_object()?;
    let mut routing = std::collections::HashMap::new();
    for (raw_key, value) in map {
        let epoch_id = raw_key.split_whitespace().next()?.to_string();
        if !epoch_id.starts_with("StartGameEpoch_") {
            continue;
        }
        let save_name = match value.get("name").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        routing.insert(save_name, epoch_id);
    }
    Some(routing)
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
/// `routing` maps the StartGameData $name (e.g. `testStartGAme`,
/// `StartGameColonization`) to the corresponding `StartGameEpoch_*` id from
/// `PlanetarySystem_Realistic.mapEpochToToStartData`. Entries whose $name is
/// not in the routing are skipped — that filters out test/Kepler/Trappist
/// placeholders and the `StartGameDataSceneJSON` non-Sol save.
///
/// Returns `None` if the entry has no populated `companyDataSave` or isn't
/// in the routing.
fn parse_scenario_start(
    entry: &Value,
    routing: &std::collections::HashMap<String, String>,
) -> Option<ScenarioStart> {
    let save_name = entry.get("$name").and_then(|v| v.as_str())?;
    let epoch_id = routing.get(save_name)?.clone();

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

    // Walk the same SerializationNodes again to collect per-corp starting
    // facilities. `objectInfoDatas[]` is a peer of `companyDataSave`; each
    // ObjectInfoData carries `companyId.id` (owning corp) and a
    // `productionItems[]` list of Facility nodes (each with an
    // `idProductionItemType` String).
    let facilities_by_corp = collect_starting_facilities(nodes);
    for corp in corp_starts.iter_mut() {
        if let Some(list) = facilities_by_corp.get(&corp.company_id) {
            corp.starting_facilities = list.clone();
        }
    }

    // Filter out WorldGovernment (the AI faction, not a playable corp).
    corp_starts.retain(|c| c.company_id != "WorldGovernment");
    corp_starts.sort_by(|a, b| a.company_id.cmp(&b.company_id));

    Some(ScenarioStart {
        scenario_id: epoch_id,
        corp_starts,
    })
}

/// Walk a flat SerializationNodes array and build a per-corp facility list.
/// Each Facility node lives inside some ObjectInfoData scope; the owning
/// corp is the `companyId.id` of that enclosing ObjectInfoData.
///
/// Strategy: track depth, remember the most-recent ObjectInfoData's
/// `companyId.id`, then capture every `idProductionItemType` String we see
/// inside that scope.  When we exit the ObjectInfoData, drop the corp until
/// we enter the next one.
///
/// Returns `corp_id → Vec<(facility_id, count)>` sorted by facility id, so
/// downstream consumers get a deterministic order.
fn collect_starting_facilities(nodes: &[Value]) -> std::collections::HashMap<String, Vec<(String, u32)>> {
    use std::collections::HashMap;
    // `obj_stack` records, for each nested ObjectInfoData node currently
    // open, the depth at which it opened plus the corp id (None until we've
    // captured the companyId.id String).  Real dumps don't nest
    // ObjectInfoData entries, but tracking a stack keeps the logic robust
    // against the serializer's quirks.
    let mut depth: i32 = 0;
    let mut obj_stack: Vec<(i32, Option<String>)> = Vec::new();
    // Tracks whether the current `companyId` sub-node belongs to the
    // currently-open ObjectInfoData (vs. some other companyId field nested
    // deeper).  `inside_company_id_field` is true between
    // `companyId StartOfNode` and the matching EndOfNode within the
    // topmost ObjectInfoData scope.
    let mut company_id_open_at: Option<i32> = None;
    let mut counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for n in nodes {
        let e = entry_of(n);
        let nm = name_of(n);
        // Match `ObjectInfoData` exactly — not `ObjectInfoData+ObjectInfoDataIDSave`
        // (a nested type that appears inside RowResourcesData) and not the
        // various List`1 wrappers carrying ObjectInfoData as a generic arg.
        let data = data_of(n);
        let after_pipe = data.find('|').map(|i| &data[i + 1..]).unwrap_or(data);
        let is_object_info_data = e == "StartOfNode"
            && nm.is_empty() // outer ObjectInfoData entries are array items (Name="")
            && (after_pipe.starts_with("ObjectInfoData,")
                || after_pipe.starts_with("Game.ObjectInfoDataScripts.ObjectInfoData,"));

        // Mark entry into ObjectInfoData BEFORE updating depth so the
        // stored depth is the level the node sits at.
        if is_object_info_data {
            obj_stack.push((depth, None));
        }
        // Mark entry into a `companyId` sub-node — only at the top of the
        // currently-open ObjectInfoData (depth == obj_open_depth + 1).
        if let Some(&(obj_depth, ref corp)) = obj_stack.last() {
            if corp.is_none()
                && e == "StartOfNode"
                && nm == "companyId"
                && depth == obj_depth + 1
            {
                company_id_open_at = Some(depth);
            }
        }
        // Capture the `id` String inside the open companyId sub-node.
        if let Some(_cid_depth) = company_id_open_at {
            if e == "String" && nm == "id" {
                if let Some(top) = obj_stack.last_mut() {
                    if top.1.is_none() {
                        top.1 = Some(data_of(n).to_string());
                    }
                }
            }
        }
        // Capture every idProductionItemType String while we're inside an
        // ObjectInfoData scope.  `idProductionItemType` is a Facility-only
        // field, so we can grab it without scoping to `productionItems` /
        // `listFacility`.  Filter to `build_*` ids — that's the facility
        // namespace (vs. spacecraft/launch-vehicle production items).
        if e == "String" && nm == "idProductionItemType" {
            if let Some(&(_, Some(ref corp))) = obj_stack.last() {
                let id = data_of(n).to_string();
                if id.starts_with("build_") {
                    *counts
                        .entry(corp.clone())
                        .or_default()
                        .entry(id)
                        .or_insert(0) += 1;
                }
            }
        }

        // Depth bookkeeping (after field captures, before close handling).
        if e == "StartOfNode" || e == "StartOfArray" {
            depth += 1;
        } else if e == "EndOfNode" || e == "EndOfArray" {
            // Close companyId sub-node when we return to its open depth.
            if let Some(cid_depth) = company_id_open_at {
                if depth == cid_depth {
                    company_id_open_at = None;
                }
            }
            depth -= 1;
            // Close the top ObjectInfoData when we return to (or below) its
            // open depth — i.e., when the matching EndOfNode for the
            // ObjectInfoData itself fires.
            if let Some(&(obj_depth, _)) = obj_stack.last() {
                if depth == obj_depth {
                    obj_stack.pop();
                }
            }
        }
    }

    let mut out: HashMap<String, Vec<(String, u32)>> = HashMap::new();
    for (corp, map) in counts {
        let mut v: Vec<(String, u32)> = map.into_iter().collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        out.insert(corp, v);
    }
    out
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

    // Pre-built starting saves for the Sol (Realistic) timeline. The
    // PlanetarySystem_Realistic descriptor's `mapEpochToToStartData` dict is
    // the source of truth: it maps each playable epoch (Early Exploration,
    // The Expansion, Colonization, Race Beyond) to the StartGameData asset
    // that holds the corresponding pre-rolled save. Note that the asset for
    // Early Exploration is misnamed `testStartGAme` in the dump — the
    // routing tells us it's actually the 2020 save shipped to players.
    let scenario_routing = raw
        .get("PlanetarySystemDescriptor")
        .and_then(build_scenario_routing)
        .unwrap_or_default();
    let mut scenario_starts: Vec<ScenarioStart> = raw
        .get("StartGameData")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| parse_scenario_start(e, &scenario_routing))
                .collect()
        })
        .unwrap_or_default();
    // Order scenarios by in-game timeline
    // (Early Exploration → The Expansion → Colonization Era → Race Beyond).
    let timeline = |s: &ScenarioStart| -> u8 {
        match s.scenario_id.as_str() {
            "StartGameEpoch_EarlyExploration" => 0,
            "StartGameEpoch_TheExpansion" => 1,
            "StartGameEpoch_Colonization" => 2,
            "StartGameEpoch_RaceBeyond" => 3,
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

    /// Routing that mirrors what the real PlanetarySystem_Realistic dump carries:
    /// four StartGameData $name values mapped to their epoch ids.
    fn realistic_routing() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("testStartGAme".to_string(), "StartGameEpoch_EarlyExploration".to_string());
        m.insert("StartGameExpansion".to_string(), "StartGameEpoch_TheExpansion".to_string());
        m.insert("StartGameColonization".to_string(), "StartGameEpoch_Colonization".to_string());
        m.insert("StartGameRaceBeyond".to_string(), "StartGameEpoch_RaceBeyond".to_string());
        m
    }

    #[test]
    fn parses_scenario_start_for_colonization() {
        let routing = realistic_routing();
        let s = parse_scenario_start(&scenario_fixture(), &routing)
            .expect("colonization should parse");
        // scenario_id is now the epoch id, not the StartGameData $name.
        assert_eq!(s.scenario_id, "StartGameEpoch_Colonization");
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
    fn parses_planetary_system_realistic_epoch_routing() {
        // The Sirenix dump's PlanetarySystem_Realistic descriptor has a
        // `mapEpochToToStartData` dict whose keys are stringified type-qualified
        // epoch ids (e.g. "StartGameEpoch_EarlyExploration (…StartGameEpoch)")
        // and whose values are $ref objects with `name` pointing at a
        // StartGameData $name. We extract a `save name → epoch id` routing.
        let descriptors = serde_json::json!([
            {
                "$name": "PlanetarySystem_Dummy",
                "mapEpochToToStartData": {}
            },
            {
                "$name": "PlanetarySystem_Realistic",
                "mapEpochToToStartData": {
                    "StartGameEpoch_EarlyExploration (ScriptableObjectScripts.StartGameConfiguration.StartGameEpoch)": {
                        "$ref": true, "type": "StartGameData", "name": "testStartGAme"
                    },
                    "StartGameEpoch_TheExpansion (ScriptableObjectScripts.StartGameConfiguration.StartGameEpoch)": {
                        "$ref": true, "type": "StartGameData", "name": "StartGameExpansion"
                    },
                    "StartGameEpoch_Colonization (ScriptableObjectScripts.StartGameConfiguration.StartGameEpoch)": {
                        "$ref": true, "type": "StartGameData", "name": "StartGameColonization"
                    },
                    "StartGameEpoch_RaceBeyond (ScriptableObjectScripts.StartGameConfiguration.StartGameEpoch)": {
                        "$ref": true, "type": "StartGameData", "name": "StartGameRaceBeyond"
                    }
                }
            }
        ]);
        let routing = build_scenario_routing(&descriptors)
            .expect("PlanetarySystem_Realistic should be present");
        assert_eq!(routing.len(), 4);
        assert_eq!(
            routing.get("testStartGAme").map(|s| s.as_str()),
            Some("StartGameEpoch_EarlyExploration")
        );
        assert_eq!(
            routing.get("StartGameExpansion").map(|s| s.as_str()),
            Some("StartGameEpoch_TheExpansion")
        );
        assert_eq!(
            routing.get("StartGameColonization").map(|s| s.as_str()),
            Some("StartGameEpoch_Colonization")
        );
        assert_eq!(
            routing.get("StartGameRaceBeyond").map(|s| s.as_str()),
            Some("StartGameEpoch_RaceBeyond")
        );
    }

    /// Fixture that mirrors the real dump's `objectInfoDatas` region:
    /// a peer of `companyDataSave` inside `serializationData.SerializationNodes`.
    /// Each `ObjectInfoData` has a `companyId.id` (the owning corp) and a
    /// `productionItems` list containing `Game.ObjectInfoDataScripts.Facility`
    /// nodes — each with an `idProductionItemType` String (`build_*`).
    ///
    /// This fixture gives NASA two Noble Gas Mines and one Metal Mine, plus
    /// a single Main Building (universal). WorldGovernment is included once
    /// to confirm we drop its facilities.
    fn facilities_fixture() -> Value {
        // Build the SerializationNodes array as JSON text to avoid blowing the
        // `serde_json::json!` macro recursion limit.
        let node = |name: &str, entry: &str, data: &str| {
            serde_json::json!({"Name": name, "Entry": entry, "Data": data})
        };
        let nodes: Vec<Value> = vec![
            // ---- companyDataSave first (matches dump ordering) -----
            node("companyDataSave", "StartOfNode", "0|List`1[CompanyDataSave]"),
            node("", "StartOfArray", "1"),
            node("", "StartOfNode", "1|CompanyDataSave"),
            node("companyID", "StartOfNode", "2|CompanyIDSave"),
            node("id", "String", "NASA"),
            node("", "EndOfNode", ""),
            node("money", "FloatingPoint", "10000000"),
            node("", "EndOfNode", ""),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
            // ---- objectInfoDatas: 2 ObjectInfoData entries --------
            node("objectInfoDatas", "StartOfNode", "100|List`1[ObjectInfoData]"),
            node("", "StartOfArray", "2"),
            // ObjectInfoData #1 — NASA, four facilities
            node("", "StartOfNode", "101|Game.ObjectInfoDataScripts.ObjectInfoData, Assembly-CSharp"),
            node("id", "Integer", "529"),
            node("companyId", "StartOfNode", "102|CompanyIDSave"),
            node("id", "String", "NASA"),
            node("", "EndOfNode", ""),
            node("productionItems", "StartOfNode", "103|List`1[ProductionItem]"),
            node("", "StartOfArray", "4"),
            node("", "StartOfNode", "104|Facility"),
            node("build", "Boolean", "true"),
            node("idProductionItemType", "String", "build_noblegasmine"),
            node("", "EndOfNode", ""),
            node("", "StartOfNode", "105|Facility"),
            node("build", "Boolean", "true"),
            node("idProductionItemType", "String", "build_noblegasmine"),
            node("", "EndOfNode", ""),
            node("", "StartOfNode", "106|Facility"),
            node("build", "Boolean", "true"),
            node("idProductionItemType", "String", "build_metalmine"),
            node("", "EndOfNode", ""),
            node("", "StartOfNode", "107|Facility"),
            node("build", "Boolean", "true"),
            node("idProductionItemType", "String", "build_main"),
            node("", "EndOfNode", ""),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
            node("", "EndOfNode", ""),
            // ObjectInfoData #2 — WorldGovernment, one facility (must be dropped)
            node("", "StartOfNode", "201|Game.ObjectInfoDataScripts.ObjectInfoData, Assembly-CSharp"),
            node("id", "Integer", "530"),
            node("companyId", "StartOfNode", "202|CompanyIDSave"),
            node("id", "String", "WorldGovernment"),
            node("", "EndOfNode", ""),
            node("productionItems", "StartOfNode", "203|List`1[ProductionItem]"),
            node("", "StartOfArray", "1"),
            node("", "StartOfNode", "204|Facility"),
            node("idProductionItemType", "String", "build_hq"),
            node("", "EndOfNode", ""),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
            node("", "EndOfNode", ""),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
        ];
        serde_json::json!({
            "$name": "StartGameColonization",
            "id": "StartGameColonization",
            "serializationData": {
                "SerializationNodes": nodes,
            }
        })
    }

    #[test]
    fn parses_starting_facilities_with_counts() {
        // NASA owns: 2x noble gas mine, 1x metal mine, 1x main building.
        // WorldGovernment is filtered out (we drop the corp entirely), so its
        // build_hq must NOT appear under NASA.
        let mut routing = std::collections::HashMap::new();
        routing.insert("StartGameColonization".to_string(), "StartGameEpoch_Colonization".to_string());
        let s = parse_scenario_start(&facilities_fixture(), &routing)
            .expect("colonization should parse");
        assert_eq!(s.corp_starts.len(), 1, "WorldGovernment must be filtered out");
        let nasa = &s.corp_starts[0];
        assert_eq!(nasa.company_id, "NASA");
        // Counts are aggregated per facility id, sorted by id for determinism.
        assert_eq!(
            nasa.starting_facilities,
            vec![
                ("build_main".to_string(), 1),
                ("build_metalmine".to_string(), 1),
                ("build_noblegasmine".to_string(), 2),
            ]
        );
    }

    #[test]
    fn parse_scenario_start_uses_epoch_id_via_routing() {
        // testStartGAme (the misnamed Early Exploration save) routes through the
        // PlanetarySystem_Realistic map to `StartGameEpoch_EarlyExploration`.
        // We change the fixture's $name to confirm the routing — not $name — drives
        // the resulting scenario_id.
        let mut fixture = scenario_fixture();
        fixture["$name"] = serde_json::Value::String("testStartGAme".into());
        let routing = realistic_routing();
        let s = parse_scenario_start(&fixture, &routing)
            .expect("testStartGAme should route to Early Exploration");
        assert_eq!(s.scenario_id, "StartGameEpoch_EarlyExploration");
    }

    #[test]
    fn parse_scenario_start_skips_unmapped_savedata() {
        // StartGameKepler isn't in PlanetarySystem_Realistic's epoch map, so
        // it gets skipped even if its serializationData were populated.
        let routing = realistic_routing();
        let placeholder = serde_json::json!({
            "$name": "StartGameKepler",
            "serializationData": { "SerializationNodes": [] }
        });
        assert!(parse_scenario_start(&placeholder, &routing).is_none());
        // StartGameData (the empty default) is also unmapped.
        let bare = serde_json::json!({
            "$name": "StartGameData",
            "serializationData": { "SerializationNodes": [] }
        });
        assert!(parse_scenario_start(&bare, &routing).is_none());
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
    fn facility_build_time_days_round_trips() {
        // GroundFacilityDescriptor entries carry timeToBuildInDays at the top
        // level — same shape as SpacecraftDescriptor / LaunchVehicleDescriptor.
        let v = serde_json::json!({
            "id": "build_lab",
            "facilityType": "Other",
            "timeToBuildInDays": 200,
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert_eq!(f.build_time_days, 200.0);
    }

    #[test]
    fn facility_launch_elevator_parses_bonus_data() {
        // build_launch_elevator: bonusData is a *single object*, not an array —
        // {bonus: "LaunchCostOptionInPlanMission", bonusParameter: 10, ...}.
        let v = serde_json::json!({
            "id": "build_launch_elevator",
            "facilityType": "LaunchFacility",
            "bonusData": {
                "bonus": "LaunchCostOptionInPlanMission",
                "bonusParameter": 10,
                "fakeLVId": "id_LV_launch_elevator_Fake",
                "fakeSCId": "",
                "spaceElevatorPrefab3dView": null
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        let (bonus, param) = f.bonus_data.expect("expected bonus_data");
        assert_eq!(bonus, "LaunchCostOptionInPlanMission");
        assert_eq!(param, 10.0);
    }

    #[test]
    fn facility_with_bonus_none_yields_empty_bonus_data() {
        // The dump sets bonus="None" on every non-launch facility — we must NOT
        // surface those (they would render as bogus "—" placeholders).
        let v = serde_json::json!({
            "id": "build_habitat",
            "facilityType": "Habitation",
            "bonusData": {
                "bonus": "None",
                "bonusParameter": 0,
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert!(f.bonus_data.is_none(), "got {:?}", f.bonus_data);
    }

    #[test]
    fn facility_habitat_parses_crew_capacity_role() {
        // build_habitat: specialAbilityFacilityNew=CrewCapacity, parameter=100.
        let v = serde_json::json!({
            "id": "build_habitat",
            "facilityType": "Habitation",
            "specialAbilityFacilityNew": "CrewCapacity",
            "specialAbilityParameter": 100,
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert_eq!(f.role.as_deref(), Some("CrewCapacity"));
        assert_eq!(f.role_magnitude, 100.0);
    }

    #[test]
    fn facility_with_role_none_drops_role() {
        // specialAbilityFacilityNew="None" is the placeholder for facilities
        // that don't carry a magnitude (e.g. telescopes). We treat that as
        // "no role" so the page renders `—` instead of "None".
        let v = serde_json::json!({
            "id": "build_observatory",
            "facilityType": "Other",
            "specialAbilityFacilityNew": "None",
            "specialAbilityParameter": 0,
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert!(f.role.is_none(), "got {:?}", f.role);
    }

    #[test]
    fn facility_magnet_station_parses_habitability_deltas() {
        // build_terraform_magnet: habitabilityParametersBonus is a *dict*, not
        // an array — keys are temperature / composition / pressure / gravity /
        // radiation / magneticFieldVisualization / etc. Only non-zero numeric
        // entries for the four player-facing terraforming knobs (and magnetic
        // field) should be surfaced.
        let v = serde_json::json!({
            "id": "build_terraform_magnet",
            "facilityType": "Other",
            "habitabilityParametersBonus": {
                "temperature": 0,
                "composition": 0,
                "pressure": 0,
                "gravity": 0,
                "water": 0,
                "radiation": -0.6,
                "magneticFieldVisualization": 0.6,
                // Flag-like fields that must NOT appear as deltas:
                "extremeVolcanism": 1,
                "environmentalToxicity": 1,
                "cryoVolcanism": 1,
                "hydroCarbonLakes": 1,
                "albedo": 0,
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        // Find the radiation delta:
        let rad = f
            .habitability_deltas
            .iter()
            .find(|(k, _)| k == "Radiation")
            .map(|(_, v)| *v);
        assert_eq!(rad, Some(-0.6), "deltas={:?}", f.habitability_deltas);
        let mag = f
            .habitability_deltas
            .iter()
            .find(|(k, _)| k == "Magnetic field")
            .map(|(_, v)| *v);
        assert_eq!(mag, Some(0.6), "deltas={:?}", f.habitability_deltas);
        // The volcanism / toxicity flags are NOT terraforming deltas and must
        // be excluded.
        assert!(!f
            .habitability_deltas
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("extremevolcanism")));
    }

    #[test]
    fn facility_without_habitability_bonus_has_empty_deltas() {
        let v = serde_json::json!({
            "id": "build_farm",
            "facilityType": "Production",
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert!(f.habitability_deltas.is_empty());
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

    #[test]
    fn parses_objective_layers_field() {
        // Each objective in the dump carries a `layer` field; "None" or empty
        // string means "no layer gating", anything else (e.g. "Asteroid") means
        // the player needs access to that layer to attempt the objective.  We
        // collect distinct non-None layers across all of a contract's objectives.
        let v = serde_json::json!({
            "id": "contract_asteroid_base",
            "isLocked": false,
            "isFinalContract": false,
            "dateStartActive": "",
            "yearsToExpire": 0,
            "objectives": [
                { "objectiveType": "Possession", "howMuch": 1, "layer": "Asteroid" },
                { "objectiveType": "BuildFacility", "howMuch": 1, "layer": "None" }
            ],
            "rewards": []
        });
        let c = parse_contract(&v).expect("contract should parse");
        assert_eq!(c.objective_layers, vec!["Asteroid".to_string()]);
    }

    #[test]
    fn parses_objective_layers_dedupes() {
        // Multiple objectives with the same non-None layer should produce a
        // single deduped entry in objective_layers.
        let v = serde_json::json!({
            "id": "contract_asteroid_mining",
            "isLocked": false,
            "isFinalContract": false,
            "dateStartActive": "",
            "yearsToExpire": 0,
            "objectives": [
                { "objectiveType": "Possession", "howMuch": 1, "layer": "Asteroid" },
                { "objectiveType": "Possession", "howMuch": 2, "layer": "Asteroid" },
                { "objectiveType": "BuildFacility", "howMuch": 1, "layer": "Asteroid" }
            ],
            "rewards": []
        });
        let c = parse_contract(&v).expect("contract should parse");
        assert_eq!(c.objective_layers, vec!["Asteroid".to_string()]);
    }
}

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
    /// Per-body gravity gate. When set, the rocket can only launch from bodies
    /// whose surface gravity falls in `[min_g, max_g]` (units: g, where Earth
    /// = 1 G). Surfaced from `canBuildParameter.terraformParameterCanBuild.list[]`
    /// entries with `terraformParameterSettingsCanBuild == "Gravity"`. Only the
    /// Al-Ice rockets (`id_Rocket_RocketType5` / `RocketType6`) carry one in the
    /// shipped dump, with a 0..1.8 envelope.
    #[serde(skip_serializing_if = "Option::is_none")]
    gravity_gate: Option<GravityGate>,
}

/// Surface-gravity envelope a rocket can launch from. Values are in g (Earth =
/// 1 G, Mars ≈ 0.38 G, Luna ≈ 0.16 G, Jupiter ≈ 2.5 G).
#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct GravityGate {
    min_g: f64,
    max_g: f64,
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
    /// Era tier (0 early, 1 mid, 2 late) from `stage` in the dump.
    #[serde(default)]
    stage: u8,
    /// Secondary unlock entries from `unlockDataList[]` — everything except
    /// the UnlockContract entries that are already on `contract_unlocks`.
    /// Each entry is a parsed bonus or facility/spacecraft/LV unlock that
    /// stacks on top of the primary `unlockData`.
    #[serde(default)]
    secondary_unlocks: Vec<SecondaryUnlock>,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct SecondaryUnlock {
    /// e.g. "UnlockBonus", "UnlockFacility", "UnlockSpacecraftType", "UnlockVehicleType".
    /// Skip "UnlockContract" — those still flow to contract_unlocks as before.
    action: String,
    /// Target id (e.g., "build_habitatdome", "spacecraft_chem_small"), or "" for pure bonuses.
    target: String,
    /// Bonus name (e.g., "ComponentThrust", "PowerProduction", "BuildCost", "LaunchCost").
    /// Empty for non-bonus actions.
    bonus: String,
    /// Numeric bonus magnitude (e.g., 20 for +20%, or 25 for +25). Zero for non-bonus.
    bonus_parameter: f64,
}

/// One entry from a facility's `canBuildParameter.terraformParameterCanBuild.list`.
/// Each entry is a *gate* on the body's current habitability state: the body's
/// reading on `parameter` must lie within `[min, max]` for construction to be
/// permitted. Real values in the dump cover Pressure (vacuum / atmosphere
/// requirements on launch facilities), Gravity, Radiation, Temperature, Water,
/// Composition, and InternalFlux (geothermal needs an active core).
#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct HabitatConstraint {
    /// Habitability parameter the constraint gates on (e.g., "Pressure", "Temperature").
    parameter: String,
    min: f64,
    max: f64,
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
    /// Per-body build gates pulled from
    /// `canBuildParameter.terraformParameterCanBuild.list`. Each entry says
    /// "the body's `parameter` must read within `[min, max]` for this
    /// facility to be buildable". Empty for the vast majority of facilities —
    /// surfaces vacuum-only launch methods, geothermal's InternalFlux gate,
    /// habitat gravity/radiation/pressure envelopes, the Earth-farm climate
    /// envelope, etc.
    #[serde(default)]
    habitat_constraints: Vec<HabitatConstraint>,
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

/// Per-resource thermal / phase constants from
/// `ResourceDefinition.terraformationInfo`. The values drive Solar Expanse's
/// atmosphere and surface-temperature sim — boiling / melting points, latent
/// heat of vaporization, heat capacity, greenhouse contribution. Units come
/// from inspecting how the C# sim consumes them (see Assembly-CSharp
/// `TerraformationInfoDef`):
///
/// * `boiling_temperature_k` / `melting_temperature_k` — kelvin (used with
///   the gas constant R = 8.314 J/(mol·K) in the Clausius-Clapeyron formula).
/// * `vaporization_latent_heat` — J/mol (divided by R in the same formula).
/// * `pressure_triple_point` — atmospheres (water's 0.00611 atm = 611 Pa,
///   CO2's 5.11 atm both match real values).
/// * `heat_capacity` — specific heat, J/(kg·K) for steam-like resources.
/// * `optical_depth_parameter` — dimensionless greenhouse coefficient
///   (formerly `gasIRAbsorbtionCoefficient`).
///
/// The C# default initializes every field to 1.0 (`TerraformationInfoDef
/// { resourceOpticalDepthParameter = 1.0, … }`), which is the "unset"
/// placeholder for ledger resources like Energy / Human / Supplies that
/// never participate in the atmosphere sim. `parse_resource` drops those
/// rows so the terraforming page only surfaces resources with real physics.
#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct TerraformationInfo {
    optical_depth_parameter: f64,
    heat_capacity: f64,
    vaporization_latent_heat: f64,
    boiling_temperature_k: f64,
    melting_temperature_k: f64,
    pressure_triple_point: f64,
}

#[derive(Serialize, Debug, Default, PartialEq)]
struct Resource {
    id: String,
    resource_type: String,    // Normal / Energy / Human
    market_price_base: f64,
    show_on_ui: bool,
    can_be_left_on_object: bool,
    /// Thermal / phase constants for the terraforming sim. `None` for
    /// resources whose `terraformationInfo` is the C# all-1.0 placeholder
    /// default (ledger resources like Energy / Human / Supplies).
    #[serde(skip_serializing_if = "Option::is_none")]
    terraformation_info: Option<TerraformationInfo>,
}

/// Per-resource thermal / phase constants from
/// `ResourceDefinition.terraformationInfo`. The values drive Solar Expanse's
/// atmosphere and surface-temperature sim — boiling / melting points, latent
/// heat of vaporization, heat capacity, greenhouse contribution. Units come
/// from inspecting how the C# sim consumes them (see Assembly-CSharp
/// `TerraformationInfoDef`):
///
/// * `boiling_temperature_k` / `melting_temperature_k` — kelvin (used with
///   the gas constant R = 8.314 J/(mol·K) in the Clausius-Clapeyron formula).
/// * `vaporization_latent_heat` — J/mol (divided by R in the same formula).
/// * `pressure_triple_point` — atmospheres (water's 0.00611 atm = 611 Pa,
///   CO2's 5.11 atm both match real values).
/// * `heat_capacity` — specific heat (J/(kg·K) for steam-like resources).
/// * `optical_depth_parameter` — dimensionless greenhouse coefficient
///   (formerly `gasIRAbsorbtionCoefficient`).
///
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

/// Initial habitability snapshot for one celestial body at scenario load.
/// Pulled from the `habitabilityParameters` sub-node inside each
/// `ObjectInfoSaves[]` entry of a `StartGameData` save: every body in the
/// loaded scene gets one entry per scenario (Early Exploration / The
/// Expansion / Colonization / Race Beyond).
///
/// The four scenarios capture the campaign timeline at increasing dates, so
/// the values drift across them as bodies are terraformed, mined, or
/// otherwise altered.  Use these to compare conditions across the four
/// pre-built starts.
///
/// Unit conventions observed in production dumps (units inferred from the
/// values for the well-known planets, since the dump itself doesn't carry
/// them):
///   * `temperature` — degrees Celsius. Earth sits at ~2.4 °C, Mars at
///     ~-63 °C, the Moon at ~40 °C (sub-solar mean).
///   * `pressure` — Earth atmospheres. Earth ≈ 0.99, Mars ≈ 0.006.
///   * `gravity` — m/s² (NOT Earth-relative). Earth ≈ 9.79, Mars ≈ 3.71,
///     the Moon ≈ 1.62.
///   * `water`, `composition`, `albedo` — dimensionless 0–1.
///   * `radiation`, `magneticFieldVisualization` — game-specific scale
///     (Earth ≈ 1 radiation, 40 magnetic; the Moon ≈ 12 radiation, 2.5
///     magnetic).
///   * `internalFlux` — heat flux (Earth = 0.08).
///   * `heatCapacityRock`, `totalHeatCapacity`, `temperatureSwings` — raw
///     simulation units.
///   * `mirrorsStrength`, `shadesStrength` — player-driven terraforming
///     deltas; usually 0 at scenario start.
///   * `extremeVolcanism`, `environmentalToxicity`, `cryoVolcanism`,
///     `hydroCarbonLakes` — game-specific scalars (all bodies ship with
///     1.0 in the current data; included for completeness in case a
///     future patch starts differentiating them).
#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct ScenarioBodyHabitability {
    /// Body display name when resolvable via the `PlanetarySystemDescriptor`
    /// id → name map; falls back to the stringified numeric id otherwise.
    body_name: String,
    /// Raw `IDObjectInfo.id` integer from the dump.  Stable across scenarios
    /// for a given body in the Sol-Realistic system.
    body_id: i32,
    /// Temperature in degrees Celsius (in-game UI units; NOT Kelvin).
    #[serde(default)]
    temperature: f64,
    /// Atmospheric composition score, 0–1.
    #[serde(default)]
    composition: f64,
    /// Pressure in atmospheres (Earth ≈ 1.0).
    #[serde(default)]
    pressure: f64,
    /// Surface gravity in Earth gravities (Earth ≈ 1.0).
    #[serde(default)]
    gravity: f64,
    /// Water level, 0–1.
    #[serde(default)]
    water: f64,
    /// Radiation level, 0–1+ (higher = worse).
    #[serde(default)]
    radiation: f64,
    /// Magnetic field strength, 0–1.
    #[serde(default)]
    magnetic_field: f64,
    /// Surface reflectivity, 0–1.
    #[serde(default)]
    albedo: f64,
    /// Internal heat flux from the body's interior.
    #[serde(default)]
    internal_flux: f64,
    /// Crustal heat capacity (raw simulation units).
    #[serde(default)]
    heat_capacity_rock: f64,
    /// Total heat capacity including atmosphere.
    #[serde(default)]
    total_heat_capacity: f64,
    /// Day-night temperature swing magnitude.
    #[serde(default)]
    temperature_swings: f64,
    /// Player mirror-installation strength delta on this body.
    #[serde(default)]
    mirrors_strength: f64,
    /// Player shade-installation strength delta on this body.
    #[serde(default)]
    shades_strength: f64,
    /// Volcanism hazard flag (0/1).
    #[serde(default)]
    extreme_volcanism: f64,
    /// Toxic-atmosphere hazard flag (0/1).
    #[serde(default)]
    environmental_toxicity: f64,
    /// Cryovolcanism hazard flag (0/1).
    #[serde(default)]
    cryo_volcanism: f64,
    /// Hydrocarbon lakes flag (0/1).
    #[serde(default)]
    hydro_carbon_lakes: f64,
}

/// One pre-built save scenario, listing every playable corp's starting state.
/// `scenario_id` is the `StartGameEpoch_*` id resolved via
/// `PlanetarySystem_Realistic.mapEpochToToStartData` — NOT the (sometimes
/// misnamed) `$name` of the underlying StartGameData asset.
#[derive(Serialize, Debug, Default, PartialEq)]
struct ScenarioStart {
    scenario_id: String, // e.g. "StartGameEpoch_EarlyExploration", "StartGameEpoch_TheExpansion", "StartGameEpoch_Colonization", "StartGameEpoch_RaceBeyond"
    corp_starts: Vec<CorpStart>,
    /// Per-body habitability snapshot at scenario start. One entry per
    /// `ObjectInfoSaves[]` element in the StartGameData; sorted by `body_id`
    /// for deterministic output. Empty for older dumps that don't carry the
    /// `habitabilityParameters` sub-node (testStartGAme is one such — it's
    /// the Early Exploration save and its bodies are at default values).
    #[serde(default)]
    body_habitability: Vec<ScenarioBodyHabitability>,
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
    /// Future-dated "this contract isn't even visible until year YYYY"
    /// timestamp, read from `dateTimeStringStart` (format
    /// `YYYY-MM-DD HH:MM:SS`).  Distinct from `date_start_active` (which uses
    /// `MM/DD/YYYY` and is for "this offer first appears on this calendar day
    /// of the player's run").  Only set when `dateTimeStringStartEnable` is
    /// true and the field is non-empty; together with `is_locked` the
    /// renderer uses the year here to drive the Order column for date-locked
    /// contracts (e.g. Exoplanet Search → 2080 → Order 2080).
    #[serde(default)]
    date_time_string_start: Option<String>,
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

/// Spacecraft module whose only job is to ferry humans up.
/// Drawn from `SpaceModuleDescriptor` entries with `specialAbilityFacilityNew == "CrewTransport"`.
#[derive(Serialize, Debug, Default, PartialEq)]
struct CrewTransport {
    id: String,
    /// Humans the module can carry. From `specialAbilityParameter`.
    capacity: i64,
    /// Dry mass in tons. From `mass`.
    mass: f64,
    is_locked: bool,
}

/// Per-celestial-body mining license fees, in dollars per tonne extracted.
/// Sourced from each loaded `ObjectInfo.ResourceMiningLicenseFeePerT`
/// dictionary, which is `[OdinSerialize]` and only populated at runtime —
/// hence why these values come from the BepInEx mod's live MonoBehaviour
/// walk rather than from AssetRipper or any save file. Earth is presently
/// the only body that charges fees; entries with empty `fees_per_t`
/// (e.g. Mars) are still emitted so the renderer can show "no fee" rows.
#[derive(Serialize, Debug, Default, PartialEq)]
struct BodyLicenseFee {
    body_name: String,
    /// Map of resource id (e.g. `"alloy"`, `"fuel"`) → license fee in
    /// dollars per tonne. Populated only when the mod walks live
    /// `ObjectInfo` MonoBehaviours; empty for bodies that don't charge.
    fees_per_t: std::collections::BTreeMap<String, f64>,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct AsteroidClass {
    name: String,
    tiers: Vec<AsteroidTier>,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct AsteroidTier {
    category: String,
    rolls: Vec<AsteroidRoll>,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct AsteroidRoll {
    resource_id: String,
    probability: f64,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct ExoplanetSystem {
    name: String,
    id: String,
    star_type: String,
    second_star_type: Option<String>,
    system_age: String,
    bodies: Vec<ExoplanetBody>,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct ExoplanetBody {
    name: String,
    planet_type: String,
    semi_major_axis_au: f64,
    eccentricity: f64,
    inclination_deg: f64,
    mass_1e24_kg: f64,
    radius_km: f64,
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct AchievementCondition {
    /// Source id of a ContractDefinition that must also be completed.
    /// Empty when this condition isn't a contract-dependency.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    required_contract: String,
    /// In-game year by which the achievement must be earned (derived from
    /// `maxTime._DateTime`).  0 when no deadline is set.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    before_year: i32,
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

#[derive(Serialize, Debug, Default, PartialEq, Clone)]
struct Achievement {
    id: String,
    name: String,
    source_type: String,
    source_id: String,
    description: String,
    /// Extra requirements beyond completing the parent contract — year
    /// deadlines and/or required prior contracts.  Empty for spacecraft /
    /// LV-sourced achievements and for unconditional contract achievements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    conditions: Vec<AchievementCondition>,
}

#[derive(Serialize)]
struct Sirenix {
    spacecraft: Vec<Spacecraft>,
    launch_vehicles: Vec<LaunchVehicle>,
    research: Vec<Research>,
    facilities: Vec<Facility>,
    space_components: Vec<SpaceComponent>,
    crew_transports: Vec<CrewTransport>,
    resources: Vec<Resource>,
    contracts: Vec<Contract>,
    scenario_starts: Vec<ScenarioStart>,
    epochs: Vec<Epoch>,
    /// Per-body mining license fees. Empty when the dump predates the
    /// BepInEx mod's `ObjectInfo` walk; otherwise one entry per body in
    /// the loaded scene, sorted by `body_name`.
    license_fees: Vec<BodyLicenseFee>,
    #[serde(default)]
    asteroid_classes: Vec<AsteroidClass>,
    #[serde(default)]
    exoplanet_systems: Vec<ExoplanetSystem>,
    #[serde(default)]
    achievements: Vec<Achievement>,
}

/// Extract the achievement id from a single steamAchievement binding object.
/// Returns `None` when the binding's inner `achievement` is null (a
/// placeholder slot) or when the name string is missing/empty.
fn extract_binding_achievement_id(binding: &Value) -> Option<String> {
    binding
        .pointer("/achievement/name")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Parse the `conditions[]` array on a steamAchievement binding.  Each
/// element becomes an `AchievementCondition`; we skip elements that yield
/// nothing useful (no contract ref AND no real maxTime year).
///
/// The dump's `maxTime` is a `_DateTime` ISO string; the sentinel for "no
/// deadline" is `0001-01-01`, while real deadlines are years like `2400`
/// or `2600`.  Anything before year 100 is treated as the sentinel.
fn parse_achievement_conditions(binding: &Value) -> Vec<AchievementCondition> {
    let Some(arr) = binding.get("conditions").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|cond| {
            let required_contract = cond
                .pointer("/contract/name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let before_year = cond
                .pointer("/maxTime/_DateTime")
                .and_then(|x| x.as_str())
                .and_then(|s| s.get(..4))
                .and_then(|s| s.parse::<i32>().ok())
                .filter(|&y| y >= 100)
                .unwrap_or(0);
            if required_contract.is_empty() && before_year == 0 {
                return None;
            }
            Some(AchievementCondition {
                required_contract,
                before_year,
            })
        })
        .collect()
}

/// Walk `ContractDefinition.steamAchievements[]` and produce one `Achievement`
/// row per (contract id, non-null binding) pair.  Empty when the contract has
/// no `steamAchievements` field, an empty array, or every binding is a
/// placeholder.
fn parse_contract_achievements(v: &Value) -> Vec<Achievement> {
    let Some(id) = v.get("id").and_then(|x| x.as_str()) else {
        return Vec::new();
    };
    let Some(arr) = v.get("steamAchievements").and_then(|x| x.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|binding| {
            let ach_id = extract_binding_achievement_id(binding)?;
            Some(Achievement {
                id: ach_id,
                name: String::new(),
                source_type: "contract".to_string(),
                source_id: id.to_string(),
                description: String::new(),
                conditions: parse_achievement_conditions(binding),
            })
        })
        .collect()
}

/// SpacecraftType[].steamAchievement is a *single* binding object (not an
/// array).  Return one `Achievement` row when the inner `achievement` is
/// non-null, else `None`.
fn parse_spacecraft_achievement(v: &Value) -> Option<Achievement> {
    let id = v.get("id").and_then(|x| x.as_str())?;
    let binding = v.get("steamAchievement")?;
    let ach_id = extract_binding_achievement_id(binding)?;
    Some(Achievement {
        id: ach_id,
        name: String::new(),
        source_type: "spacecraft".to_string(),
        source_id: id.to_string(),
        description: String::new(),
        conditions: parse_achievement_conditions(binding),
    })
}

/// LaunchVehicleType[].steamAchievement is shaped identically to the
/// SpacecraftType field.  As of the current dump no LV actually populates a
/// non-null inner achievement, but we still parse the field so the page
/// would render automatically if the game added one.
fn parse_launch_vehicle_achievement(v: &Value) -> Option<Achievement> {
    let id = v.get("id").and_then(|x| x.as_str())?;
    let binding = v.get("steamAchievement")?;
    let ach_id = extract_binding_achievement_id(binding)?;
    Some(Achievement {
        id: ach_id,
        name: String::new(),
        source_type: "launch_vehicle".to_string(),
        source_id: id.to_string(),
        description: String::new(),
        conditions: parse_achievement_conditions(binding),
    })
}

/// Walk all three sources, dedupe exact (source_type, source_id, id) triples,
/// and emit a stable order: contract → spacecraft → launch_vehicle, then by
/// source_id, then by achievement id.
fn collect_achievements(raw: &Value) -> Vec<Achievement> {
    let mut out: Vec<Achievement> = Vec::new();

    if let Some(arr) = raw.get("ContractDefinition").and_then(|v| v.as_array()) {
        for v in arr {
            out.extend(parse_contract_achievements(v));
        }
    }
    if let Some(arr) = raw.get("SpacecraftType").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(a) = parse_spacecraft_achievement(v) {
                out.push(a);
            }
        }
    }
    if let Some(arr) = raw.get("LaunchVehicleType").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(a) = parse_launch_vehicle_achievement(v) {
                out.push(a);
            }
        }
    }

    // Dedupe exact (source_type, source_id, id) triples.
    let mut seen: std::collections::BTreeSet<(String, String, String)> =
        std::collections::BTreeSet::new();
    out.retain(|a| seen.insert((a.source_type.clone(), a.source_id.clone(), a.id.clone())));

    // Deterministic order for rendering.
    let order_key = |a: &Achievement| -> u8 {
        match a.source_type.as_str() {
            "contract" => 0,
            "spacecraft" => 1,
            "launch_vehicle" => 2,
            _ => 99,
        }
    };
    out.sort_by(|a, b| {
        order_key(a)
            .cmp(&order_key(b))
            .then(a.source_id.cmp(&b.source_id))
            .then(a.id.cmp(&b.id))
    });
    out
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

    // canBuildParameter.terraformParameterCanBuild.list[] carries per-body
    // build/launch gates — habitability-range envelopes the body must satisfy.
    // For LVs only Gravity entries are surfaced (Al-Ice rockets carry a
    // 0..1.8 G cap). Other parameters (Pressure / Temperature / …) are
    // ignored here on purpose — they don't appear on launch vehicles in the
    // shipped dump and adding them speculatively risks lying about envelopes
    // the game might interpret differently for LVs vs. facilities.
    // If multiple Gravity entries appear we take the intersection (max of
    // mins, min of maxes) as the effective gate.
    let mut gravity_gate: Option<GravityGate> = None;
    if let Some(list) = v
        .pointer("/canBuildParameter/terraformParameterCanBuild/list")
        .and_then(|x| x.as_array())
    {
        for entry in list {
            let param = entry
                .get("terraformParameterSettingsCanBuild")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            if param != "Gravity" {
                continue;
            }
            let min = entry.get("min").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let max = entry.get("max").and_then(|x| x.as_f64()).unwrap_or(0.0);
            gravity_gate = Some(match gravity_gate {
                None => GravityGate { min_g: min, max_g: max },
                Some(g) => GravityGate {
                    min_g: g.min_g.max(min),
                    max_g: g.max_g.min(max),
                },
            });
        }
    }

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
        gravity_gate,
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
    //
    // While walking `unlockDataList[]`, also collect everything *else* (bonuses,
    // facility/spacecraft/LV unlocks) into `secondary_unlocks` so the wiki can
    // surface stacked bonuses next to the primary unlock.  Contract entries
    // continue to flow into `contract_unlocks` only — they're already covered
    // by the contracts page.
    let mut contract_unlocks: Vec<String> = Vec::new();
    let mut secondary_unlocks: Vec<SecondaryUnlock> = Vec::new();
    if action == "UnlockContract" && !parameter1.is_empty() {
        contract_unlocks.push(parameter1.to_string());
    }
    if let Some(list) = v.get("unlockDataList").and_then(|x| x.as_array()) {
        for entry in list {
            let act = entry
                .get("actionUnlock")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            if act == "UnlockContract" {
                if let Some(p1) = entry
                    .get("parameter1")
                    .and_then(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                {
                    contract_unlocks.push(p1.to_string());
                }
                continue;
            }
            if act.is_empty() || act == "None" {
                continue;
            }
            let target = entry
                .get("parameter1")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let bonus = entry
                .get("bonus")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty() && *s != "None")
                .unwrap_or("")
                .to_string();
            let bonus_parameter = entry
                .get("bonusParameter")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            // For pure bonus entries `parameter1` is just a Roman-numeral tier
            // label (e.g. "III"), not a build/lv/sc id — drop it from target so
            // the renderer doesn't try to resolve it.
            let target = if act == "UnlockBonus" { String::new() } else { target };
            secondary_unlocks.push(SecondaryUnlock {
                action: act.to_string(),
                target,
                bonus,
                bonus_parameter,
            });
        }
    }

    let stage = v
        .get("stage")
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as u8;

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
        stage,
        secondary_unlocks,
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

    // canBuildParameter — per-body build gates. We surface two flavors here:
    //   * terraformParameterCanBuild.list[]: habitability-range gates
    //     (Pressure / Temperature / Gravity / Radiation / Water /
    //     Composition / InternalFlux), each with a [min, max] envelope.
    //   * canBuildBy="ObjectTypes" + objectTypes!="Planet": object-kind
    //     gates (currently only "Asteroid" — used by build_asteroid_engine_facility).
    //     Modeled as a constraint with parameter="ObjectType" and the type
    //     name folded into the min/max-less label downstream.
    // Most facilities ship with no gates at all; we emit an empty list then.
    let mut habitat_constraints: Vec<HabitatConstraint> = Vec::new();
    if let Some(list) = v
        .pointer("/canBuildParameter/terraformParameterCanBuild/list")
        .and_then(|x| x.as_array())
    {
        for entry in list {
            let Some(parameter) = entry
                .get("terraformParameterSettingsCanBuild")
                .and_then(|x| x.as_str())
            else {
                continue;
            };
            if parameter.is_empty() || parameter == "None" {
                continue;
            }
            let min = entry.get("min").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let max = entry.get("max").and_then(|x| x.as_f64()).unwrap_or(0.0);
            habitat_constraints.push(HabitatConstraint {
                parameter: parameter.to_string(),
                min,
                max,
            });
        }
    }
    // Object-kind gate: when canBuildBy=="ObjectTypes" the facility is only
    // buildable on the listed object type (e.g. Asteroid). We surface that
    // with a synthetic "ObjectType" parameter so the renderer can pick a
    // friendly label without leaking the raw canBuildBy enum.
    let can_build_by = v
        .pointer("/canBuildParameter/canBuildBy")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    if can_build_by == "ObjectTypes" {
        if let Some(obj_type) = v
            .pointer("/canBuildParameter/objectTypes")
            .and_then(|x| x.as_str())
        {
            if !obj_type.is_empty() && obj_type != "None" && obj_type != "Planet" {
                habitat_constraints.push(HabitatConstraint {
                    parameter: format!("ObjectType:{obj_type}"),
                    min: 0.0,
                    max: 0.0,
                });
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
        habitat_constraints,
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

/// Parse a single `ObjectInfo` entry from the BepInEx mod's runtime walk into
/// a `BodyLicenseFee`. The entry has the shape:
///   {"name": "Earth", "resourceMiningLicenseFeePerT": {"id_resource_alloy": 30, ...}}
///
/// Resource ids may carry the `id_resource_` prefix (from
/// `MyIDScriptableObject.ID`); we strip it to align with `parse_resource`'s
/// normalized resource id space. Returns `None` when the entry is missing a
/// `name` field — that's never happened in practice but lets the caller
/// gracefully skip a malformed row instead of poisoning the whole vec.
fn parse_body_license_fee(v: &Value) -> Option<BodyLicenseFee> {
    let body_name = v.get("name")?.as_str()?.to_string();
    if body_name.is_empty() {
        return None;
    }
    let mut fees_per_t = std::collections::BTreeMap::new();
    if let Some(map) = v
        .get("resourceMiningLicenseFeePerT")
        .and_then(|x| x.as_object())
    {
        for (k, val) in map {
            let Some(amount) = val.as_f64() else { continue };
            if amount == 0.0 {
                continue;
            }
            let res_id = normalize_resource_id(k);
            fees_per_t.insert(res_id, amount);
        }
    }
    Some(BodyLicenseFee {
        body_name,
        fees_per_t,
    })
}

/// Parse a single `ObjectSubType` entry into an asteroid class roll table.
/// Filters to entries whose id starts with `ObjectSubType.Asteroid` — that
/// prefix is the marker for the five asteroid classes (Carbon, Dark,
/// Helium3, Metal, Stone). Returns `None` for anything else, including
/// blank-id entries (the dump emits placeholder ObjectSubType rows with
/// empty ids).
///
/// The shape in the dump is `miningFactors[][]` — an array of *buckets*,
/// each holding a list of `{Category, probability, ResourceDefinition.name}`
/// rolls. The `Category` field on each roll ("High" / "Mid" / "Low") is the
/// quality tier of the resulting deposit, NOT the bucket index — AsteroidMetal
/// for instance has its High roll in bucket[1], not bucket[0]. We group
/// strictly by `Category`.  Empty buckets are dropped; tiers are emitted in
/// a stable High → Mid → Low order regardless of dump bucket order.
fn parse_asteroid_class(v: &Value) -> Option<AsteroidClass> {
    let id = v.get("id")?.as_str()?;
    let name = id.strip_prefix("ObjectSubType.Asteroid")?;
    if name.is_empty() {
        return None;
    }

    let mut by_category: std::collections::BTreeMap<String, Vec<AsteroidRoll>> =
        std::collections::BTreeMap::new();

    for bucket in v
        .get("miningFactors")
        .and_then(|x| x.as_array())
        .into_iter()
        .flatten()
    {
        let Some(entries) = bucket.as_array() else { continue };
        for entry in entries {
            let Some(category) = entry.get("Category").and_then(|x| x.as_str()) else {
                continue;
            };
            let Some(rid) = entry
                .pointer("/ResourceDefinition/name")
                .and_then(|x| x.as_str())
            else {
                continue;
            };
            let probability = entry
                .get("probability")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            by_category
                .entry(category.to_string())
                .or_default()
                .push(AsteroidRoll {
                    resource_id: normalize_resource_id(rid),
                    probability,
                });
        }
    }

    // Emit tiers in canonical High → Mid → Low order, dropping any others
    // (defensive — the dump has only ever shown those three values).
    let order = ["High", "Mid", "Low"];
    let mut tiers: Vec<AsteroidTier> = Vec::new();
    for cat in order.iter() {
        if let Some(rolls) = by_category.remove(*cat) {
            if !rolls.is_empty() {
                tiers.push(AsteroidTier {
                    category: (*cat).to_string(),
                    rolls,
                });
            }
        }
    }

    Some(AsteroidClass {
        name: name.to_string(),
        tiers,
    })
}

fn parse_crew_transport(v: &Value) -> Option<CrewTransport> {
    let ability = v.get("specialAbilityFacilityNew").and_then(|x| x.as_str()).unwrap_or("");
    if ability != "CrewTransport" {
        return None;
    }
    let id = v.get("id")?.as_str()?.to_string();
    if id.is_empty() {
        return None;
    }
    let capacity = v.get("specialAbilityParameter").and_then(|x| x.as_i64()).unwrap_or(0);
    let mass = lookup_f64(v, &["mass"]).unwrap_or(0.0);
    let is_locked = lookup_bool(v, &["isLocked"]).unwrap_or(false);
    Some(CrewTransport { id, capacity, mass, is_locked })
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
        terraformation_info: parse_terraformation_info(v.get("terraformationInfo")),
    })
}

/// Parse `terraformationInfo`. Returns `None` when the field is missing or
/// carries the C# all-1.0 placeholder default (ledger resources never set
/// real values); otherwise returns the populated struct.
fn parse_terraformation_info(v: Option<&Value>) -> Option<TerraformationInfo> {
    let obj = v?;
    let ti = TerraformationInfo {
        optical_depth_parameter: lookup_f64(obj, &["resourceOpticalDepthParameter"]).unwrap_or(0.0),
        heat_capacity: lookup_f64(obj, &["resourceHeatCapacity"]).unwrap_or(0.0),
        vaporization_latent_heat: lookup_f64(obj, &["vaporizationLatentHeat"]).unwrap_or(0.0),
        boiling_temperature_k: lookup_f64(obj, &["baseTemperatureBoiling"]).unwrap_or(0.0),
        melting_temperature_k: lookup_f64(obj, &["temperatureMelting"]).unwrap_or(0.0),
        pressure_triple_point: lookup_f64(obj, &["pressureTriplePoint"]).unwrap_or(0.0),
    };
    // The C# default initializes every field to 1.0. Treat that exact
    // signature as "unset" and drop the row. Any resource with real physics
    // overrides at least one field to a non-1.0 value (water has 373 K
    // boiling, hydrogen 14 K melting, etc.).
    let is_placeholder = ti.optical_depth_parameter == 1.0
        && ti.heat_capacity == 1.0
        && ti.vaporization_latent_heat == 1.0
        && ti.boiling_temperature_k == 1.0
        && ti.melting_temperature_k == 1.0
        && ti.pressure_triple_point == 1.0;
    if is_placeholder {
        return None;
    }
    Some(ti)
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
    // dateTimeStringStart: "YYYY-MM-DD HH:MM:SS" — empty string means "no
    // future-dated lockout"; normalize to None so the renderer can fall back
    // to natural depth ordering for those contracts.
    let date_time_string_start = v
        .get("dateTimeStringStart")
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
        date_time_string_start,
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
    parse_scenario_start_with_body_names(entry, routing, &std::collections::HashMap::new())
}

/// Same as `parse_scenario_start`, but also fills each body_habitability
/// entry's `body_name` from the supplied id → display-name resolver. Pass an
/// empty map to fall back to the numeric body id stringified.
fn parse_scenario_start_with_body_names(
    entry: &Value,
    routing: &std::collections::HashMap<String, String>,
    body_names: &std::collections::HashMap<i32, String>,
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

    // Per-body starting habitability — one entry per `ObjectInfoSaves[]`
    // element. Sorted by body_id for deterministic output.
    let body_habitability = collect_body_habitability(nodes, body_names);

    Some(ScenarioStart {
        scenario_id: epoch_id,
        corp_starts,
        body_habitability,
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

/// Walk the SerializationNodes stream and pull out a per-body habitability
/// snapshot — one entry per `ObjectInfoSaves[]` element.
///
/// The stream shape we're matching looks like:
/// ```text
/// ObjectInfoSaves StartOfNode
///   StartOfArray <count>
///     StartOfNode <ObjectInfoSave>
///       IDObjectInfo StartOfNode
///         id Integer <body_id>          ← body id we capture
///       EndOfNode
///       ...
///       habitabilityParameters StartOfNode
///         temperature   FloatingPoint <val>
///         composition   FloatingPoint <val>
///         pressure      FloatingPoint <val>
///         ...
///       EndOfNode
///     EndOfNode
///     ...
///   EndOfArray
/// EndOfNode
/// ```
///
/// We track three pieces of state as we walk:
///   * `in_obj_save_at` — depth at which the current ObjectInfoSave opened
///     (None outside any ObjectInfoSave), so we can scope id capture to its
///     direct `IDObjectInfo` child rather than other random `id` Integers
///     nested deeper.
///   * `in_id_for_object_info_at` — depth at which the current IDObjectInfo
///     opened, so we capture the right `id`.
///   * `in_habit_at` — depth at which the current `habitabilityParameters`
///     opened, so we only capture FloatingPoint fields that sit directly
///     inside the habitabilityParameters scope (depth == in_habit_at + 1).
///
/// Bodies whose `habitabilityParameters` block is missing (some scenarios'
/// minor bodies) are dropped — emitting them with all-zero values would
/// be noise. Sorted by body_id for stable downstream rendering.
fn collect_body_habitability(
    nodes: &[Value],
    body_names: &std::collections::HashMap<i32, String>,
) -> Vec<ScenarioBodyHabitability> {
    let mut out: Vec<ScenarioBodyHabitability> = Vec::new();

    let mut depth: i32 = 0;
    // Each ObjectInfoSave we're currently inside (just one in practice, but
    // tracking a stack keeps the logic robust). Carries the open depth plus
    // the in-progress entry once we've captured its body id.
    let mut obj_stack: Vec<(i32, Option<ScenarioBodyHabitability>)> = Vec::new();
    // Depth at which the current ObjectInfoSave's IDObjectInfo sub-node
    // opened. None when we're not inside an IDObjectInfo scope.
    let mut in_id_for_object_info_at: Option<i32> = None;
    // Depth at which the current habitabilityParameters sub-node opened.
    let mut in_habit_at: Option<i32> = None;
    // Whether we've already seen `habitabilityParameters` for the topmost
    // ObjectInfoSave — guards against re-capturing if some future dump
    // version emits a second habitability block (it doesn't currently).
    let mut habit_filled = false;

    for n in nodes {
        let e = entry_of(n);
        let nm = name_of(n);
        let data = data_of(n);

        // ObjectInfoSave entries are array items, so their Name is empty and
        // the Data field's type suffix carries the class name.
        let after_pipe = data.find('|').map(|i| &data[i + 1..]).unwrap_or(data);
        let is_object_info_save = e == "StartOfNode"
            && nm.is_empty()
            && after_pipe.starts_with("Manager.SaveGameData+ObjectInfoSave");

        if is_object_info_save {
            obj_stack.push((depth, None));
            habit_filled = false;
        }

        // Open IDObjectInfo (must be a direct child of the current ObjectInfoSave).
        if let Some(&(obj_depth, _)) = obj_stack.last() {
            if e == "StartOfNode" && nm == "IDObjectInfo" && depth == obj_depth + 1 {
                in_id_for_object_info_at = Some(depth);
            }
        }
        // Capture body id from `id Integer <n>` inside the open IDObjectInfo.
        if in_id_for_object_info_at.is_some()
            && e == "Integer"
            && nm == "id"
        {
            if let Ok(body_id) = data.parse::<i32>() {
                if let Some(top) = obj_stack.last_mut() {
                    if top.1.is_none() {
                        let body_name = body_names
                            .get(&body_id)
                            .cloned()
                            .unwrap_or_else(|| body_id.to_string());
                        top.1 = Some(ScenarioBodyHabitability {
                            body_id,
                            body_name,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Open habitabilityParameters (must be a direct child of the
        // current ObjectInfoSave, depth == obj_depth + 1).
        if let Some(&(obj_depth, _)) = obj_stack.last() {
            if e == "StartOfNode"
                && nm == "habitabilityParameters"
                && depth == obj_depth + 1
                && !habit_filled
            {
                in_habit_at = Some(depth);
            }
        }

        // Capture floating-point fields that sit DIRECTLY inside the
        // habitabilityParameters block (depth == in_habit_at + 1). Nested
        // wrapper nodes (temperatureWithAtmosphereOld is a Nullable double
        // wrapper at depth +1, with the actual FloatingPoint at +2) are
        // skipped by the depth check.
        if let Some(hd) = in_habit_at {
            if e == "FloatingPoint" && depth == hd + 1 {
                if let Ok(v) = data.parse::<f64>() {
                    if let Some(top) = obj_stack.last_mut() {
                        if let Some(b) = top.1.as_mut() {
                            match nm {
                                "temperature" => b.temperature = v,
                                "composition" => b.composition = v,
                                "pressure" => b.pressure = v,
                                "gravity" => b.gravity = v,
                                "water" => b.water = v,
                                "radiation" => b.radiation = v,
                                "magneticFieldVisualization" => b.magnetic_field = v,
                                "albedo" => b.albedo = v,
                                "internalFlux" => b.internal_flux = v,
                                "heatCapacityRock" => b.heat_capacity_rock = v,
                                "totalHeatCapacity" => b.total_heat_capacity = v,
                                "temperatureSwings" => b.temperature_swings = v,
                                "mirrorsStrength" => b.mirrors_strength = v,
                                "shadesStrength" => b.shades_strength = v,
                                "extremeVolcanism" => b.extreme_volcanism = v,
                                "environmentalToxicity" => b.environmental_toxicity = v,
                                "cryoVolcanism" => b.cryo_volcanism = v,
                                "hydroCarbonLakes" => b.hydro_carbon_lakes = v,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Depth bookkeeping — must run after field captures, before
        // close-handling so the open-depth checks above see the pre-update
        // depth.
        if e == "StartOfNode" || e == "StartOfArray" {
            depth += 1;
        } else if e == "EndOfNode" || e == "EndOfArray" {
            // Close inner scopes when we return to or below their open depth.
            if let Some(hd) = in_habit_at {
                if depth == hd {
                    in_habit_at = None;
                    habit_filled = true;
                }
            }
            if let Some(idod) = in_id_for_object_info_at {
                if depth == idod {
                    in_id_for_object_info_at = None;
                }
            }
            depth -= 1;
            // Close the topmost ObjectInfoSave when we return to its open
            // depth — i.e., the matching EndOfNode for the save itself.
            if let Some(&(obj_depth, _)) = obj_stack.last() {
                if depth == obj_depth {
                    let (_, entry) = obj_stack.pop().unwrap();
                    if let Some(body) = entry {
                        // Only surface bodies whose habitabilityParameters
                        // block was actually present — habit_filled signals
                        // we walked through (and out of) it. Bodies without
                        // a habitabilityParameters sub-node (old-style
                        // saves, debug placeholders) are dropped.
                        if habit_filled {
                            out.push(body);
                        }
                    }
                    habit_filled = false;
                }
            }
        }
    }

    out.sort_by_key(|b| b.body_id);
    out
}

/// Build the `body_id` → display-name map used by `collect_body_habitability`.
///
/// The PlanetarySystem_Dummy descriptor's `solarSystemData.tabObjectInfoData`
/// list is the only place in the dump that carries a complete planet/moon
/// id-to-name mapping (the Realistic descriptor's list is empty — it gets
/// populated at runtime). Each entry has:
///   * `objectInfoId` — the `IDObjectInfo.id` integer used elsewhere.
///   * `idTranslation` — a localization key like
///     `"CelestialBodiesNames.Earth"`; we strip the `CelestialBodiesNames.`
///     prefix to get the plain body name ("Earth").
///   * `customName` — overrides idTranslation when non-empty.
///
/// Asteroids and exotic bodies aren't in this list, so the resolver returns
/// `None` for them and the caller falls back to the numeric id.
fn build_body_name_map(
    descriptors: &Value,
) -> std::collections::HashMap<i32, String> {
    let mut map = std::collections::HashMap::new();
    let arr = match descriptors.as_array() {
        Some(a) => a,
        None => return map,
    };
    // Prefer Dummy (the most complete list); fall back to others if Dummy
    // is missing entirely. Within each descriptor's list, later entries
    // override earlier ones — harmless in practice since ids are unique.
    let prefer_order = |name: &str| -> u8 {
        match name {
            "PlanetarySystem_Dummy" => 0,
            "PlanetarySystem_Realistic" => 1,
            _ => 2,
        }
    };
    let mut ordered: Vec<&Value> = arr.iter().collect();
    ordered.sort_by_key(|d| {
        prefer_order(d.get("$name").and_then(|v| v.as_str()).unwrap_or(""))
    });
    for d in ordered {
        let tab = match d
            .pointer("/solarSystemData/tabObjectInfoData")
            .and_then(|v| v.as_array())
        {
            Some(t) => t,
            None => continue,
        };
        for row in tab {
            let id = match row.get("objectInfoId").and_then(|v| v.as_i64()) {
                Some(i) => i as i32,
                None => continue,
            };
            let custom = row
                .get("customName")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let name = if !custom.is_empty() {
                custom.to_string()
            } else {
                let raw = row
                    .get("idTranslation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                raw.strip_prefix("CelestialBodiesNames.")
                    .unwrap_or(raw)
                    .to_string()
            };
            if name.is_empty() {
                continue;
            }
            // Only insert if absent so the preferred-descriptor row wins.
            map.entry(id).or_insert(name);
        }
    }
    map
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

/// Map a `PlanetarySystemDescriptor.$name` to the in-game player-facing
/// system label. The dump's identifiers are dev-internal (and the Trappist
/// one is misspelled "Trapist" in some `StartGame*` siblings); the names
/// below match what the player sees in the new-game scenario picker and in
/// `CelestialBodiesNames` translation keys for the system's planets.
fn exoplanet_display_name(dump_name: &str) -> Option<&'static str> {
    match dump_name {
        "PlanetarySystem_Trappist" => Some("Trappist-1"),
        "PlanetarySystem_Kepler90" => Some("Kepler-90"),
        "PlanetarySystem_TauCeti" => Some("Tau Ceti"),
        "PlanetarySystem_ProximaCentauri" => Some("Proxima Centauri"),
        _ => None,
    }
}

/// Strip the `startype_` prefix from a `StarTypeDefinition` reference,
/// yielding just the spectral class (e.g. `startype_M8` → `"M8"`,
/// `startype_G2` → `"G2"`).
fn star_class_from_ref(s: &str) -> String {
    s.strip_prefix("startype_").unwrap_or(s).to_string()
}

/// Parse one body entry from `solarSystemData.tabObjectInfoData[]`. Returns
/// `None` if the entry is missing the orbital sub-block we need.
fn parse_exoplanet_body(v: &Value) -> Option<ExoplanetBody> {
    let nbody = v.get("addNBodyData")?;
    let name = nbody.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if name.is_empty() {
        return None;
    }
    let planet_type = v
        .pointer("/planetType/name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let f = |k: &str| -> f64 { nbody.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0) };
    Some(ExoplanetBody {
        name,
        planet_type,
        semi_major_axis_au: f("a"),
        eccentricity: f("e"),
        // Dump field is misspelled `inclication` — match it verbatim.
        inclination_deg: f("inclication"),
        mass_1e24_kg: f("massIn1Pow24"),
        radius_km: f("radiusKM"),
    })
}

/// Parse a `PlanetarySystemDescriptor` entry into an `ExoplanetSystem`,
/// returning `None` if the entry isn't one of the four shipped exoplanet
/// systems or if its `solarSystemData.star1` is null (the `PerfectSystem`
/// stub has no star and no bodies, so we skip it).
fn parse_exoplanet_system(v: &Value) -> Option<ExoplanetSystem> {
    let dump_name = v.get("$name").and_then(|x| x.as_str())?;
    let display = exoplanet_display_name(dump_name)?;
    let data = v.get("solarSystemData")?;
    let star1 = data
        .pointer("/star1/name")
        .and_then(|x| x.as_str())
        .map(star_class_from_ref)?;
    let second = data
        .pointer("/star2/name")
        .and_then(|x| x.as_str())
        .map(star_class_from_ref);
    let system_age = data
        .get("systemAge")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let bodies: Vec<ExoplanetBody> = data
        .get("tabObjectInfoData")
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(parse_exoplanet_body).collect())
        .unwrap_or_default();
    Some(ExoplanetSystem {
        name: display.to_string(),
        id: dump_name.to_string(),
        star_type: star1,
        second_star_type: second,
        system_age,
        bodies,
    })
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

    let mut crew_transports: Vec<CrewTransport> = raw
        .get("SpaceModuleDescriptor")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_crew_transport).collect())
        .unwrap_or_default();
    crew_transports.sort_by(|a, b| a.capacity.cmp(&b.capacity).then(a.id.cmp(&b.id)));

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
    // The Dummy descriptor's tabObjectInfoData carries the only complete
    // body-id → display-name mapping in the dump.  Empty map is fine — the
    // parser falls back to stringified numeric ids for any unmapped body
    // (asteroids and other exotic bodies aren't in the list either way).
    let body_name_map = raw
        .get("PlanetarySystemDescriptor")
        .map(build_body_name_map)
        .unwrap_or_default();
    let mut scenario_starts: Vec<ScenarioStart> = raw
        .get("StartGameData")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    parse_scenario_start_with_body_names(e, &scenario_routing, &body_name_map)
                })
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

    // Per-body mining license fees. The mod walks live `ObjectInfo`
    // MonoBehaviours, so this key is only present in dumps from a built
    // BepInEx mod (no top-level `ObjectInfo` key → empty vector, never an
    // error). Sort by body_name for stable downstream rendering.
    let mut license_fees: Vec<BodyLicenseFee> = raw
        .get("ObjectInfo")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_body_license_fee).collect())
        .unwrap_or_default();
    license_fees.sort_by(|a, b| a.body_name.cmp(&b.body_name));

    let mut asteroid_classes: Vec<AsteroidClass> = raw
        .get("ObjectSubType")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_asteroid_class).collect())
        .unwrap_or_default();
    asteroid_classes.sort_by(|a, b| a.name.cmp(&b.name));

    let mut exoplanet_systems: Vec<ExoplanetSystem> = raw
        .get("PlanetarySystemDescriptor")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_exoplanet_system).collect())
        .unwrap_or_default();
    exoplanet_systems.sort_by(|a, b| {
        a.star_type.cmp(&b.star_type).then(a.name.cmp(&b.name))
    });

    let achievements = collect_achievements(&raw);

    let out = Sirenix {
        spacecraft,
        launch_vehicles,
        research,
        facilities,
        space_components,
        crew_transports,
        resources,
        contracts,
        scenario_starts,
        epochs,
        license_fees,
        asteroid_classes,
        exoplanet_systems,
        achievements,
    };
    serde_json::to_writer_pretty(fs::File::create(&output)?, &out)?;
    eprintln!(
        "wrote {} ({} spacecraft, {} LVs, {} research, {} facilities, {} components, {} crew transports, {} resources, {} contracts, {} scenarios, {} epochs, {} bodies w/ license fees, {} asteroid classes, {} exoplanet systems, {} achievements)",
        output.display(),
        out.spacecraft.len(),
        out.launch_vehicles.len(),
        out.research.len(),
        out.facilities.len(),
        out.space_components.len(),
        out.crew_transports.len(),
        out.resources.len(),
        out.contracts.len(),
        out.scenario_starts.len(),
        out.epochs.len(),
        out.license_fees.len(),
        out.asteroid_classes.len(),
        out.exoplanet_systems.len(),
        out.achievements.len(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_crew_compartment_module() {
        let v = serde_json::json!({
            "$name": "module_crew_compartment",
            "id": "module_crew_compartment",
            "specialAbilityFacilityNew": "CrewTransport",
            "specialAbilityParameter": 5,
            "mass": 5,
            "isLocked": false,
        });
        let ct = parse_crew_transport(&v).expect("should parse");
        assert_eq!(ct.id, "module_crew_compartment");
        assert_eq!(ct.capacity, 5);
        assert_eq!(ct.mass, 5.0);
        assert!(!ct.is_locked);
    }

    #[test]
    fn rejects_non_crew_transport_modules() {
        let v = serde_json::json!({
            "id": "module_habitat",
            "specialAbilityFacilityNew": "CrewCapacity",
            "specialAbilityParameter": 100,
            "mass": 20,
        });
        assert!(parse_crew_transport(&v).is_none());
    }

    #[test]
    fn parses_locked_crew_large() {
        let v = serde_json::json!({
            "id": "module_crew_large",
            "specialAbilityFacilityNew": "CrewTransport",
            "specialAbilityParameter": 100,
            "mass": 60,
            "isLocked": true,
        });
        let ct = parse_crew_transport(&v).expect("should parse");
        assert_eq!(ct.capacity, 100);
        assert_eq!(ct.mass, 60.0);
        assert!(ct.is_locked);
    }

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
    fn parses_gravity_gate_from_can_build_parameter() {
        // Real shape from the Sirenix dump for id_Rocket_RocketType5 / RocketType6
        // (Al-Ice rockets): a single Gravity 0..1.8 gate restricting launch to
        // bodies with surface gravity at or below 1.8 g.
        let v = serde_json::json!({
            "id": "id_Rocket_RocketType5",
            "canBuildParameter": {
                "terraformParameterCanBuild": {
                    "list": [
                        {
                            "terraformParameterSettingsCanBuild": "Gravity",
                            "min": 0,
                            "max": 1.8
                        }
                    ]
                }
            }
        });
        let lv = parse_launch_vehicle(&v).expect("parses");
        let gate = lv.gravity_gate.expect("gravity_gate populated");
        assert_eq!(gate.min_g, 0.0);
        assert_eq!(gate.max_g, 1.8);
    }

    #[test]
    fn launch_vehicle_without_gravity_gate_has_none() {
        // Most LVs don't carry a canBuildParameter at all; gravity_gate stays None.
        let v = serde_json::json!({ "id": "lv_chem_seadragon" });
        let lv = parse_launch_vehicle(&v).expect("parses");
        assert!(lv.gravity_gate.is_none());
    }

    #[test]
    fn launch_vehicle_ignores_non_gravity_can_build_entries() {
        // If the list carries e.g. a Pressure gate (we don't surface those for
        // LVs), gravity_gate stays None.
        let v = serde_json::json!({
            "id": "lv_chem_seadragon",
            "canBuildParameter": {
                "terraformParameterCanBuild": {
                    "list": [
                        { "terraformParameterSettingsCanBuild": "Pressure", "min": 0.0001, "max": 2 }
                    ]
                }
            }
        });
        let lv = parse_launch_vehicle(&v).expect("parses");
        assert!(lv.gravity_gate.is_none());
    }

    #[test]
    fn launch_vehicle_intersects_multiple_gravity_gates() {
        // Defensive: if multiple Gravity entries appear, take the tightest
        // (intersection of all [min, max] ranges).
        let v = serde_json::json!({
            "id": "lv_synth",
            "canBuildParameter": {
                "terraformParameterCanBuild": {
                    "list": [
                        { "terraformParameterSettingsCanBuild": "Gravity", "min": 0.0, "max": 2.0 },
                        { "terraformParameterSettingsCanBuild": "Gravity", "min": 0.5, "max": 1.5 }
                    ]
                }
            }
        });
        let lv = parse_launch_vehicle(&v).expect("parses");
        let gate = lv.gravity_gate.expect("gravity_gate populated");
        assert_eq!(gate.min_g, 0.5);
        assert_eq!(gate.max_g, 1.5);
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

    #[test]
    fn parses_research_stage_for_era_tier() {
        // `stage` is the era tier (0 early / 1 mid / 2 late).  Most entries are 0;
        // mid- and late-game research carry non-zero values that map to the
        // research-tree progression we surface as an "Era" column.
        let v = serde_json::json!({
            "id": "research_fusionprop_4",
            "researchType": {"name": "Engineering"},
            "researchSubType": {"name": "SubBranch_Fusion"},
            "stage": 2,
            "unlockData": {
                "actionUnlock": "UnlockBonus",
                "parameter1": "III",
                "bonus": "ComponentExhaustV",
                "bonusParameter": 25,
                "id_ComponentOrOther": ["eng_fusion"]
            }
        });
        let r = parse_research(&v).expect("should parse");
        assert_eq!(r.stage, 2);

        // Missing stage defaults to 0 (early).
        let bare = serde_json::json!({
            "id": "research_chem_main1",
            "researchType": {"name": "Engineering"},
            "researchSubType": {"name": "SubBranch_Chemical"}
        });
        let r0 = parse_research(&bare).expect("should parse");
        assert_eq!(r0.stage, 0);
    }

    #[test]
    fn parses_research_secondary_unlocks_from_unlock_data_list() {
        // A single research entry can stack multiple secondary unlock actions in
        // `unlockDataList[]`: bonuses, facility unlocks, etc.  We must capture
        // every entry except `UnlockContract` (which still flows to
        // `contract_unlocks`), so the wiki Unlocks cell can render them.
        let v = serde_json::json!({
            "id": "research_fusionprop_4",
            "researchType": {"name": "Engineering"},
            "researchSubType": {"name": "SubBranch_Fusion"},
            "stage": 2,
            "unlockData": {
                "actionUnlock": "UnlockBonus",
                "parameter1": "III",
                "bonus": "ComponentExhaustV",
                "bonusParameter": 25,
                "id_ComponentOrOther": ["eng_fusion"]
            },
            "unlockDataList": [
                {
                    "actionUnlock": "UnlockBonus",
                    "parameter1": "III",
                    "bonus": "ComponentThrust",
                    "bonusParameter": 20,
                    "id_ComponentOrOther": ["eng_fusion"]
                },
                {
                    "actionUnlock": "UnlockBonus",
                    "parameter1": "",
                    "bonus": "PowerProduction",
                    "bonusParameter": 25,
                    "id_ComponentOrOther": ["build_power_fusion"]
                },
                {
                    "actionUnlock": "UnlockFacility",
                    "parameter1": "build_habitatdome",
                    "bonus": "None",
                    "bonusParameter": 0,
                    "id_ComponentOrOther": []
                },
                {
                    "actionUnlock": "UnlockContract",
                    "parameter1": "contract_foo",
                    "bonus": "None",
                    "bonusParameter": 0,
                    "id_ComponentOrOther": []
                }
            ]
        });
        let r = parse_research(&v).expect("should parse");
        // Contract entries continue to flow into `contract_unlocks` only — not
        // into the secondary-unlocks vector.
        assert_eq!(r.contract_unlocks, vec!["contract_foo"]);
        // Three non-contract entries: two bonuses + one facility unlock.
        assert_eq!(r.secondary_unlocks.len(), 3);

        assert_eq!(r.secondary_unlocks[0].action, "UnlockBonus");
        assert_eq!(r.secondary_unlocks[0].bonus, "ComponentThrust");
        assert_eq!(r.secondary_unlocks[0].bonus_parameter, 20.0);
        assert_eq!(r.secondary_unlocks[0].target, "");

        assert_eq!(r.secondary_unlocks[1].action, "UnlockBonus");
        assert_eq!(r.secondary_unlocks[1].bonus, "PowerProduction");
        assert_eq!(r.secondary_unlocks[1].bonus_parameter, 25.0);

        assert_eq!(r.secondary_unlocks[2].action, "UnlockFacility");
        assert_eq!(r.secondary_unlocks[2].target, "build_habitatdome");
        assert_eq!(r.secondary_unlocks[2].bonus, "");
        assert_eq!(r.secondary_unlocks[2].bonus_parameter, 0.0);
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
    fn facility_parses_habitat_pressure_constraint() {
        // Synthetic facility carrying one Pressure 0.0001..2 constraint —
        // mirrors the shape used by `canBuildParameter.terraformParameterCanBuild`
        // in the real dump.
        let v = serde_json::json!({
            "id": "build_pressure_synth",
            "facilityType": "LaunchFacility",
            "canBuildParameter": {
                "terraformParameterCanBuild": {
                    "list": [
                        {
                            "terraformParameterSettingsCanBuild": "Pressure",
                            "min": 0.0001,
                            "max": 2
                        }
                    ]
                }
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert_eq!(f.habitat_constraints.len(), 1);
        let c = &f.habitat_constraints[0];
        assert_eq!(c.parameter, "Pressure");
        assert_eq!(c.min, 0.0001);
        assert_eq!(c.max, 2.0);
    }

    #[test]
    fn facility_magrails_real_shape_parses_pressure_atmosphere() {
        // Real-data shape from the Sirenix dump for build_launch_magrails:
        // a single Pressure constraint with min=0.0001, max=2 (atmosphere only).
        let v = serde_json::json!({
            "id": "build_launch_magrails",
            "facilityType": "LaunchFacility",
            "possiblePlacement": "Surface",
            "canBuildParameter": {
                "terraformParameterCanBuild": {
                    "list": [
                        {
                            "terraformParameterSettingsCanBuild": "Pressure",
                            "min": 0.0001,
                            "max": 2
                        }
                    ]
                }
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert_eq!(
            f.habitat_constraints,
            vec![HabitatConstraint {
                parameter: "Pressure".to_string(),
                min: 0.0001,
                max: 2.0,
            }]
        );
    }

    #[test]
    fn facility_asteroid_only_gate_surfaces_as_object_type_constraint() {
        // build_asteroid_engine_facility: canBuildBy="ObjectTypes", objectTypes="Asteroid".
        // This is an object-kind gate, not a habitability range — we surface it
        // through the same habitat_constraints vec with a synthetic parameter
        // name so the renderer can pick a friendly label.
        let v = serde_json::json!({
            "id": "build_asteroid_engine_facility",
            "facilityType": "Other",
            "canBuildParameter": {
                "canBuildBy": "ObjectTypes",
                "objectTypes": "Asteroid",
                "terraformParameterCanBuild": { "list": [] }
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert_eq!(f.habitat_constraints.len(), 1);
        assert!(
            f.habitat_constraints[0].parameter.starts_with("ObjectType:"),
            "expected ObjectType:* synthetic parameter, got {:?}",
            f.habitat_constraints[0]
        );
        assert!(f.habitat_constraints[0].parameter.contains("Asteroid"));
    }

    #[test]
    fn facility_can_build_by_object_types_with_planet_kind_is_not_a_gate() {
        // The default `objectTypes:"Planet"` (with canBuildBy != "ObjectTypes")
        // is not a gate — most facilities have it. We must NOT emit a
        // constraint for that.
        let v = serde_json::json!({
            "id": "build_lab",
            "facilityType": "Other",
            "canBuildParameter": {
                "canBuildBy": "None",
                "objectTypes": "Planet",
                "terraformParameterCanBuild": { "list": [] }
            }
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert!(f.habitat_constraints.is_empty());
    }

    #[test]
    fn facility_without_can_build_parameter_has_empty_habitat_constraints() {
        // Most facilities have no canBuildParameter — habitat_constraints
        // must default to an empty Vec (serialized as []).
        let v = serde_json::json!({
            "id": "build_lab",
            "facilityType": "Other",
        });
        let f = parse_facility(&v, "Ground").expect("parses");
        assert!(f.habitat_constraints.is_empty());
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

    #[test]
    fn parses_earth_license_fees_from_object_info() {
        // The BepInEx mod emits each ObjectInfo MonoBehaviour as
        //   {"name": <body>, "resourceMiningLicenseFeePerT": {<resId>: <fee>, ...}}
        // Earth is the only body that currently charges fees; values come
        // from the runtime walk because the field is [OdinSerialize].
        let v = serde_json::json!({
            "name": "Earth",
            "resourceMiningLicenseFeePerT": {
                "id_resource_alloy": 30,
                "id_resource_fuel": 50,
                "id_resource_steel": 0
            }
        });
        let bf = parse_body_license_fee(&v).expect("Earth entry should parse");
        assert_eq!(bf.body_name, "Earth");
        // Zero-fee entries are dropped — they would render identically to
        // "no fee" rows and add noise to downstream lookups.
        assert_eq!(bf.fees_per_t.len(), 2);
        assert_eq!(bf.fees_per_t.get("alloy"), Some(&30.0));
        assert_eq!(bf.fees_per_t.get("fuel"), Some(&50.0));
        assert!(bf.fees_per_t.get("steel").is_none());
    }

    #[test]
    fn parses_body_with_empty_license_fee_map() {
        // Mars (and most other bodies) doesn't charge a license fee; the
        // dict is empty. We still emit the body so the renderer can show
        // "no fees" if it wants to.
        let v = serde_json::json!({
            "name": "Mars",
            "resourceMiningLicenseFeePerT": {}
        });
        let bf = parse_body_license_fee(&v).expect("Mars entry should parse");
        assert_eq!(bf.body_name, "Mars");
        assert!(bf.fees_per_t.is_empty());
    }

    #[test]
    fn license_fees_round_trip_through_serde() {
        // Round-trip check: the BodyLicenseFee struct must serialize into
        // the same shape gen_pages.rs deserializes from. Mirrors the
        // synthetic Earth + empty-Mars dump the spec calls out.
        let earth = BodyLicenseFee {
            body_name: "Earth".to_string(),
            fees_per_t: [("alloy".to_string(), 30.0), ("fuel".to_string(), 50.0)]
                .into_iter()
                .collect(),
        };
        let mars = BodyLicenseFee {
            body_name: "Mars".to_string(),
            fees_per_t: Default::default(),
        };
        let json = serde_json::to_string(&vec![&earth, &mars]).unwrap();
        let back: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(back[0]["body_name"], "Earth");
        assert_eq!(back[0]["fees_per_t"]["alloy"], 30.0);
        assert_eq!(back[0]["fees_per_t"]["fuel"], 50.0);
        assert_eq!(back[1]["body_name"], "Mars");
        assert!(back[1]["fees_per_t"].as_object().unwrap().is_empty());
    }

    #[test]
    fn parses_date_locked_contract() {
        // Date-locked contracts (e.g. Exoplanet Search) carry both
        // `isLocked: true` and a `dateTimeStringStart: "YYYY-MM-DD HH:MM:SS"`.
        // The renderer needs that string to derive the contract's display
        // Order (year extracted from the start date).
        let v = serde_json::json!({
            "id": "contract_general_exoplanetsearch",
            "isLocked": true,
            "isFinalContract": false,
            "dateTimeStringStart": "2080-01-01 00:00:00",
            "dateStartActive": "",
            "yearsToExpire": 0,
            "objectives": [],
            "rewards": []
        });
        let c = parse_contract(&v).expect("contract should parse");
        assert!(c.is_locked);
        assert_eq!(
            c.date_time_string_start.as_deref(),
            Some("2080-01-01 00:00:00")
        );
    }

    fn carbon_asteroid_fixture() -> Value {
        serde_json::json!({
            "id": "ObjectSubType.AsteroidCarbon",
            "miningFactors": [
                [
                    {
                        "Category": "High",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_volatile" }
                    }
                ],
                [],
                [
                    { "Category": "Low", "probability": 0.45, "ResourceDefinition": { "name": "id_resource_metal" } },
                    { "Category": "Low", "probability": 0.45, "ResourceDefinition": { "name": "id_resource_water" } },
                    { "Category": "Low", "probability": 0.10, "ResourceDefinition": { "name": "id_resource_silicon" } }
                ]
            ]
        })
    }

    fn trappist_fixture() -> Value {
        serde_json::json!({
            "$name": "PlanetarySystem_Trappist",
            "$type": "PlanetarySystemDescriptor",
            "id": "PlanetarySystem_Trappist",
            "solarSystemData": {
                "star1": { "$ref": true, "type": "StarTypeDefinition", "name": "startype_M8" },
                "star2": null,
                "planetAmount": 2,
                "systemAge": "Mature",
                "tabObjectInfoData": [
                    {
                        "addNBodyData": {
                            "name": "TRAPPIST-1b",
                            "a": 0.0115, "e": 0.02, "p": 0,
                            "omega_uc": 10, "omega_lc": 249,
                            "inclication": 1,
                            "massIn1Pow24": 8.18164, "radiusKM": 7390.36
                        },
                        "planetType": { "$ref": true, "type": "GeneratedPlanetType", "name": "planet_rocky_volcanic" }
                    },
                    {
                        "addNBodyData": {
                            "name": "TRAPPIST-1c",
                            "a": 0.0158, "e": 0.01,
                            "inclication": 0.85,
                            "massIn1Pow24": 7.811376, "radiusKM": 6988.987
                        },
                        "planetType": { "name": "planet_rocky_barren" }
                    }
                ]
            }
        })
    }

    /// Fixture mirroring the real dump's `ObjectInfoSaves[]` region inside a
    /// StartGameData's `serializationData.SerializationNodes` stream. Each
    /// `ObjectInfoSave` carries an `IDObjectInfo.id` (Integer) and a
    /// `habitabilityParameters` sub-node with the body's start-of-scenario
    /// environmental state.
    ///
    /// This fixture builds two bodies: id=66 (Earth-like values) and id=59
    /// (Mars-like values), so the test can assert both shape and ordering.
    fn body_habitability_fixture() -> Value {
        let node = |name: &str, entry: &str, data: &str| {
            serde_json::json!({"Name": name, "Entry": entry, "Data": data})
        };
        // Helper for the habitabilityParameters sub-node — the field order
        // mirrors what the real dump emits (TerraformationConfig+HabitabilityParametersNew).
        let habit = |temp: &str,
                     pressure: &str,
                     gravity: &str,
                     water: &str,
                     radiation: &str,
                     albedo: &str|
         -> Vec<Value> {
            vec![
                node(
                    "habitabilityParameters",
                    "StartOfNode",
                    "X|Data.ScriptableObject.Terraformation.TerraformationConfig+HabitabilityParametersNew, Assembly-CSharp",
                ),
                node("temperature", "FloatingPoint", temp),
                node("composition", "FloatingPoint", "0"),
                node("pressure", "FloatingPoint", pressure),
                node("gravity", "FloatingPoint", gravity),
                node("water", "FloatingPoint", water),
                node("radiation", "FloatingPoint", radiation),
                node("magneticFieldVisualization", "FloatingPoint", "0"),
                node("albedo", "FloatingPoint", albedo),
                node("internalFlux", "FloatingPoint", "0"),
                node("heatCapacityRock", "FloatingPoint", "0"),
                // The optional `temperatureWithAtmosphereOld` Nullable double
                // appears as a wrapper node; include the empty form for
                // realism.
                node(
                    "temperatureWithAtmosphereOld",
                    "StartOfNode",
                    "System.Nullable`1[[System.Double, mscorlib]], mscorlib",
                ),
                node("", "Null", ""),
                node("", "EndOfNode", ""),
                node("saturationPressureForBoilingOld", "Null", ""),
                node("totalHeatCapacity", "FloatingPoint", "0"),
                node("prevTotalHeatCapacity", "FloatingPoint", "0"),
                node("temperatureSwings", "FloatingPoint", "0"),
                node("mirrorsStrength", "FloatingPoint", "0"),
                node("shadesStrength", "FloatingPoint", "0"),
                node("extremeVolcanism", "FloatingPoint", "0"),
                node("environmentalToxicity", "FloatingPoint", "0"),
                node("cryoVolcanism", "FloatingPoint", "0"),
                node("hydroCarbonLakes", "FloatingPoint", "0"),
                node("", "EndOfNode", ""),
            ]
        };
        let mut nodes: Vec<Value> = vec![
            // Minimal companyDataSave so the outer parse_scenario_start
            // doesn't bail before reaching ObjectInfoSaves.
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
            // ObjectInfoSaves: two bodies.
            node(
                "ObjectInfoSaves",
                "StartOfNode",
                "3|List`1[Manager.SaveGameData+ObjectInfoSave]",
            ),
            node("", "StartOfArray", "2"),
            // Body #1 — id=66 (Earth-like)
            node("", "StartOfNode", "4|Manager.SaveGameData+ObjectInfoSave"),
            node("IDObjectInfo", "StartOfNode", "5|Game.Info.IdForObjectInfo"),
            node("id", "Integer", "66"),
            node("", "EndOfNode", ""),
            node("isInGameDestroy", "Boolean", "false"),
            node("idImpactFrom", "Integer", "-1"),
            node("customName", "Null", ""),
        ];
        // Earth-like habitability (15 °C, 1 atm, 1 g, 0.71 water, 0 radiation, 0.3 albedo)
        nodes.extend(habit("15", "1", "1", "0.71", "0", "0.3"));
        nodes.push(node("", "EndOfNode", "")); // close ObjectInfoSave #1
        // Body #2 — id=59 (Mars-like)
        nodes.push(node("", "StartOfNode", "6|Manager.SaveGameData+ObjectInfoSave"));
        nodes.push(node("IDObjectInfo", "StartOfNode", "7|Game.Info.IdForObjectInfo"));
        nodes.push(node("id", "Integer", "59"));
        nodes.push(node("", "EndOfNode", ""));
        nodes.push(node("isInGameDestroy", "Boolean", "false"));
        nodes.push(node("idImpactFrom", "Integer", "-1"));
        nodes.push(node("customName", "Null", ""));
        // Mars-like: -63 °C, 0.006 atm, 0.38 g, 0 water, 1.2 radiation, 0.25 albedo
        nodes.extend(habit("-63", "0.006", "0.38", "0", "1.2", "0.25"));
        nodes.push(node("", "EndOfNode", "")); // close ObjectInfoSave #2
        nodes.push(node("", "EndOfArray", ""));
        nodes.push(node("", "EndOfNode", "")); // close ObjectInfoSaves
        serde_json::json!({
            "$name": "StartGameColonization",
            "id": "StartGameColonization",
            "serializationData": {
                "SerializationNodes": nodes,
            }
        })
    }

    #[test]
    fn parses_carbon_asteroid_class() {
        let class = parse_asteroid_class(&carbon_asteroid_fixture())
            .expect("AsteroidCarbon should parse");
        assert_eq!(class.name, "Carbon");
        // Two non-empty tiers: High and Low. The empty Mid bucket is dropped.
        assert_eq!(class.tiers.len(), 2);
        let high = class.tiers.iter().find(|t| t.category == "High").expect("High tier");
        assert_eq!(high.rolls.len(), 1);
        assert_eq!(high.rolls[0].resource_id, "volatile");
        assert!((high.rolls[0].probability - 1.0).abs() < 1e-9);
        let low = class.tiers.iter().find(|t| t.category == "Low").expect("Low tier");
        assert_eq!(low.rolls.len(), 3);
        let metal = low.rolls.iter().find(|r| r.resource_id == "metal").expect("metal roll");
        assert!((metal.probability - 0.45).abs() < 1e-9);
        let silicon = low.rolls.iter().find(|r| r.resource_id == "silicon").expect("silicon roll");
        assert!((silicon.probability - 0.10).abs() < 1e-9);
    }

    #[test]
    fn parses_metal_asteroid_with_empty_first_bucket() {
        // AsteroidMetal has an empty bucket[0]; its High tier lives in
        // bucket[1].  The parser must NOT key tier categories by bucket
        // index — it must use the `Category` field on each roll.
        let v = serde_json::json!({
            "id": "ObjectSubType.AsteroidMetal",
            "miningFactors": [
                [],
                [
                    {
                        "Category": "High",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_raremetal" }
                    }
                ],
                [
                    {
                        "Category": "Low",
                        "probability": 0.65,
                        "ResourceDefinition": { "name": "id_resource_metal" }
                    },
                    {
                        "Category": "Low",
                        "probability": 0.30,
                        "ResourceDefinition": { "name": "id_resource_uran" }
                    },
                    {
                        "Category": "Low",
                        "probability": 0.05,
                        "ResourceDefinition": { "name": "id_resource_water" }
                    }
                ]
            ]
        });
        let class = parse_asteroid_class(&v).expect("AsteroidMetal should parse");
        assert_eq!(class.name, "Metal");
        assert_eq!(class.tiers.len(), 2);
        let high = class.tiers.iter().find(|t| t.category == "High").expect("High tier");
        assert_eq!(high.rolls.len(), 1);
        assert_eq!(high.rolls[0].resource_id, "raremetal");
    }

    #[test]
    fn parses_helium3_asteroid_lowercases_resource_id() {
        // The raw dump emits id_resource_HEL3 in mixed case; the same id
        // appears lowercased everywhere else (resources page, refiner I/O).
        // `parse_asteroid_class` must run the resource id through the same
        // normalize_resource_id() used by parse_facility so cross-page joins
        // line up.
        let v = serde_json::json!({
            "id": "ObjectSubType.AsteroidHelium3",
            "miningFactors": [
                [],
                [],
                [
                    {
                        "Category": "Low",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_HEL3" }
                    }
                ]
            ]
        });
        let class = parse_asteroid_class(&v).expect("AsteroidHelium3 should parse");
        assert_eq!(class.name, "Helium3");
        let low = class.tiers.iter().find(|t| t.category == "Low").expect("Low tier");
        assert_eq!(low.rolls[0].resource_id, "hel3");
    }

    #[test]
    fn rejects_non_asteroid_object_subtypes() {
        // Empty-id entries and non-Asteroid prefixes must be skipped.
        let blank = serde_json::json!({"id": "", "miningFactors": []});
        assert!(parse_asteroid_class(&blank).is_none());
        let other = serde_json::json!({"id": "ObjectSubType.Comet", "miningFactors": []});
        assert!(parse_asteroid_class(&other).is_none());
    }

    #[test]
    fn tier_order_is_high_mid_low() {
        // The renderer wants a consistent High → Mid → Low order regardless
        // of bucket order in the dump.
        let v = serde_json::json!({
            "id": "ObjectSubType.AsteroidStone",
            "miningFactors": [
                [
                    {
                        "Category": "Low",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_metal" }
                    }
                ],
                [
                    {
                        "Category": "Mid",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_silicon" }
                    }
                ],
                [
                    {
                        "Category": "High",
                        "probability": 1.0,
                        "ResourceDefinition": { "name": "id_resource_water" }
                    }
                ]
            ]
        });
        let class = parse_asteroid_class(&v).expect("parses");
        let cats: Vec<&str> = class.tiers.iter().map(|t| t.category.as_str()).collect();
        assert_eq!(cats, vec!["High", "Mid", "Low"]);
    }

    #[test]
    fn parses_trappist_system_into_exoplanet_system() {
        let sys = parse_exoplanet_system(&trappist_fixture())
            .expect("Trappist fixture should parse");
        assert_eq!(sys.name, "Trappist-1");
        assert_eq!(sys.id, "PlanetarySystem_Trappist");
        assert_eq!(sys.star_type, "M8");
        assert!(sys.second_star_type.is_none());
        assert_eq!(sys.system_age, "Mature");
        assert_eq!(sys.bodies.len(), 2);
    }

    #[test]
    fn parses_trappist_body_with_orbital_data() {
        let sys = parse_exoplanet_system(&trappist_fixture()).expect("parses");
        let b = &sys.bodies[0];
        assert_eq!(b.name, "TRAPPIST-1b");
        assert_eq!(b.planet_type, "planet_rocky_volcanic");
        assert!((b.semi_major_axis_au - 0.0115).abs() < 1e-9);
        assert!((b.eccentricity - 0.02).abs() < 1e-9);
        // Misspelled `inclication` field carries the inclination value.
        assert!((b.inclination_deg - 1.0).abs() < 1e-9);
        assert!((b.mass_1e24_kg - 8.18164).abs() < 1e-6);
        assert!((b.radius_km - 7390.36).abs() < 1e-3);
    }

    #[test]
    fn rejects_non_exoplanet_planetary_system() {
        // The Sol scenario, Simplified, Dummy, JSON, and the empty
        // PerfectSystem stub are not exoplanet systems. The filter must
        // return None for any descriptor whose $name isn't one of the
        // four shipped exoplanet systems.
        let v = serde_json::json!({
            "$name": "PlanetarySystem_Realistic",
            "solarSystemData": {
                "star1": { "name": "startype_G2" },
                "star2": null,
                "systemAge": "Mature",
                "tabObjectInfoData": []
            }
        });
        assert!(parse_exoplanet_system(&v).is_none());

        // PerfectSystem ships with a null star and zero bodies — its
        // PlanetarySystemDescriptor exists but isn't a real destination.
        let perfect = serde_json::json!({
            "$name": "PlanetarySystem_PerfectSystem",
            "solarSystemData": {
                "star1": null,
                "star2": null,
                "systemAge": "Young",
                "tabObjectInfoData": []
            }
        });
        assert!(parse_exoplanet_system(&perfect).is_none());
    }

    #[test]
    fn star_class_strips_startype_prefix() {
        assert_eq!(star_class_from_ref("startype_M8"), "M8");
        assert_eq!(star_class_from_ref("startype_G2"), "G2");
        // Passes through anything that doesn't carry the prefix.
        assert_eq!(star_class_from_ref("F9"), "F9");
    }

    #[test]
    fn parses_resource_terraformation_info_for_water() {
        // Water's real values from the Sirenix dump. Confirms field-name
        // mapping and that the thermal constants survive parsing.
        let v = serde_json::json!({
            "id": "id_resource_water",
            "resourceType": "Normal",
            "showOnUI": true,
            "canBeLeftOnObject": true,
            "marketClearingPriceBase": 0.0,
            "terraformationInfo": {
                "resourceOpticalDepthParameter": 0.002,
                "resourceHeatCapacity": 1860,
                "vaporizationLatentHeat": 50000,
                "baseTemperatureBoiling": 373,
                "temperatureMelting": 220,
                "pressureTriplePoint": 0.00611
            }
        });
        let r = parse_resource(&v).expect("parses");
        let ti = r.terraformation_info.expect("terraformation info present");
        assert_eq!(ti.optical_depth_parameter, 0.002);
        assert_eq!(ti.heat_capacity, 1860.0);
        assert_eq!(ti.vaporization_latent_heat, 50000.0);
        assert_eq!(ti.boiling_temperature_k, 373.0);
        assert_eq!(ti.melting_temperature_k, 220.0);
        assert_eq!(ti.pressure_triple_point, 0.00611);
    }

    #[test]
    fn parse_resource_drops_placeholder_terraformation_info() {
        // The TerraformationInfoDef default in the C# code initializes every
        // field to 1.0. Resources that never override it (antimatter, energy,
        // human/colonists, supplies, …) should land with `None` so the
        // terraforming page can skip them rather than rendering a row of 1's.
        let v = serde_json::json!({
            "id": "id_resource_antimatter",
            "resourceType": "Normal",
            "showOnUI": true,
            "terraformationInfo": {
                "resourceOpticalDepthParameter": 1.0,
                "resourceHeatCapacity": 1.0,
                "vaporizationLatentHeat": 1.0,
                "baseTemperatureBoiling": 1.0,
                "temperatureMelting": 1.0,
                "pressureTriplePoint": 1.0
            }
        });
        let r = parse_resource(&v).expect("parses");
        assert!(
            r.terraformation_info.is_none(),
            "all-1.0 placeholder should be dropped, got {:?}",
            r.terraformation_info
        );
    }

    #[test]
    fn parse_resource_handles_missing_terraformation_info() {
        // Old / hand-rolled fixtures may omit the field entirely; the parser
        // must not panic and must leave the field as None.
        let v = serde_json::json!({
            "id": "id_resource_water",
            "resourceType": "Normal",
            "showOnUI": true,
        });
        let r = parse_resource(&v).expect("parses");
        assert!(r.terraformation_info.is_none());
    }
    // ---- (branch additions follow) ----
    // ---------- Steam achievements ----------
    //
    // Three sources in the dump:
    //   * ContractDefinition[].steamAchievements[]   (array of binding objects)
    //   * SpacecraftType[].steamAchievement          (single binding object)
    //   * LaunchVehicleType[].steamAchievement       (single binding object)
    //
    // A "binding object" has shape:
    //   { conditions: [...], conditionsType: "All", achievement: { ..., name: "id_achievement_X" } | null }
    //
    // The inner `achievement` is null on entries that exist only as placeholders
    // (Cheat / Fake / etc.) — we MUST skip those, otherwise we'd emit empty rows.

    #[test]
    fn parse_contract_achievements_extracts_each_binding() {
        let v = serde_json::json!({
            "id": "contract_asteroid_impact",
            "steamAchievements": [
                {
                    "conditions": [],
                    "conditionsType": "All",
                    "achievement": {
                        "$ref": true,
                        "type": "SteamAchievement",
                        "name": "id_achievement_NotToday"
                    }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        assert_eq!(achs.len(), 1);
        assert_eq!(achs[0].id, "id_achievement_NotToday");
        assert_eq!(achs[0].source_type, "contract");
        assert_eq!(achs[0].source_id, "contract_asteroid_impact");
    }

    #[test]
    fn parse_contract_achievements_handles_multiple_bindings_per_contract() {
        // Real example: contract_general_interstellar2 binds two achievements
        // (ToInfinity unconditionally, Wanderlust if completed before 2400).
        let v = serde_json::json!({
            "id": "contract_general_interstellar2",
            "steamAchievements": [
                {
                    "conditions": [],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_ToInfinity" }
                },
                {
                    "conditions": [{"minTime": {"_DateTime": "0001-01-01T00:00:00.0000000"}, "maxTime": {"_DateTime": "2400-01-01T00:00:00.0010000"}, "negate": false}],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_Wanderlust" }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        let ids: Vec<&str> = achs.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["id_achievement_ToInfinity", "id_achievement_Wanderlust"]);
        assert!(achs.iter().all(|a| a.source_id == "contract_general_interstellar2"));
    }

    #[test]
    fn parse_contract_achievements_skips_null_bindings() {
        // Some contracts have a binding entry whose inner `achievement` is null —
        // these are placeholders and must not produce a row.
        let v = serde_json::json!({
            "id": "contract_placeholder",
            "steamAchievements": [
                { "conditions": [], "conditionsType": "All", "achievement": null }
            ]
        });
        assert!(parse_contract_achievements(&v).is_empty());
    }

    #[test]
    fn parse_contract_achievements_empty_when_no_field() {
        let v = serde_json::json!({ "id": "contract_no_field" });
        assert!(parse_contract_achievements(&v).is_empty());
    }

    #[test]
    fn parse_spacecraft_achievement_extracts_id_and_source() {
        // SpacecraftType[].steamAchievement is a *single* binding object,
        // not an array.
        let v = serde_json::json!({
            "id": "spacecraft_fusion_large",
            "steamAchievement": {
                "conditions": [],
                "conditionsType": "All",
                "achievement": { "name": "id_achievement_ThePowerofaStar" }
            }
        });
        let a = parse_spacecraft_achievement(&v).expect("should parse");
        assert_eq!(a.id, "id_achievement_ThePowerofaStar");
        assert_eq!(a.source_type, "spacecraft");
        assert_eq!(a.source_id, "spacecraft_fusion_large");
    }

    #[test]
    fn parse_spacecraft_achievement_skips_null_inner() {
        let v = serde_json::json!({
            "id": "spacecraft_chem_mid",
            "steamAchievement": {
                "conditions": [],
                "conditionsType": "All",
                "achievement": null
            }
        });
        assert!(parse_spacecraft_achievement(&v).is_none());
    }

    #[test]
    fn parse_spacecraft_achievement_skips_missing_field() {
        let v = serde_json::json!({ "id": "spacecraft_no_achievement" });
        assert!(parse_spacecraft_achievement(&v).is_none());
    }

    #[test]
    fn parse_launch_vehicle_achievement_extracts_id_and_source() {
        // No real-data examples exist (every LV in the dump has a null inner),
        // but the field shape mirrors SpacecraftType — make sure we'd surface
        // it correctly if the game added one.
        let v = serde_json::json!({
            "id": "lv_chem_seadragon",
            "steamAchievement": {
                "conditions": [],
                "conditionsType": "All",
                "achievement": { "name": "id_achievement_HeavyLifter" }
            }
        });
        let a = parse_launch_vehicle_achievement(&v).expect("should parse");
        assert_eq!(a.id, "id_achievement_HeavyLifter");
        assert_eq!(a.source_type, "launch_vehicle");
        assert_eq!(a.source_id, "lv_chem_seadragon");
    }

    #[test]
    fn parse_launch_vehicle_achievement_skips_null_inner() {
        let v = serde_json::json!({
            "id": "lv_nuke_large",
            "steamAchievement": {
                "conditions": [],
                "conditionsType": "All",
                "achievement": null
            }
        });
        assert!(parse_launch_vehicle_achievement(&v).is_none());
    }

    #[test]
    fn collect_achievements_dedupes_exact_source_pairs() {
        // If somehow the same (source_type, source_id, achievement_id) appears
        // twice (defensive — the dump shouldn't, but multiple bindings per
        // contract could otherwise collide), keep only one row.
        let raw = serde_json::json!({
            "ContractDefinition": [
                {
                    "id": "contract_a",
                    "steamAchievements": [
                        { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_X" } },
                        { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_X" } }
                    ]
                }
            ],
            "SpacecraftType": [],
            "LaunchVehicleType": []
        });
        let achs = collect_achievements(&raw);
        assert_eq!(achs.len(), 1);
        assert_eq!(achs[0].id, "id_achievement_X");
    }

    #[test]
    fn collect_achievements_keeps_distinct_sources_for_same_achievement() {
        // Real example: id_achievement_ThePowerofaStar is bound by multiple
        // spacecraft (fusion_small/mid/large + asteroid_puller).  Each is a
        // distinct way to earn it and should produce its own row.
        let raw = serde_json::json!({
            "ContractDefinition": [],
            "SpacecraftType": [
                { "id": "spacecraft_fusion_large", "steamAchievement": { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_ThePowerofaStar" } } },
                { "id": "spacecraft_fusion_mid", "steamAchievement": { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_ThePowerofaStar" } } }
            ],
            "LaunchVehicleType": []
        });
        let achs = collect_achievements(&raw);
        assert_eq!(achs.len(), 2);
        assert!(achs.iter().all(|a| a.id == "id_achievement_ThePowerofaStar"));
        let sources: Vec<&str> = achs.iter().map(|a| a.source_id.as_str()).collect();
        assert!(sources.contains(&"spacecraft_fusion_large"));
        assert!(sources.contains(&"spacecraft_fusion_mid"));
    }

    #[test]
    fn collect_achievements_orders_by_source_type_then_id() {
        // Deterministic output for stable downstream rendering: by source_type
        // (contract → spacecraft → launch_vehicle), then by source_id ascending,
        // then by achievement id.
        let raw = serde_json::json!({
            "ContractDefinition": [
                { "id": "contract_z", "steamAchievements": [ { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_Z" } } ] },
                { "id": "contract_a", "steamAchievements": [ { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_A" } } ] }
            ],
            "SpacecraftType": [
                { "id": "spacecraft_b", "steamAchievement": { "conditions": [], "conditionsType": "All", "achievement": { "name": "id_achievement_B" } } }
            ],
            "LaunchVehicleType": []
        });
        let achs = collect_achievements(&raw);
        let order: Vec<(&str, &str)> = achs
            .iter()
            .map(|a| (a.source_type.as_str(), a.source_id.as_str()))
            .collect();
        assert_eq!(
            order,
            vec![
                ("contract", "contract_a"),
                ("contract", "contract_z"),
                ("spacecraft", "spacecraft_b"),
            ]
        );
    }

    // ---------- Achievement conditions ----------
    //
    // The `conditions` array on a steamAchievement binding holds the *actual*
    // achievement requirements beyond "complete the parent contract":
    //   * deadline conditions carry a `maxTime` (._DateTime) — earn the
    //     achievement before this in-game date.  The year is what players
    //     care about.
    //   * dependency conditions carry a `contract` ref to a *different*
    //     ContractDefinition that must also have been completed.
    //
    // Conditions on placeholder achievements (inner achievement == null) are
    // ignored entirely (no row is emitted).

    #[test]
    fn parse_contract_achievements_captures_year_deadline_condition() {
        // contract_general_interstellar2's "Wanderlust" achievement requires
        // completing it before 2400-01-01.
        let v = serde_json::json!({
            "id": "contract_general_interstellar2",
            "steamAchievements": [
                {
                    "conditions": [
                        {
                            "minTime": { "_DateTime": "0001-01-01T00:00:00.0000000" },
                            "maxTime": { "_DateTime": "2400-01-01T00:00:00.0010000" },
                            "negate": false
                        }
                    ],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_Wanderlust" }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        assert_eq!(achs.len(), 1);
        assert_eq!(achs[0].conditions.len(), 1, "year deadline should be captured");
        assert_eq!(achs[0].conditions[0].before_year, 2400);
        assert!(achs[0].conditions[0].required_contract.is_empty());
    }

    #[test]
    fn parse_contract_achievements_captures_required_contract_condition() {
        // The "MarsTerraformed" achievement on contract_mars_terraform_atmo2
        // requires also having completed contract_mars_terraform_water.
        let v = serde_json::json!({
            "id": "contract_mars_terraform_atmo2",
            "steamAchievements": [
                {
                    "conditions": [
                        {
                            "contract": {
                                "$ref": true,
                                "type": "ContractDefinition",
                                "name": "contract_mars_terraform_water"
                            },
                            "negate": false
                        }
                    ],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_MarsTerraformed" }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        assert_eq!(achs.len(), 1);
        assert_eq!(achs[0].conditions.len(), 1);
        assert_eq!(
            achs[0].conditions[0].required_contract,
            "contract_mars_terraform_water"
        );
        assert_eq!(achs[0].conditions[0].before_year, 0);
    }

    #[test]
    fn parse_contract_achievements_captures_combined_conditions() {
        // The "Terraform" achievement carries both: requires terraform_water
        // AND must be earned before year 2600.  Both conditions surface.
        let v = serde_json::json!({
            "id": "contract_mars_terraform_atmo2",
            "steamAchievements": [
                {
                    "conditions": [
                        {
                            "contract": {
                                "$ref": true,
                                "type": "ContractDefinition",
                                "name": "contract_mars_terraform_water"
                            },
                            "negate": false
                        },
                        {
                            "minTime": { "_DateTime": "0001-01-01T00:00:00.0000000" },
                            "maxTime": { "_DateTime": "2600-01-01T00:00:00.0010000" },
                            "negate": false
                        }
                    ],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_Terraform" }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        assert_eq!(achs.len(), 1);
        assert_eq!(achs[0].conditions.len(), 2);
        // First condition: required contract; second: year deadline.
        let req: Vec<&str> = achs[0]
            .conditions
            .iter()
            .map(|c| c.required_contract.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(req, vec!["contract_mars_terraform_water"]);
        let yr: Vec<i32> = achs[0]
            .conditions
            .iter()
            .map(|c| c.before_year)
            .filter(|&y| y != 0)
            .collect();
        assert_eq!(yr, vec![2600]);
    }

    #[test]
    fn parse_contract_achievements_leaves_conditions_empty_when_array_is_empty() {
        let v = serde_json::json!({
            "id": "contract_x",
            "steamAchievements": [
                {
                    "conditions": [],
                    "conditionsType": "All",
                    "achievement": { "name": "id_achievement_X" }
                }
            ]
        });
        let achs = parse_contract_achievements(&v);
        assert_eq!(achs.len(), 1);
        assert!(achs[0].conditions.is_empty());
    }

    #[test]
    fn parses_body_habitability_for_two_bodies() {
        // The Colonization save has 150 ObjectInfoSaves; this fixture exercises
        // two of them to keep the assertions readable while covering the
        // multi-body case.
        let mut routing = std::collections::HashMap::new();
        routing.insert(
            "StartGameColonization".to_string(),
            "StartGameEpoch_Colonization".to_string(),
        );
        let s = parse_scenario_start(&body_habitability_fixture(), &routing)
            .expect("colonization should parse");
        // Sorted by body_id so the order is deterministic regardless of dump
        // ordering.
        assert_eq!(s.body_habitability.len(), 2);
        assert_eq!(s.body_habitability[0].body_id, 59);
        assert_eq!(s.body_habitability[1].body_id, 66);

        // Mars-like body (id=59)
        let mars = &s.body_habitability[0];
        assert_eq!(mars.temperature, -63.0);
        assert_eq!(mars.pressure, 0.006);
        assert_eq!(mars.gravity, 0.38);
        assert_eq!(mars.radiation, 1.2);
        assert_eq!(mars.water, 0.0);
        assert_eq!(mars.albedo, 0.25);

        // Earth-like body (id=66)
        let earth = &s.body_habitability[1];
        assert_eq!(earth.temperature, 15.0);
        assert_eq!(earth.pressure, 1.0);
        assert_eq!(earth.gravity, 1.0);
        assert_eq!(earth.water, 0.71);
        assert_eq!(earth.radiation, 0.0);
        assert_eq!(earth.albedo, 0.3);
    }

    #[test]
    fn body_habitability_with_resolver_uses_friendly_name() {
        // build_body_name_map is fed by the PlanetarySystemDescriptor's
        // tabObjectInfoData (id → idTranslation). The renderer hands the
        // resolved name to body_habitability via post-processing in the
        // parser.
        let mut routing = std::collections::HashMap::new();
        routing.insert(
            "StartGameColonization".to_string(),
            "StartGameEpoch_Colonization".to_string(),
        );
        let mut name_for: std::collections::HashMap<i32, String> =
            std::collections::HashMap::new();
        name_for.insert(66, "Earth".to_string());
        name_for.insert(59, "Mars".to_string());
        let s = parse_scenario_start_with_body_names(
            &body_habitability_fixture(),
            &routing,
            &name_for,
        )
        .expect("colonization should parse");
        let mars = s.body_habitability.iter().find(|b| b.body_id == 59).unwrap();
        assert_eq!(mars.body_name, "Mars");
        let earth = s.body_habitability.iter().find(|b| b.body_id == 66).unwrap();
        assert_eq!(earth.body_name, "Earth");
    }

    #[test]
    fn body_habitability_falls_back_to_numeric_id_when_name_missing() {
        // Asteroids and other small bodies aren't in tabObjectInfoData (the
        // Dummy descriptor only carries the 34 planets+moons). We still want
        // them in the output, but with a numeric-id body_name.
        let mut routing = std::collections::HashMap::new();
        routing.insert(
            "StartGameColonization".to_string(),
            "StartGameEpoch_Colonization".to_string(),
        );
        let name_for: std::collections::HashMap<i32, String> =
            std::collections::HashMap::new();
        let s = parse_scenario_start_with_body_names(
            &body_habitability_fixture(),
            &routing,
            &name_for,
        )
        .expect("colonization should parse");
        // With no resolver, body_name falls back to the numeric id as a string.
        assert_eq!(s.body_habitability[0].body_name, "59");
        assert_eq!(s.body_habitability[1].body_name, "66");
    }

    #[test]
    fn builds_body_name_map_from_planetary_system_descriptors() {
        // The PlanetarySystem_Dummy descriptor's tabObjectInfoData is the only
        // place in the dump that carries a complete planet/moon id→name
        // mapping. Each entry has `objectInfoId` (the int id) and
        // `idTranslation` (a "CelestialBodiesNames.Earth" key we strip down
        // to "Earth"). customName takes precedence if set.
        let descriptors = serde_json::json!([
            {
                "$name": "PlanetarySystem_Realistic",
                "solarSystemData": { "tabObjectInfoData": [] }
            },
            {
                "$name": "PlanetarySystem_Dummy",
                "solarSystemData": {
                    "tabObjectInfoData": [
                        { "objectInfoId": 66, "idTranslation": "CelestialBodiesNames.Earth", "customName": "" },
                        { "objectInfoId": 59, "idTranslation": "CelestialBodiesNames.Mars", "customName": "" },
                        // customName wins if present.
                        { "objectInfoId": 999, "idTranslation": "CelestialBodiesNames.Old", "customName": "Custom" },
                    ]
                }
            }
        ]);
        let map = build_body_name_map(&descriptors);
        assert_eq!(map.get(&66).map(|s| s.as_str()), Some("Earth"));
        assert_eq!(map.get(&59).map(|s| s.as_str()), Some("Mars"));
        assert_eq!(map.get(&999).map(|s| s.as_str()), Some("Custom"));
    }

    #[test]
    fn parses_dropped_habit_fields_default_to_zero() {
        // Older / minimal dumps may have a habitabilityParameters node with
        // only a subset of the fields populated. The struct's #[serde(default)]
        // already handles that on the JSON side; the parser must mirror it
        // by leaving the f64 fields at 0.0 when the corresponding
        // FloatingPoint node is absent.
        let node = |name: &str, entry: &str, data: &str| {
            serde_json::json!({"Name": name, "Entry": entry, "Data": data})
        };
        let nodes: Vec<Value> = vec![
            node("companyDataSave", "StartOfNode", "0|List`1[CompanyDataSave]"),
            node("", "StartOfArray", "0"),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
            node(
                "ObjectInfoSaves",
                "StartOfNode",
                "1|List`1[Manager.SaveGameData+ObjectInfoSave]",
            ),
            node("", "StartOfArray", "1"),
            node("", "StartOfNode", "2|Manager.SaveGameData+ObjectInfoSave"),
            node("IDObjectInfo", "StartOfNode", "3|Game.Info.IdForObjectInfo"),
            node("id", "Integer", "1234"),
            node("", "EndOfNode", ""),
            // Minimal habitabilityParameters: temperature only.
            node(
                "habitabilityParameters",
                "StartOfNode",
                "4|HabitabilityParametersNew",
            ),
            node("temperature", "FloatingPoint", "42"),
            node("", "EndOfNode", ""),
            node("", "EndOfNode", ""),
            node("", "EndOfArray", ""),
            node("", "EndOfNode", ""),
        ];
        let fixture = serde_json::json!({
            "$name": "StartGameColonization",
            "serializationData": {"SerializationNodes": nodes}
        });
        let mut routing = std::collections::HashMap::new();
        routing.insert(
            "StartGameColonization".to_string(),
            "StartGameEpoch_Colonization".to_string(),
        );
        let s = parse_scenario_start(&fixture, &routing).expect("parses");
        assert_eq!(s.body_habitability.len(), 1);
        let b = &s.body_habitability[0];
        assert_eq!(b.body_id, 1234);
        assert_eq!(b.temperature, 42.0);
        // All other fields should be zero (untouched defaults).
        assert_eq!(b.pressure, 0.0);
        assert_eq!(b.gravity, 0.0);
        assert_eq!(b.water, 0.0);
        assert_eq!(b.radiation, 0.0);
    }
}

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const AU_IN_KM: f64 = 149_597_870.7;

#[derive(Deserialize)]
struct Locale {
    celestial_bodies: Vec<CelestialBody>,
    spacecraft: Vec<NameDesc>,
    launch_vehicles: Vec<NameDesc>,
    research: Vec<ResearchEntry>,
    corporations: Vec<Corporation>,
    contracts: Vec<NameDesc>,
    resources: Vec<ResourceEntry>,
    #[allow(dead_code)]
    facilities: Vec<Facility>,
    #[allow(dead_code)]
    habitability_scales: BTreeMap<String, Vec<String>>,
    #[allow(dead_code)]
    cargo: Vec<NameDesc>,
}

#[derive(Deserialize)]
struct CelestialBody {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct NameDesc {
    #[allow(dead_code)]
    id: String,
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct ResearchEntry {
    #[allow(dead_code)]
    id: String,
    category: String,
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct Corporation {
    #[allow(dead_code)]
    id: String,
    name: String,
    description: String,
    traits: String,
}

#[derive(Deserialize)]
struct ResourceEntry {
    #[allow(dead_code)]
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct Facility {
    id: String,
    name: String,
    #[allow(dead_code)]
    description: String,
}

#[derive(Deserialize)]
struct Stats {
    bodies: Vec<Body>,
}

/// Per-celestial-body mining license fees, mirroring `parse_sirenix::BodyLicenseFee`.
/// Currently only Earth has non-empty fees; other bodies serialize with an
/// empty map. When the dump predates the BepInEx mod's `ObjectInfo` walk the
/// outer vec is empty and the resources page falls back to em-dashes.
#[derive(Deserialize, Clone, Default)]
struct BodyLicenseFeeStat {
    body_name: String,
    #[serde(default)]
    fees_per_t: BTreeMap<String, f64>,
}

#[derive(Deserialize, Default)]
struct Sirenix {
    spacecraft: Vec<SpacecraftStat>,
    #[serde(default)]
    launch_vehicles: Vec<LaunchVehicleStat>,
    #[serde(default)]
    research: Vec<ResearchStat>,
    #[serde(default)]
    facilities: Vec<FacilityStat>,
    #[serde(default)]
    space_components: Vec<SpaceComponentStat>,
    #[serde(default)]
    resources: Vec<ResourceStat>,
    #[serde(default)]
    contracts: Vec<ContractStat>,
    #[serde(default)]
    scenario_starts: Vec<ScenarioStartStat>,
    #[serde(default)]
    epochs: Vec<EpochStat>,
    /// Per-body mining license fees, sourced from the BepInEx mod's runtime
    /// walk of `ObjectInfo` MonoBehaviours. Empty for any dump produced
    /// before the mod was rebuilt with the ObjectInfo emitter; the
    /// resources page falls back to em-dashes in that case.
    #[serde(default)]
    license_fees: Vec<BodyLicenseFeeStat>,
    #[serde(default)]
    asteroid_classes: Vec<AsteroidClassStat>,
    #[serde(default)]
    exoplanet_systems: Vec<ExoplanetSystemStat>,
    #[serde(default)]
    achievements: Vec<AchievementStat>,
}

#[derive(Deserialize, Clone, Default)]
struct AsteroidClassStat {
    name: String,
    #[serde(default)]
    tiers: Vec<AsteroidTierStat>,
}

#[derive(Deserialize, Clone, Default)]
struct AsteroidTierStat {
    category: String,
    #[serde(default)]
    rolls: Vec<AsteroidRollStat>,
}

#[derive(Deserialize, Clone, Default)]
struct AsteroidRollStat {
    resource_id: String,
    probability: f64,
}

#[derive(Deserialize, Clone, Default)]
struct ExoplanetSystemStat {
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    id: String,
    star_type: String,
    #[serde(default)]
    second_star_type: Option<String>,
    system_age: String,
    #[serde(default)]
    bodies: Vec<ExoplanetBodyStat>,
}

#[derive(Deserialize, Clone, Default)]
struct ExoplanetBodyStat {
    name: String,
    planet_type: String,
    semi_major_axis_au: f64,
    eccentricity: f64,
    inclination_deg: f64,
    mass_1e24_kg: f64,
    radius_km: f64,
}

#[derive(Deserialize, Clone, Default)]
struct AchievementStat {
    id: String,
    #[serde(default)]
    name: String,
    source_type: String,
    source_id: String,
    #[serde(default)]
    description: String,
    /// Extra requirements parsed from the binding's `conditions[]` — year
    /// deadlines and required prior contracts.  Empty when the achievement
    /// has no constraints beyond completing the parent contract.
    #[serde(default)]
    conditions: Vec<AchievementConditionStat>,
}

#[derive(Deserialize, Clone, Default)]
struct AchievementConditionStat {
    /// Contract id that must also be completed; empty when the condition is
    /// purely a year deadline.
    #[serde(default)]
    required_contract: String,
    /// In-game year by which the achievement must be earned; 0 when no
    /// deadline applies.
    #[serde(default)]
    before_year: i32,
}

#[derive(Deserialize, Clone, Default)]
struct EpochStat {
    id: String,
    start_date_string: String,
    #[allow(dead_code)]
    is_locked: bool,
    #[serde(default)]
    possible_player_companies: Vec<String>,
}

#[derive(Deserialize, Clone, Default)]
struct ScenarioStartStat {
    scenario_id: String,
    #[serde(default)]
    corp_starts: Vec<CorpStartStat>,
    /// Per-body initial habitability snapshot at scenario load. One entry per
    /// `ObjectInfoSaves[]` element in the StartGameData save; sorted by
    /// `body_id` for deterministic order. Empty for the Early Exploration
    /// (testStartGAme) save whose dump doesn't carry a populated
    /// `habitabilityParameters` block.
    #[serde(default)]
    body_habitability: Vec<ScenarioBodyHabitabilityStat>,
}

/// Mirrors `parse_sirenix::ScenarioBodyHabitability`. See that struct's doc
/// comment for unit conventions (notably: temperature in °C, pressure in
/// Earth atmospheres, gravity in m/s²).
#[derive(Deserialize, Clone, Default)]
struct ScenarioBodyHabitabilityStat {
    body_name: String,
    #[allow(dead_code)]
    body_id: i32,
    #[serde(default)]
    temperature: f64,
    #[serde(default)]
    composition: f64,
    #[serde(default)]
    pressure: f64,
    #[serde(default)]
    gravity: f64,
    #[serde(default)]
    water: f64,
    #[serde(default)]
    radiation: f64,
    #[serde(default)]
    magnetic_field: f64,
    #[serde(default)]
    albedo: f64,
    #[allow(dead_code)]
    #[serde(default)]
    internal_flux: f64,
    #[allow(dead_code)]
    #[serde(default)]
    heat_capacity_rock: f64,
    #[allow(dead_code)]
    #[serde(default)]
    total_heat_capacity: f64,
    #[serde(default)]
    temperature_swings: f64,
    #[allow(dead_code)]
    #[serde(default)]
    mirrors_strength: f64,
    #[allow(dead_code)]
    #[serde(default)]
    shades_strength: f64,
    #[allow(dead_code)]
    #[serde(default)]
    extreme_volcanism: f64,
    #[allow(dead_code)]
    #[serde(default)]
    environmental_toxicity: f64,
    #[allow(dead_code)]
    #[serde(default)]
    cryo_volcanism: f64,
    #[allow(dead_code)]
    #[serde(default)]
    hydro_carbon_lakes: f64,
}

#[derive(Deserialize, Clone, Default)]
struct CorpStartStat {
    company_id: String,
    starting_money: f64,
    #[serde(default)]
    completed_research: Vec<String>,
    starting_launch_vehicles: i64,
    starting_spacecraft: i64,
    /// `(build_* id, count)` pairs of facilities the corp owns at scenario
    /// load.  Sourced from `objectInfoDatas[].productionItems[]` in the
    /// Sirenix dump; duplicates collapse to a single entry with their count.
    #[serde(default)]
    starting_facilities: Vec<(String, u32)>,
}

/// Mirror of `parse_sirenix::TerraformationInfo`. Field units are documented
/// on the source struct; the renderer treats every value verbatim and only
/// converts kelvin → celsius for display.
#[derive(Deserialize, Clone, Default, Debug)]
struct TerraformationInfoStat {
    optical_depth_parameter: f64,
    heat_capacity: f64,
    vaporization_latent_heat: f64,
    boiling_temperature_k: f64,
    melting_temperature_k: f64,
    pressure_triple_point: f64,
}

#[derive(Deserialize, Clone)]
struct ResourceStat {
    id: String,
    resource_type: String,
    market_price_base: f64,
    show_on_ui: bool,
    #[allow(dead_code)]
    can_be_left_on_object: bool,
    /// Thermal / phase constants surfaced on the terraforming page. `None`
    /// for resources whose `terraformationInfo` is the C# placeholder
    /// (energy, human, supplies, etc.) — parse_sirenix drops those.
    #[serde(default)]
    terraformation_info: Option<TerraformationInfoStat>,
}

#[derive(Deserialize, Clone)]
struct ContractStat {
    id: String,
    #[allow(dead_code)]
    is_locked: bool,
    is_final: bool,
    #[serde(default)]
    objectives: Vec<ContractObjectiveStat>,
    money_reward: f64,
    unlock_rewards: Vec<String>,
    facility_grants: Vec<String>,
    spacecraft_grants: Vec<String>,
    launch_vehicle_grants: Vec<String>,
    resource_grants: Vec<ResourceCost>,
    /// In-game date when the contract first becomes offerable. Format is
    /// `MM/DD/YYYY` from the dump; rendered as just `YYYY` on the table.
    #[serde(default)]
    date_start_active: Option<String>,
    /// Future-dated "this contract isn't even visible until year YYYY"
    /// timestamp from `dateTimeStringStart` in the dump (format
    /// `YYYY-MM-DD HH:MM:SS`).  Distinct from `date_start_active`.  Used
    /// together with `is_locked` to drive the Order column for date-locked
    /// contracts (e.g. Exoplanet Search → 2080 → Order 2080).
    #[serde(default)]
    date_time_string_start: Option<String>,
    /// Years the contract stays offerable before it disappears. `0` means
    /// "never expires" → rendered as "—".
    #[serde(default)]
    years_to_expire: f64,
    /// Distinct non-None `layer` values from this contract's objectives.  In
    /// production data the only non-None value seen is `"Asteroid"`.  Used to
    /// bump depth/Order in the contracts table so asteroid-layer contracts
    /// appear after the asteroid-belt gate even when their `unlock_rewards`
    /// chain has no upstream entries.
    #[serde(default)]
    objective_layers: Vec<String>,
    /// True iff at least one of this contract's objectives carries
    /// `layer: "None"`.  Used together with `objective_layers` to identify
    /// the bridge contracts (currently Humans on Mars + Space Dock) whose
    /// completion takes the player into the asteroid belt — see the
    /// gate-identification logic in `page_contracts`.
    #[serde(default)]
    has_layer_none_objective: bool,
}

#[derive(Deserialize, Clone)]
struct ContractObjectiveStat {
    kind: String,
    quantity: f64,
    target: Option<String>,
}

/// Mirror of `parse_sirenix::HabitatConstraint`. Each entry says "the body's
/// reading on `parameter` must lie within `[min, max]` for this facility to
/// be buildable here".
#[derive(Deserialize, Clone)]
struct HabitatConstraintStat {
    parameter: String,
    min: f64,
    max: f64,
}

#[derive(Deserialize, Clone)]
struct FacilityStat {
    id: String,
    descriptor: String,
    placement: String,
    facility_type: String,
    build_cost: Vec<ResourceCost>,
    maintenance_per_day: f64,
    workers_required: i64,
    energy_consumption: f64,
    research_prereq: Option<String>,
    #[allow(dead_code)]
    is_obsolete: bool,
    #[allow(dead_code)]
    can_be_scrapped: bool,
    #[allow(dead_code)]
    can_be_turned_off: bool,
    /// Days to construct. From `timeToBuildInDays`.
    #[serde(default)]
    build_time_days: f64,
    /// Launch-method bonus, surfaced only for the seven launch facilities.
    /// Shape: `(bonus_kind, magnitude)`.
    #[serde(default)]
    bonus_data: Option<(String, f64)>,
    /// `specialAbilityFacilityNew` enum (CrewCapacity, Refiner, Mining, Lab,
    /// EnergyProduction, etc.) — `None` when the facility has no role.
    #[serde(default)]
    role: Option<String>,
    /// `specialAbilityParameter` — role-dependent magnitude (crew count,
    /// research rate, mining rate, etc.).
    #[serde(default)]
    role_magnitude: f64,
    /// Terraforming deltas keyed by friendly label (Temperature, Atmosphere,
    /// Radiation, Magnetic field, …). Empty for non-terraforming facilities.
    #[serde(default)]
    habitability_deltas: Vec<(String, f64)>,
    /// Per-body build gates from `canBuildParameter.terraformParameterCanBuild`.
    /// Each entry pins a habitability parameter (Pressure / Temperature /
    /// Gravity / Radiation / Water / Composition / InternalFlux) to a
    /// `[min, max]` range the body must satisfy for construction.
    /// Empty for the vast majority of facilities.
    #[serde(default)]
    habitat_constraints: Vec<HabitatConstraintStat>,
    /// Resources this facility outputs per day. Pulled by `parse_sirenix` from
    /// structured `refinerData.output`, `energyProductionData`, `resourcesToMine`,
    /// and `byproducts` — never from description text. Drives the resources-page
    /// Producers column; using structured data avoids the false matches the
    /// substring heuristic produced (e.g., Exotic Alloy Production showing up as
    /// a producer of its inputs).
    #[serde(default)]
    produces: Vec<ResourceCost>,
    /// Resources this facility consumes per day. From structured `refinerData.input`
    /// (or `energyProductionData.input` for power facilities). Drives the
    /// resources-page Consumers column.
    #[serde(default)]
    consumes: Vec<ResourceCost>,
}

#[derive(Deserialize, Clone)]
struct SpaceComponentStat {
    id: String,
    category: String,
    thrust: f64,
    exhaust_v: f64,
    #[allow(dead_code)]
    mass: f64,
    #[allow(dead_code)]
    power: f64,
    #[allow(dead_code)]
    fuel_capacity: f64,
    #[allow(dead_code)]
    cargo_capacity: f64,
    #[allow(dead_code)]
    life_support_max: f64,
    fuel_type: Option<String>,
    #[allow(dead_code)]
    is_locked: bool,
}

#[derive(Deserialize, Clone)]
struct ResearchStat {
    id: String,
    work_hours: f64,
    branch: String,
    subbranch: String,
    prereqs: Vec<String>,
    action: String,
    unlock_target: Option<String>,
    bonus_kind: Option<String>,
    bonus_amount: f64,
    bonus_components: Vec<String>,
    show_in_tree: bool,
    #[serde(default)]
    contract_unlocks: Vec<String>,
    /// Era tier (0 early, 1 mid, 2 late) — drives the "Era" column.
    #[serde(default)]
    stage: u8,
    /// Stacked secondary unlocks from `unlockDataList[]` (excluding contracts).
    /// Surfaced as extra lines in the Unlocks cell.
    #[serde(default)]
    secondary_unlocks: Vec<SecondaryUnlockStat>,
}

#[derive(Deserialize, Clone, Default)]
struct SecondaryUnlockStat {
    action: String,
    #[serde(default)]
    target: String,
    #[serde(default)]
    bonus: String,
    #[serde(default)]
    bonus_parameter: f64,
}

#[derive(Deserialize, Clone)]
struct LaunchVehicleStat {
    id: String,
    max_payload: f64,
    #[allow(dead_code)]
    max_fuel_load: f64,
    #[allow(dead_code)]
    exhaust_velocity: f64,
    reusability: f64,
    can_send_human: bool,
    #[allow(dead_code)]
    is_locked: bool,
    build_cost: Vec<ResourceCost>,
    build_time_days: f64,
    launch_cost: f64,
    maintenance_cost_per_day: f64,
    #[serde(default)]
    fuel_type_on_start: Option<String>,
    /// Per-body gravity gate from `canBuildParameter.terraformParameterCanBuild`
    /// — the rocket can only launch from bodies whose surface gravity falls
    /// in `[min_g, max_g]` (units: g). `None` for every LV except Al-Ice in
    /// the shipped dump.
    #[serde(default)]
    gravity_gate: Option<GravityGateStat>,
}

/// Mirror of `parse_sirenix::GravityGate`. Bounds in g, Earth = 1 G.
#[derive(Deserialize, Clone)]
struct GravityGateStat {
    min_g: f64,
    max_g: f64,
}

#[derive(Deserialize, Clone)]
struct SpacecraftStat {
    id: String,
    #[allow(dead_code)]
    engine_module: Option<String>,
    engine_type: String,
    mass: f64,
    cargo_capacity: f64,
    fuel_capacity: f64,
    reusability: f64,
    /// True when the SpacecraftType has `needLaunchVehicleToGoToMoon=true`
    /// — that is, an LV is required from *any* planet/moon. False means the
    /// craft can self-launch from low-G bodies, but Earth still forces an LV
    /// (Earth is the hard-coded `Company.mainObjectInfo` in the gate).
    needs_launch_vehicle: bool,
    built_in_orbit: bool,
    #[allow(dead_code)]
    can_be_built_by_player: bool,
    build_cost: Vec<ResourceCost>,
    build_time_days: f64,
    launch_cost: f64,
}

#[derive(Deserialize, Clone)]
struct ResourceCost {
    resource_id: String,
    amount: f64,
}

#[derive(Deserialize, Clone)]
struct Body {
    name: String,
    parent: Option<String>,
    mass_1e24_kg: Option<f64>,
    radius_km: Option<f64>,
    semi_major_axis_au: Option<f64>,
    eccentricity: Option<f64>,
    inclination_deg: Option<f64>,
    #[allow(dead_code)]
    perihelion_au: Option<f64>,
    #[allow(dead_code)]
    longitude_deg: Option<f64>,
    #[allow(dead_code)]
    omega_lc_deg: Option<f64>,
    #[allow(dead_code)]
    omega_uc_deg: Option<f64>,
    #[allow(dead_code)]
    body_type: Option<i64>,
    orbit_data_source: Option<String>,
    /// Asteroid class identifier (`Carbon`, `Dark`, `Helium3`, `Metal`,
    /// `Stone`) for asteroid bodies, sourced from the body's
    /// `objectSubType` reference when present.  None for any body not
    /// classified per-asteroid in the dump — at the time of writing the
    /// game stores classes only on the per-class roll tables
    /// (`ObjectSubType.Asteroid*` in sirenix-dump.json) and not on the
    /// per-asteroid `ObjectInfo` rows, so this field is None for every
    /// asteroid in the current pipeline.  The asteroid table renders an
    /// em-dash for None and falls back gracefully.
    #[serde(default)]
    asteroid_class: Option<String>,
}

const PLANETS: &[&str] = &[
    "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
];

fn moons_by_parent() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Earth", vec!["Moon"]),
        ("Mars", vec!["Phobos", "Deimos"]),
        ("Jupiter", vec!["Amalthea", "Io", "Europa", "Ganymede", "Callisto"]),
        ("Saturn", vec!["Titan", "Enceladus", "Rhea", "Iapetus", "Tethys", "Mimas", "Hyperion", "Dione"]),
        ("Uranus", vec!["Ariel", "Umbriel", "Titania", "Oberon", "Puck"]),
        ("Neptune", vec!["Triton", "Proteus", "Nereid"]),
        ("Pluto", vec!["Charon"]),
    ]
}

const ASTEROIDS_BELT: &[&str] = &[
    "Ceres", "Pallas", "Juno", "Vesta", "Astraea", "Hebe", "Iris", "Flora",
    "Metis", "Hygiea", "Parthenope", "Victoria", "Egeria",
    "Riema", "Dwornik", "Noviomagum", "Sharp", "Tirza", "Thule", "Haidea", "Duboshin",
];
const ASTEROIDS_NEO: &[&str] = &[
    "Apophis", "Bennu", "Ganymed", "Itokawa", "Ishtar", "Cruithne", "Kamooalewa",
];
const ASTEROIDS_TROJAN_GREEK: &[&str] = &[
    "Patroclus", "Aneas", "Paris", "Achilles", "Hektor", "Agamemnon", "Nestor",
];
const ASTEROIDS_FICTIONAL: &[&str] = &[
    "Peppin", "TJ66", "Terrora", "Kurai", "Koza", "Nosfer", "Kris", "Rider",
    "Usher", "Trus", "Dover", "Nebula", "Sunset", "Geraldino", "Varsoviom",
    "Kutno", "Extinctor",
];
const COMETS: &[&str] = &[
    "1P Halley", "5D Brorsen", "3D Biela", "4P Faye", "2P Encke", "Wild 2", "67P/C-G", "Tempel 1",
];
const EXOPLANETS_TRAPPIST: &[&str] = &[
    "Trappist-1b", "Trappist-1c", "Trappist-1d", "Trappist-1e", "Trappist-1f", "Trappist-1g", "Trappist-1h",
];
const EXOPLANETS_KEPLER: &[&str] = &[
    "Kepler-90b", "Kepler-90c", "Kepler-90d", "Kepler-90e", "Kepler-90f", "Kepler-90g", "Kepler-90h", "Kepler-90i",
];

struct WikiCtx<'a> {
    name_for: BTreeMap<&'a str, &'a str>,
    body_for: BTreeMap<&'a str, &'a Body>,
}

impl<'a> WikiCtx<'a> {
    fn build(locale: &'a Locale, stats: &'a Stats) -> Self {
        let mut name_for = BTreeMap::new();
        for b in &locale.celestial_bodies {
            name_for.insert(b.id.as_str(), b.name.as_str());
        }
        let mut body_for: BTreeMap<&'a str, &'a Body> = BTreeMap::new();
        for b in &stats.bodies {
            body_for.insert(b.name.as_str(), b);
            let trimmed = b.name.trim();
            if trimmed.len() != b.name.len() && !trimmed.is_empty() {
                body_for.entry(trimmed).or_insert(b);
            }
        }
        Self { name_for, body_for }
    }

    fn display<'b>(&self, id: &'b str) -> &'b str
    where
        'a: 'b,
    {
        self.name_for.get(id).copied().unwrap_or(id)
    }

    fn body(&self, id: &str) -> Option<&'a Body> {
        if let Some(b) = self.body_for.get(id) {
            return Some(*b);
        }
        let display = self.name_for.get(id).copied()?;
        let key = display.trim();
        self.body_for.get(key).or_else(|| self.body_for.get(display)).copied()
    }
}

fn fmt_opt(v: Option<f64>, places: usize) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:.places$}"),
        _ => "—".to_string(),
    }
}

fn fmt_mass(v: Option<f64>) -> String {
    match v {
        Some(0.0) | None => "—".to_string(),
        Some(x) if x < 0.001 => format!("{x:.2e}"),
        Some(x) if x < 10.0 => format!("{x:.3}"),
        Some(x) => format!("{x:.1}"),
    }
}

fn fmt_radius(v: Option<f64>) -> String {
    match v {
        Some(0.0) | None => "—".to_string(),
        Some(x) if x < 100.0 => format!("{x:.1}"),
        Some(x) => format!("{x:.0}"),
    }
}

fn fmt_au(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() && x > 0.0 => format!("{x:.4}"),
        _ => "—".to_string(),
    }
}

fn moon_distance_km(body: &Body) -> Option<f64> {
    let a = body.semi_major_axis_au?;
    let factor = if matches!(body.orbit_data_source.as_deref(), Some("OrbitUniversal")) {
        1.0 / 1000.0
    } else {
        1.0
    };
    Some(a * factor * AU_IN_KM)
}

fn fmt_km(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() && x > 0.0 => {
            if x >= 1000.0 {
                format!("{}", (x.round() as i64))
            } else {
                format!("{x:.1}")
            }
        }
        _ => "—".to_string(),
    }
}

fn md_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    md_table_with_tips(headers, &[], rows)
}

/// Same as md_table but each header gets wrapped in a `<span title="…">` when a
/// tooltip is provided.  The CSS-only hover popup in default.html surfaces the
/// description as an instant tooltip; the same prose still appears in the
/// "Reading the table" section below each table for reference.
fn md_table_with_tips(headers: &[&str], tooltips: &[Option<&str>], rows: &[Vec<String>]) -> String {
    let labels: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| match tooltips.get(i).and_then(|x| *x) {
            Some(t) => format!("<span title=\"{}\">{}</span>", t, h),
            None => h.to_string(),
        })
        .collect();
    let mut out = String::new();
    out.push_str("| ");
    out.push_str(&labels.join(" | "));
    out.push_str(" |\n| ");
    out.push_str(&vec!["---"; headers.len()].join(" | "));
    out.push_str(" |\n");
    for r in rows {
        out.push_str("| ");
        out.push_str(&r.join(" | "));
        out.push_str(" |\n");
    }
    out
}

fn escape_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").trim().to_string()
}

fn write_file(root: &Path, rel: &str, content: &str) -> Result<()> {
    let path = root.join(rel);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
    eprintln!("wrote {}", path.display());
    Ok(())
}

fn page_planets(ctx: &WikiCtx) -> String {
    let moons_by_parent = moons_by_parent();
    let moon_counts: BTreeMap<&str, usize> = moons_by_parent
        .iter()
        .map(|(p, m)| (*p, m.len()))
        .collect();

    let rows: Vec<Vec<String>> = PLANETS
        .iter()
        .map(|p| {
            let display = ctx.display(p);
            let body = ctx.body(p);
            let mass = body.and_then(|b| b.mass_1e24_kg);
            let radius = body.and_then(|b| b.radius_km);
            let a = body.and_then(|b| b.semi_major_axis_au);
            let e = body.and_then(|b| b.eccentricity);
            let i = body.and_then(|b| b.inclination_deg);
            let moons = moon_counts.get(p).copied().unwrap_or(0);
            vec![
                format!("**{display}**"),
                fmt_mass(mass),
                fmt_radius(radius),
                fmt_au(a),
                fmt_opt(e, 4),
                fmt_opt(i, 2),
                if moons > 0 { moons.to_string() } else { "—".into() },
            ]
        })
        .collect();

    let table = md_table(
        &["Planet", "Mass (×10²⁴ kg)", "Radius (km)", "Semi-major axis (AU)", "Eccentricity", "Inclination (°)", "Moons"],
        &rows,
    );

    format!(
        "# Planets\n\n\
The nine major planets of the Solar System available in Solar Expanse.\n\n\
{table}\n\
## See also\n\n\
- [Moons](moons.md)\n\
- [Asteroids](asteroids.md)\n\
- [Comets](comets.md)\n\
- [Celestial Bodies overview](README.md)\n"
    )
}

fn page_moons(ctx: &WikiCtx) -> String {
    let mut out = String::from(
        "# Moons\n\n\
Natural satellites orbiting each planet, grouped by parent body. Distance is\n\
measured from the parent planet's center.\n\n",
    );
    for (parent, moons) in moons_by_parent() {
        let parent_name = ctx.display(parent);
        let rows: Vec<Vec<String>> = moons
            .iter()
            .map(|m| {
                let display = ctx.display(m);
                let body = ctx.body(m);
                let mass = body.and_then(|b| b.mass_1e24_kg);
                let dist = body.and_then(moon_distance_km);
                let e = body.and_then(|b| b.eccentricity);
                let i = body.and_then(|b| b.inclination_deg);
                vec![
                    format!("**{display}**"),
                    fmt_mass(mass),
                    fmt_km(dist),
                    fmt_opt(e, 4),
                    fmt_opt(i, 2),
                ]
            })
            .collect();
        let table = md_table(
            &["Moon", "Mass (×10²⁴ kg)", "Distance (km)", "Eccentricity", "Inclination (°)"],
            &rows,
        );
        out.push_str(&format!("## Moons of {parent_name}\n\n{table}\n"));
    }
    out.push_str("## See also\n\n- [Planets](planets.md)\n- [Asteroids](asteroids.md)\n- [Celestial Bodies overview](README.md)\n");
    out
}

fn asteroid_table(ctx: &WikiCtx, ids: &[&str]) -> String {
    let rows: Vec<Vec<String>> = ids
        .iter()
        .map(|id| {
            let display = ctx.display(id);
            let body = ctx.body(id);
            let radius = body.and_then(|b| b.radius_km);
            let a = body.and_then(|b| b.semi_major_axis_au);
            let e = body.and_then(|b| b.eccentricity);
            let i = body.and_then(|b| b.inclination_deg);
            let class_cell = body
                .and_then(|b| b.asteroid_class.as_deref())
                .map(asteroid_class_cell)
                .unwrap_or_else(|| "—".to_string());
            vec![
                format!("**{display}**"),
                class_cell,
                fmt_radius(radius),
                fmt_au(a),
                fmt_opt(e, 4),
                fmt_opt(i, 2),
            ]
        })
        .collect();
    md_table_with_tips(
        &["Asteroid", "Class", "Radius (km)", "Semi-major axis (AU)", "Eccentricity", "Inclination (°)"],
        &[
            None,
            Some("Mining roll table — Carbon / Dark / Helium-3 / Metal / Stone."),
            None,
            None,
            None,
            None,
        ],
        &rows,
    )
}

/// Render a raw asteroid-class name (`Carbon`, `Dark`, `Helium3`, `Metal`,
/// `Stone`) as a markdown link to the matching anchor on the
/// `../asteroid-taxonomy/` page, with a hover tooltip naming the full set
/// of classes.  GitHub's automatic header anchors lowercase the H2 text
/// and replace spaces with hyphens, so `## Helium-3 Asteroid` becomes
/// `#helium-3-asteroid` — we mirror that here.
fn asteroid_class_cell(raw: &str) -> String {
    let display = asteroid_class_display(raw);
    let anchor = format!("{}-asteroid", display.to_ascii_lowercase());
    format!(
        "[<span title=\"Mining roll table — Carbon / Dark / Helium-3 / Metal / Stone.\">{display}</span>](../asteroid-taxonomy/#{anchor})"
    )
}

fn page_asteroids(ctx: &WikiCtx) -> String {
    let belt = asteroid_table(ctx, ASTEROIDS_BELT);
    let neo = asteroid_table(ctx, ASTEROIDS_NEO);
    let trojan = asteroid_table(ctx, ASTEROIDS_TROJAN_GREEK);
    let other = asteroid_table(ctx, ASTEROIDS_FICTIONAL);
    format!(
        "# Asteroids\n\n\
Small bodies that can be probed, mined, captured, and in some cases pushed into\n\
new orbits using mass-driver engine modules.\n\n\
## Main Belt (Inner / Middle / Outer)\n\n\
The classical belt between Mars and Jupiter. The game subdivides the belt into Inner, Middle, and Outer regions.\n\n\
{belt}\n\
## Near-Earth Objects (NEOs)\n\n\
Asteroids on orbits that bring them close to Earth — early-game targets for sample-return contracts.\n\n\
{neo}\n\
## Jupiter Trojans and Greeks\n\n\
Co-orbital with Jupiter at the L4 (Greeks) and L5 (Trojans) Lagrange points.\n\n\
{trojan}\n\
## Other Asteroids\n\n\
Procedural and named bodies that appear in scenarios beyond the canonical roster.\n\n\
{other}\n\
## See also\n\n\
- [Asteroid Taxonomy](../asteroid-taxonomy/) — per-class mining roll probabilities (what each class yields when you mine)\n\
- [Comets](comets.md)\n\
- [Celestial Bodies overview](README.md)\n"
    )
}

/// Render an asteroid-class' raw name (the bit after `ObjectSubType.Asteroid`)
/// into a player-facing label.  Only `Helium3` needs a non-trivial transform
/// (→ `Helium-3`); the others (Carbon, Dark, Metal, Stone) pass through.
fn asteroid_class_display(name: &str) -> String {
    match name {
        "Helium3" => "Helium-3".to_string(),
        other => other.to_string(),
    }
}

/// Format a 0.0–1.0 probability as a percentage with no trailing decimal noise.
/// `1.0` → `100%`, `0.45` → `45%`, `0.105` → `10.5%`.
fn fmt_probability(p: f64) -> String {
    let pct = p * 100.0;
    if (pct - pct.round()).abs() < 1e-6 {
        format!("{}%", pct.round() as i64)
    } else {
        // One decimal is enough for everything the dump has emitted.
        format!("{:.1}%", pct)
    }
}

/// Per-class mining roll tables sourced from `ObjectSubType[]` in the
/// Sirenix dump.  This is a taxonomy page — it answers "if I land on a
/// Carbon-class asteroid and mine a deposit, what might I get?" — not a
/// per-asteroid annotation (per-asteroid class isn't part of the saved
/// data we currently parse).
fn page_asteroid_taxonomy(locale: &Locale, sirenix: &Sirenix) -> String {
    let res_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    let mut classes: Vec<&AsteroidClassStat> = sirenix.asteroid_classes.iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));

    let mut sections = String::new();
    for class in classes {
        let label = asteroid_class_display(&class.name);
        sections.push_str(&format!("## {label} Asteroid\n\n"));
        let mut rows: Vec<Vec<String>> = Vec::new();
        for tier in class.tiers.iter().filter(|t| !t.rolls.is_empty()) {
            for roll in &tier.rolls {
                let resource_label = res_name
                    .get(roll.resource_id.as_str())
                    .copied()
                    .unwrap_or(roll.resource_id.as_str());
                rows.push(vec![
                    tier.category.clone(),
                    resource_label.to_string(),
                    fmt_probability(roll.probability),
                ]);
            }
        }
        sections.push_str(&md_table(&["Tier", "Resource", "Probability"], &rows));
        sections.push('\n');
    }

    format!(
        "# Asteroid Taxonomy\n\n\
Each asteroid has a class — Carbon, Dark, Helium-3, Metal, or Stone.\n\
When you mine a deposit on an asteroid, the resource you actually extract\n\
is rolled from the table below, grouped by the deposit's quality tier\n\
(High, Mid, or Low).  Probabilities within each tier sum to 100%.\n\n\
{sections}\
## See also\n\n\
- [Asteroids](../celestial-bodies/asteroids.md) — list of named asteroids in the game\n\
- [Resources](../resources/) — what each mined resource is used for\n"
    )
}

fn page_comets(ctx: &WikiCtx) -> String {
    let rows: Vec<Vec<String>> = COMETS
        .iter()
        .map(|c| {
            let display = ctx.display(c);
            let body = ctx.body(c);
            let radius = body.and_then(|b| b.radius_km);
            let a = body.and_then(|b| b.semi_major_axis_au);
            let e = body.and_then(|b| b.eccentricity);
            let i = body.and_then(|b| b.inclination_deg);
            vec![
                format!("**{display}**"),
                fmt_radius(radius),
                fmt_au(a),
                fmt_opt(e, 4),
                fmt_opt(i, 2),
            ]
        })
        .collect();
    let table = md_table(
        &["Comet", "Radius (km)", "Semi-major axis (AU)", "Eccentricity", "Inclination (°)"],
        &rows,
    );
    format!(
        "# Comets\n\n\
Periodic comets that pass through the inner system on highly eccentric orbits.\n\n\
{table}\n\
## See also\n\n\
- [Asteroids](asteroids.md)\n\
- [Celestial Bodies overview](README.md)\n"
    )
}

fn exoplanet_table(ctx: &WikiCtx, ids: &[&str]) -> String {
    let rows: Vec<Vec<String>> = ids
        .iter()
        .map(|id| {
            let display = ctx.display(id);
            let body = ctx.body(id);
            let mass = body.and_then(|b| b.mass_1e24_kg);
            let radius = body.and_then(|b| b.radius_km);
            let a = body.and_then(|b| b.semi_major_axis_au);
            let e = body.and_then(|b| b.eccentricity);
            let i = body.and_then(|b| b.inclination_deg);
            vec![
                format!("**{display}**"),
                fmt_mass(mass),
                fmt_radius(radius),
                fmt_au(a),
                fmt_opt(e, 4),
                fmt_opt(i, 2),
            ]
        })
        .collect();
    md_table(
        &["Planet", "Mass (×10²⁴ kg)", "Radius (km)", "Semi-major axis (AU)", "Eccentricity", "Inclination (°)"],
        &rows,
    )
}

fn page_exoplanets(_ctx: &WikiCtx, sirenix: &Sirenix) -> String {
    // Both `/celestial-bodies/exoplanets.md` and `/exoplanets/` render the
    // same dump-driven content (4 systems with real orbital data).  The
    // legacy hand-curated em-dash placeholder table was being served at the
    // Bodies-nav path; consolidating onto `page_exoplanets_systems` so the
    // Bodies-nav URL no longer shows empty cells.
    page_exoplanets_systems(sirenix)
}

fn page_launch_windows(ctx: &WikiCtx) -> String {
    let earth = match ctx.body("Earth") {
        Some(b) => b,
        None => return "# Launch Windows\n\nEarth data not available.\n".into(),
    };
    let earth_a = match earth.semi_major_axis_au {
        Some(a) if a > 0.0 => a,
        _ => return "# Launch Windows\n\nEarth orbital data not available.\n".into(),
    };
    let earth_period_years = earth_a.powf(1.5);

    // Build the full set of sun-orbiting targets from every taxonomy bucket,
    // remembering which bucket each id came from so we can label its Type.
    let mut targets: Vec<(&str, &'static str)> = Vec::new();
    targets.extend(
        PLANETS
            .iter()
            .filter(|p| **p != "Earth")
            .map(|p| (*p, "Planet")),
    );
    for &id in ASTEROIDS_BELT.iter() { targets.push((id, "Asteroid")); }
    for &id in ASTEROIDS_NEO.iter() { targets.push((id, "Asteroid")); }
    for &id in ASTEROIDS_TROJAN_GREEK.iter() { targets.push((id, "Asteroid")); }
    for &id in ASTEROIDS_FICTIONAL.iter() { targets.push((id, "Asteroid")); }
    for &id in COMETS.iter() { targets.push((id, "Comet")); }

    // Collect (display, a, t_years, synodic_years, longitude, body_type) for everything we can match.
    let mut data: Vec<(String, f64, f64, f64, f64, &'static str)> = Vec::new();
    for (id, body_type) in &targets {
        let b = match ctx.body(id) {
            Some(b) => b,
            None => continue,
        };
        if !matches!(b.orbit_data_source.as_deref(), Some("SolarBody")) {
            continue;
        }
        let a = match b.semi_major_axis_au {
            Some(a) if a > 0.0 => a,
            _ => continue,
        };
        let t_years = a.powf(1.5);
        let inv = 1.0 / earth_period_years - 1.0 / t_years;
        if inv.abs() < 1e-9 {
            continue;
        }
        let synodic_years = 1.0 / inv.abs();
        let display = ctx.display(id).to_string();
        let longitude = b.longitude_deg.unwrap_or(0.0);
        data.push((display, a, t_years, synodic_years, longitude, *body_type));
    }
    data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Synodic-period overview table.  Each row gets a `data-body-type`
    // attribute via a small "Type" column — the JS filter uses that to
    // hide/show by type.
    let table_rows: Vec<Vec<String>> = data
        .iter()
        .map(|(display, a, t_years, syn, _, body_type)| {
            let synodic_days = syn * 365.25;
            let base_label = if *syn < 2.0 {
                format!("{:.0} days (~{:.1} months)", synodic_days, syn * 12.0)
            } else {
                format!("{:.1} years", syn)
            };
            // Bodies in near-1:1 resonance with Earth (semi-major axis very
            // close to 1 AU) yield synodic periods of centuries — the math
            // is right but the number reads as a bug.  Flag those rows.
            let synodic_label = if *syn > 50.0 {
                format!(
                    "<span title=\"This body's orbital period is nearly Earth's, so the synodic-period formula produces an extreme value. See the Practical reading bullet below.\">{} *(near-resonance — see note below)*</span>",
                    base_label
                )
            } else {
                base_label
            };
            vec![
                format!("**{}**", display),
                body_type.to_string(),
                format!("{:.3}", a),
                format!("{:.2} yr", t_years),
                synodic_label,
            ]
        })
        .collect();
    let table = md_table_with_tips(
        &["Body", "Type", "Semi-major axis (AU)", "Orbital period", "Earth ↔ body window"],
        &[
            None,
            Some("Planet, Asteroid, or Comet — used by the filter checkboxes above"),
            Some("Average distance from the Sun in astronomical units (1 AU = Earth's distance)"),
            Some("Time for one orbit around the Sun, derived from a via Kepler's third law"),
            Some("Interval between consecutive Hohmann-style launch opportunities from Earth — the synodic period"),
        ],
        &table_rows,
    );

    // Embed every body's orbital data for the calculator so the user can
    // pick any from/to pair.  Includes Earth so it can be on either side.
    // Also includes the Sun as a synthetic flyby body (a = 0) — the
    // gravity-assist calculator treats this as a solar Oberth maneuver.
    let mut calc_bodies: Vec<String> = Vec::new();
    // Earth first
    calc_bodies.push(format!(
        "{{\"name\":\"Earth\",\"a\":{a},\"longitude\":{lng}}}",
        a = earth_a,
        lng = earth.longitude_deg.unwrap_or(0.0),
    ));
    // Sun next — a = 0 signals "solar Oberth" to the JS code.  Name chosen
    // to match the game's "Solar (orbit)" route node terminology while
    // still reading naturally in the typeahead.
    calc_bodies.push("{\"name\":\"Sun\",\"a\":0,\"longitude\":0}".to_string());
    for (display, a, _t, _syn, longitude, _type) in &data {
        calc_bodies.push(format!(
            "{{\"name\":\"{name}\",\"a\":{a},\"longitude\":{lng}}}",
            name = display.replace('"', "\\\""),
            a = a,
            lng = longitude,
        ));
    }
    let calc_data = format!("[{}]", calc_bodies.join(","));

    format!(
        "# Launch Windows\n\n\
**Jump to:** [Window calculator](#window-calculator) · [Earth ↔ body table](#body-table) · [Gravity-assist trajectory](#gravity-assist)\n\n\
> **Heads-up:** these numbers are computed by the wiki from the orbital\n\
> elements the game ships, *not* read from the game itself.  The in-game\n\
> Plan Mission window uses live n-body propagation including gravitational\n\
> perturbations and your spacecraft's specific Δv budget, so the dates and\n\
> intervals here are a **planning approximation** — the porkchop plot is\n\
> the source of truth at launch time.\n\n\
## What counts as a launch window\n\n\
A *launch window* here is the moment when an idealized **Hohmann transfer**\n\
launched from one body's orbit will arrive at the target body just as that\n\
body reaches the transfer ellipse's far side.  Concretely, at the moment of\n\
launch the target has to lead (for outer bodies) or trail (for inner bodies)\n\
the origin by a specific phase angle so that body and spacecraft meet on\n\
arrival.  Earth–Mars windows recur every ~26 months (synodic period); the\n\
most recent real-world ones were 2020-07, 2022-09, 2024-10.\n\n\
This is a single idealised window per synodic period — *not* a multi-day\n\
porkchop plot.  In practice the in-game planner gives you a range of days\n\
on either side at slightly higher Δv cost; the table here is the centre of\n\
that range.\n\n\
The **synodic period** is how often the Earth-body pair returns to that\n\
same relative geometry.  Computed from each body's semi-major axis via\n\
Kepler's third law (`T_years = a^(3/2)`) and\n\
`synodic = 1 / |1/T_earth − 1/T_body|`.\n\n\
## Window calculator\n\n\
<a id=\"window-calculator\"></a>\n\n\
Pick a *from* body, *to* body, and a start date.  The calculator lists the\n\
next five Hohmann-transfer launch windows from that pair, plus the arrival\n\
date for each (transfer time = `0.5 × ((a_from + a_to) / 2)^1.5` years).\n\
The body fields are typeahead — start typing and pick from the dropdown.\n\
The start date defaults to **2020-01-01**, the game's campaign start year.\n\n\
<div class=\"calc\">\n\
<label>From: <input id=\"calc-from\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Earth\"></label>\n\
<label>To: <input id=\"calc-to\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Mars\"></label>\n\
<label>Start date: <input type=\"date\" id=\"calc-date\" value=\"2020-01-01\"></label>\n\
<button id=\"calc-submit\" type=\"button\">Calculate</button>\n\
<datalist id=\"calc-bodies\"></datalist>\n\
<div id=\"calc-result\"></div>\n\
</div>\n\n\
<script>\n\
window.LAUNCH_WINDOW_ALL_BODIES = {data};\n\
window.LAUNCH_WINDOW_EARTH = {{\"a\":{earth_a},\"longitude\":{earth_lng}}};\n\
</script>\n\
<script src=\"{{{{ '/assets/js/launch-windows.js' | relative_url }}}}?v={{{{ site.data.wiki.generated_at }}}}\"></script>\n\n\
## Earth ↔ body launch windows\n\n\
The static table below shows the synodic period — i.e. how often an Earth-to-body launch window opens — for every sun-orbiting target. For from-other-body launch windows, use the calculator above.\n\n\
<div id=\"body-table\" markdown=\"1\">\n\
<div class=\"body-filters\">\n\
<label>Filter: <input id=\"body-filter\" type=\"search\"></label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Planet\" checked> Planets</label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Asteroid\"> Asteroids</label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Comet\"> Comets</label>\n\
</div>\n\n\
*Moons aren't listed — launch windows are computed from each body's heliocentric orbit, so to reach a moon you target its **parent planet** in this table (e.g. Phobos → Mars, Europa → Jupiter, Titan → Saturn). The moon's position around the parent is handled inside the in-game flight planner.*\n\n\
{table}\n\
</div>\n\n\
## Practical reading\n\n\
- **Earth → Mercury** opens most often — ~116 days, less than every 4 months.\n\
- **Earth → Venus** ~19 months.\n\
- **Earth → Mars** opens roughly every 26 months — every mid-game player has\n\
  watched their cargo manifest waiting for one of these.\n\
- **Earth → Jupiter and beyond** are short intervals (~13 months) because the\n\
  outer planets move slowly relative to Earth, so Earth laps them almost\n\
  yearly.  The Hohmann transfer itself takes years.\n\
- Asteroid-belt bodies sit between Mars and Jupiter — windows ~14–16 months.\n\
- **Near-resonance bodies** (Cruithne at 0.998 AU, Kamoʻoalewa at 1.001 AU) share Earth's orbital period almost exactly, so `1/T_earth − 1/T_body` is tiny and the synodic-period formula produces multi-century intervals. The number is mathematically correct but practically meaningless — these bodies are effectively co-orbital, so any month is a launch month and the in-game planner handles phasing directly.\n\n\
Moons aren't here — launching from Earth to the Moon (or Phobos, Europa, etc.)\n\
doesn't have a useful synodic period; you wait for your spacecraft to be\n\
ready and the in-game flight planner handles phasing.\n\n\
## Gravity-assist trajectory\n\n\
<a id=\"gravity-assist\"></a>\n\n\
> **Heads-up:** these trajectories are computed by the wiki using a\n\
> patched-conic model on circular coplanar orbits.  The in-game Plan\n\
> Mission window uses full n-body propagation, so the dates, Δv values,\n\
> and even the best flyby choice may not match what the game's flight\n\
> planner reports.  Treat this as a **first-cut planning tool**, not a\n\
> precise trajectory — confirm in-game before committing to a craft.\n\n\
For outer-system targets a *gravity assist* — a deep flyby of an intermediate\n\
body that bends the spacecraft's trajectory at no propellant cost — can cut\n\
the launch Δv dramatically.  Pick any *from*, *flyby*, and *to* body and the\n\
calculator searches a coarse grid of launch and flyby dates, returning the\n\
lowest-cost single-flyby trajectory it can find.\n\n\
**Important caveats:**\n\n\
- This is a **single** gravity assist (one intermediate body).  Real\n\
  outer-planet missions usually chain several — Cassini did Venus-Venus-Earth-Jupiter,\n\
  for example — and those aren't modelled here.\n\
- It's a **patched-conic** approximation: each leg is a heliocentric Kepler\n\
  arc and the flyby itself is treated as an instantaneous rotation of the\n\
  v∞ vector.  In particular, the flyby is assumed capable of bending v∞ by\n\
  any angle for free (the actual maximum bend depends on flyby altitude\n\
  and the body's mass).\n\
- Bodies are assumed to move on **circular coplanar** orbits anchored at\n\
  the game's epoch — same Keplerian approximation the window calculator\n\
  above uses.\n\n\
The reported \"Δv proxy\" is `|v_spacecraft − v_origin|` at launch plus\n\
`|v_spacecraft − v_target|` at arrival, both expressed in km/s; it\n\
ignores escape Δv from low Earth orbit and capture Δv at the target.\n\n\
### Trajectory calculator\n\n\
Pick a *from* body and a *to* body and the calculator scans every other\n\
body in the game as a candidate flyby (planets, asteroids, comets, and the\n\
Sun itself as a solar-Oberth maneuver).  It returns the **direct** (no-flyby)\n\
trajectory plus the top five flyby routes that beat direct by the most Δv.\n\
The scan runs entirely in your browser — expect 5–30 seconds depending on\n\
window length.\n\n\
<div class=\"calc\">\n\
<label>From: <input id=\"ga-from\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Earth\"></label>\n\
<label>To: <input id=\"ga-to\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Pluto\"></label>\n\
<label>Search from: <input type=\"date\" id=\"ga-date\" value=\"2020-01-01\"></label>\n\
<button id=\"ga-submit\" type=\"button\">Calculate</button>\n\
<div id=\"ga-result\"></div>\n\
</div>\n\n\
<script src=\"{{{{ '/assets/js/gravity-assist.js' | relative_url }}}}?v={{{{ site.data.wiki.generated_at }}}}\"></script>\n\n\
## See also\n\n\
- [Planets](planets.md)\n\
- [Celestial Bodies overview](README.md)\n",
        data = calc_data,
        earth_a = earth_a,
        earth_lng = earth.longitude_deg.unwrap_or(0.0),
    )
}

fn page_celestial_index() -> String {
    let asteroid_count = ASTEROIDS_BELT.len()
        + ASTEROIDS_NEO.len()
        + ASTEROIDS_TROJAN_GREEK.len()
        + ASTEROIDS_FICTIONAL.len();
    let moon_count = moons_by_parent().iter().map(|(_, m)| m.len()).sum::<usize>();
    let exoplanet_count = EXOPLANETS_TRAPPIST.len() + EXOPLANETS_KEPLER.len();
    let rows: Vec<Vec<String>> = vec![
        vec![
            "**[Planets](planets.md)**".into(),
            PLANETS.len().to_string(),
            "Major body orbiting the Sun. Most planets host one or more moons.".into(),
        ],
        vec![
            "**[Moons](moons.md)**".into(),
            moon_count.to_string(),
            "Natural satellite orbiting a planet.".into(),
        ],
        vec![
            "**[Asteroids](asteroids.md)**".into(),
            asteroid_count.to_string(),
            "Small body. Some are in the main belt, some are near-Earth, and some co-orbit Jupiter at the Trojan/Greek points. Asteroids can be pulled into new orbits with an Asteroid Engine Module.".into(),
        ],
        vec![
            "**[Comets](comets.md)**".into(),
            COMETS.len().to_string(),
            "Periodic body on a highly eccentric orbit.".into(),
        ],
        vec![
            "**[Exoplanets](exoplanets.md)**".into(),
            exoplanet_count.to_string(),
            "Body in a non-Solar system. Reachable only via a generation ship.".into(),
        ],
    ];
    let count_table = md_table(&["Type", "Count", "Notes"], &rows);
    format!(
        "# Celestial Bodies\n\n\
All natural objects in Solar Expanse — from the nine planets, through\n\
moons and asteroid belts, out to comets and the Trappist-1 and Kepler-90\n\
exoplanet systems reachable in the late game.\n\n\
{count_table}\n\
## Orbital data\n\n\
Orbital elements below are anchored at the **2020-01-01 campaign-start epoch**\n\
the game ships — the same epoch the in-game flight planner uses for its\n\
initial body positions.\n\n\
| Field | Meaning | Unit |\n\
| --- | --- | --- |\n\
| Mass | Body mass | 10²⁴ kg |\n\
| Radius | Mean radius | km |\n\
| Semi-major axis | Average orbital radius (around the Sun for planets, around the parent for moons) | AU (planets), km (moons) |\n\
| Eccentricity | Orbital ellipticity (0 = circular) | dimensionless |\n\
| Inclination | Tilt relative to the ecliptic | degrees |\n\n\
## Habitability\n\n\
The Object Info window grades every body on four habitability axes:\n\n\
| Axis | Labels (worst → best) |\n\
| --- | --- |\n\
| Temperature | Extremely Cold · Cold · Temperate · Hot · Extremely Hot · Melting Hot |\n\
| Atmosphere | No Atmosphere · Thin Atmosphere · Earth-like Atmosphere · Non-breathable · High Pressure · Extreme Pressure |\n\
| Gravitation | Extreme Gravity · High Gravity · Standard Gravity · Low Gravity · Minimal Gravity · 0g |\n\
| Radiation | No Radiation · Minor · Noticeable · Significant · Serious hazard · Extreme hazard |\n\n\
Combined into a single **Habitability %**, with crew status:\n\n\
| Habitability | Crew status |\n\
| --- | --- |\n\
| Excellent (≈100%) | A perfect place for life. |\n\
| Good | Our crews can live here with minor issues. |\n\
| Marginal | Our crews will struggle to survive here. |\n\
| Hostile | Our crews cannot land here — the object is too hostile. |\n\n\
Habitability can be improved through terraforming.\n\n\
## Pages\n\n\
- [Planets](planets.md) — the nine major bodies\n\
- [Moons](moons.md) — natural satellites of each planet\n\
- [Asteroids](asteroids.md) — main belt, NEOs, Trojans/Greeks, and others\n\
- [Comets](comets.md) — periodic comets\n\
- [Exoplanets](exoplanets.md) — Trappist-1 and Kepler-90 systems\n\
- [Launch Windows](launch-windows.md) — synodic periods for planning Earth → body missions\n\
- [Initial habitability per scenario](scenario-state.md) — start-of-scenario temperature, pressure, gravity, water, radiation, etc. for each named body, compared across the four pre-built saves\n"
    )
}

fn engine_category_for(stat: &SpacecraftStat) -> u8 {
    // Sort order: Chemical, Electric, Nuclear, Fusion, Solar, Other
    match stat.engine_type.as_str() {
        "chemical" => 0,
        "electric" => 1,
        "nuclear" => 2,
        "fusion" => 3,
        "solar" => 4,
        _ => 5,
    }
}

/// HTML-safe anchor id for an entry row.  e.g. ("research", "research_chem_main1")
/// → "research-research_chem_main1".  Non-alphanumeric characters become dashes.
fn anchor_id(kind: &str, id: &str) -> String {
    let slug: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("{kind}-{slug}")
}

/// Inline anchor tag to place before an entry's display name so other rows can
/// link directly to it.
fn anchor_tag(kind: &str, id: &str) -> String {
    format!("<a id=\"{}\"></a>", anchor_id(kind, id))
}

/// Markdown link to an entry on another page.  `page_dir` is the page's URL
/// directory (e.g., "research", "facilities"), relative to /docs/.
fn link_cross_page(page_dir: &str, kind: &str, id: &str, display: &str) -> String {
    format!("[{display}](../{page_dir}/#{anchor})", anchor = anchor_id(kind, id))
}

/// Markdown link to another row on the *same* page.
fn link_same_page(kind: &str, id: &str, display: &str) -> String {
    format!("[{display}](#{anchor})", anchor = anchor_id(kind, id))
}

fn fmt_amount(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e9 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}

/// Abbreviate large numbers with k/M/B suffixes for cargo, money, and resource
/// quantities.  Values below 1,000 are written out in full.
fn fmt_abbrev(v: f64) -> String {
    if v <= 0.0 {
        return "0".into();
    }
    let (scaled, suffix) = if v >= 1e9 {
        (v / 1e9, "B")
    } else if v >= 1e6 {
        (v / 1e6, "M")
    } else if v >= 1e3 {
        (v / 1e3, "k")
    } else {
        return fmt_amount(v);
    };
    if (scaled - scaled.round()).abs() < 0.05 {
        format!("{:.0}{}", scaled, suffix)
    } else {
        format!("{:.1}{}", scaled, suffix)
    }
}

fn fmt_build_cost(cost: &[ResourceCost], resource_name: &BTreeMap<&str, &str>) -> String {
    if cost.is_empty() {
        return "—".into();
    }
    cost.iter()
        .map(|c| {
            let label = resource_name.get(c.resource_id.as_str()).copied().unwrap_or(c.resource_id.as_str());
            // Render each item as a small game icon followed by the abbreviated
            // amount.  The icons live in /images/resources/<id>.png and were
            // cropped out of the game's sprite atlas by extract-icons.
            // `alt` covers screen readers / icon-load failures.
            // white-space:nowrap keeps the icon and amount on one line even
            // when the cell is narrow.  The wrapping span also widens the
            // tooltip hover target to the whole token.
            format!(
                "<span style=\"white-space:nowrap\" title=\"{label}\"><img src=\"../images/resources/{id}.png\" width=\"16\" alt=\"{label}\"/>&nbsp;{amount}</span>",
                id = c.resource_id,
                label = label,
                amount = fmt_abbrev(c.amount),
            )
        })
        .collect::<Vec<_>>()
        .join("<br>")
}

fn fmt_reusability(r: f64) -> &'static str {
    if r <= 0.01 {
        "No"
    } else if r >= 0.99 {
        "Yes"
    } else {
        "Partial"
    }
}

fn fmt_thrust(kn: f64) -> String {
    // SpaceComponent.thrust is stored in newtons.  Display as kN or MN for readability.
    if kn <= 0.0 {
        "—".into()
    } else if kn >= 1_000_000.0 {
        format!("{:.1} MN", kn / 1_000_000.0)
    } else if kn >= 1_000.0 {
        format!("{:.0} kN", kn / 1_000.0)
    } else {
        format!("{:.0} N", kn)
    }
}

fn fmt_exhaust(v: f64) -> String {
    // exhaustV is stored in km/s.
    if v <= 0.0 {
        "—".into()
    } else {
        format!("{:.1} km/s", v)
    }
}

fn page_spacecraft(locale: &Locale, sirenix: &Sirenix) -> String {
    let id_to_name: BTreeMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|x| (x.id.as_str(), x.name.as_str()))
        .collect();
    let id_to_desc: BTreeMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|x| (x.id.as_str(), x.description.as_str()))
        .collect();
    let resource_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    let component_by_id: BTreeMap<&str, &SpaceComponentStat> = sirenix
        .space_components
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();

    // Reverse map: spacecraft_id → research_id of the node that unlocks it.
    // Built from research entries with `action == "UnlockSpacecraftType"`.
    let research_unlocking_sc: BTreeMap<&str, &str> = sirenix
        .research
        .iter()
        .filter(|r| r.action == "UnlockSpacecraftType")
        .filter_map(|r| r.unlock_target.as_deref().map(|t| (t, r.id.as_str())))
        .collect();
    let research_display_name: BTreeMap<&str, &str> = locale
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    // Keep only entries that have a player-facing locale name AND a populated stat row.
    let mut entries: Vec<&SpacecraftStat> = sirenix
        .spacecraft
        .iter()
        .filter(|s| id_to_name.get(s.id.as_str()).map_or(false, |n| !n.is_empty()))
        .collect();
    entries.sort_by(|a, b| {
        engine_category_for(a)
            .cmp(&engine_category_for(b))
            .then(a.mass.partial_cmp(&b.mass).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.id.cmp(&b.id))
    });

    let mut out = String::from(
        "# Spacecraft\n\n\
Interplanetary vehicles — capable of operating between orbits, sometimes landing\n\
on planets, but built and launched separately from the launch vehicles that\n\
lift them to space.\n\n",
    );

    // Group by engine category, render a table per group.
    let mut current = u8::MAX;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let flush = |out: &mut String, rows: &mut Vec<Vec<String>>, header: &str| {
        if rows.is_empty() {
            return;
        }
        out.push_str(&format!("## {header}\n\n"));
        out.push_str(&md_table_with_tips(
            &[
                "Spacecraft",
                "Mass (t)",
                "Cargo (t)",
                "Fuel (t)",
                "Thrust",
                "Exhaust V",
                "Reusable",
                "Built at",
                "Requires LV",
                "Build cost",
                "Time (d)",
                "Description",
            ],
            &[
                None,
                Some("Dry mass in tonnes"),
                Some("Cargo capacity in tonnes"),
                Some("Fuel capacity in tonnes"),
                Some("Default engine thrust"),
                Some("Effective exhaust velocity — chemical ~3-5 km/s, nuclear ~8-15, fusion 20+"),
                Some("Survives the trip and can fly again (Yes / Partial / No)"),
                Some("Where the spacecraft is assembled — Orbit means built in an orbital shipyard; Surface means built on a planet"),
                Some("When the craft needs a launch vehicle to leave a planet/moon surface. Earth always requires an LV; on lower-gravity bodies many craft can self-launch."),
                Some("Resources required to construct"),
                Some("Build time in days"),
                None,
            ],
            rows,
        ));
        out.push('\n');
        rows.clear();
    };

    for s in &entries {
        let cat = engine_category_for(s);
        if cat != current {
            let header = match current {
                0 => "Chemical Propulsion",
                1 => "Electric Propulsion",
                2 => "Nuclear Propulsion",
                3 => "Fusion Propulsion",
                4 => "Solar Sails",
                _ => "Other",
            };
            flush(&mut out, &mut rows, header);
            current = cat;
        }
        let display_name = id_to_name.get(s.id.as_str()).copied().unwrap_or(s.id.as_str());
        let desc = id_to_desc.get(s.id.as_str()).copied().unwrap_or("");
        let engine = s
            .engine_module
            .as_deref()
            .and_then(|id| component_by_id.get(id).copied());
        let thrust = engine.map(|e| fmt_thrust(e.thrust)).unwrap_or_else(|| "—".into());
        let exhaust = engine.map(|e| fmt_exhaust(e.exhaust_v)).unwrap_or_else(|| "—".into());
        // The Orbital Payload Container is the only spacecraft that isn't
        // player-built at all — it's spawned by the launch elevator / mass
        // driver / spin launch / catapult facilities.  In the dump this is
        // signalled by an empty build_cost AND build_time_days == 0 (other
        // upper-stage craft like Centaur have non-zero build time even when
        // their cost is empty).  For this oddball, render the build columns
        // and "Built at" as em-dash rather than fabricating "Surface" / 0d.
        let is_spawned_not_built = s.build_cost.is_empty() && s.build_time_days == 0.0;
        let built_at: String = if is_spawned_not_built {
            "—".into()
        } else if s.built_in_orbit {
            "Orbit".into()
        } else {
            "Surface".into()
        };
        // Launch-vehicle requirement comes from `SpacecraftType.needLaunchVehicleToGoToMoon`
        // combined with the game's gate in `FindSuitableLaunchVehicleForFlight`:
        //   - If the craft is built in orbit, it never surfaces (label that explicitly).
        //   - If the flag is true, the craft needs an LV from *every* planet/moon.
        //   - If the flag is false, the craft can self-launch from low-G bodies
        //     (Luna, Mars, asteroids), but Earth's special case as
        //     `Company.mainObjectInfo` forces an LV anyway.
        let launch_vehicle_cell: String = if is_spawned_not_built {
            // The Orbital Payload Container is spawned by a launch facility
            // (elevator, mass driver, spin launch, catapult) rather than
            // launched conventionally — the LV column doesn't apply.
            "—".into()
        } else if s.built_in_orbit {
            "Built in orbit".into()
        } else if s.needs_launch_vehicle {
            "Any body".into()
        } else {
            "Earth only".into()
        };
        let build_cost_cell = if is_spawned_not_built {
            "—".into()
        } else {
            fmt_build_cost(&s.build_cost, &resource_name)
        };
        let build_time_cell = if is_spawned_not_built {
            "—".into()
        } else {
            fmt_amount(s.build_time_days)
        };
        // The OPC's hull cargoCapacity is a 999999 sentinel meaning "limited by
        // the carrier launch vehicle" rather than the hull.  Rendering 999999 is
        // misleading; the player-verified in-game value is 800 t (constrained by
        // the Super-heavy lifter / launch facility payload caps).
        let cargo_cell = if is_spawned_not_built && s.cargo_capacity >= 999_999.0 {
            "800".into()
        } else {
            fmt_amount(s.cargo_capacity)
        };
        // Prepend a research-unlock link to the description cell rather than
        // the name cell — putting it under the name wrapped most of the time
        // and pushed the table layout around.  Putting it before the desc
        // text keeps the column where the long text already is.
        let unlock_prefix = research_unlocking_sc
            .get(s.id.as_str())
            .map(|rid| {
                let label = research_display_name
                    .get(rid)
                    .copied()
                    .filter(|n| !n.is_empty())
                    .unwrap_or(rid);
                format!(
                    "<sub>Unlock: {}</sub><br>",
                    link_cross_page("research", "research", rid, &escape_cell(label))
                )
            })
            .unwrap_or_default();
        let desc_cell = format!("{}{}", unlock_prefix, escape_cell(desc));
        rows.push(vec![
            format!(
                "{anchor}**{name}**",
                anchor = anchor_tag("spacecraft", &s.id),
                name = escape_cell(display_name)
            ),
            fmt_amount(s.mass),
            cargo_cell,
            fmt_amount(s.fuel_capacity),
            thrust,
            exhaust,
            fmt_reusability(s.reusability).into(),
            built_at,
            launch_vehicle_cell,
            build_cost_cell,
            build_time_cell,
            desc_cell,
        ]);
    }
    let header = match current {
        0 => "Chemical Propulsion",
        1 => "Electric Propulsion",
        2 => "Nuclear Propulsion",
        3 => "Fusion Propulsion",
        4 => "Solar Sails",
        _ => "Other",
    };
    flush(&mut out, &mut rows, header);
    out.push_str("\n");

    out.push_str(
        "## Reading the table\n\n\
- **Mass / Cargo / Fuel** are listed in tonnes; capacities are the spacecraft's hull limit before any module changes.\n\
- **Engine thrust** is the force the spacecraft's default engine produces, in newtons (or kilo-/mega-newtons for readability). More thrust = shorter burns, higher acceleration, but the spacecraft can only carry so much fuel.\n\
- **Exhaust V** is the engine's effective exhaust velocity in km/s, equivalent to specific impulse (multiply by ~102 to get ISP in seconds). Higher exhaust V = more Δv per kilogram of fuel = longer reach, but typically less thrust. Chemical engines sit around 3–5 km/s; nuclear thermal 8–15; fusion and ion drives 20+.\n\
- **Build cost** is the resource cost of building the spacecraft itself (engine and tank modules are paid for separately when configured).\n\
- **Built at** is where the craft is assembled: *Orbit* means it's built in an orbital shipyard and never lands; *Surface* means it's built on a planet's surface (some surface craft are full SSTOs, some are upper stages or ride a [launch vehicle](../launch-vehicles/) — the player picks which LV to pair with the craft at flight-planning time, so no fixed mapping is listed here).\n\
- **Requires LV** says when the craft must ride an LV to reach orbit: *Earth only* means it can self-launch from Luna, Mars, and asteroids but still needs an LV from Earth's gravity well; *Any body* means it needs an LV from every planet/moon (most early chemical and electric craft); *Built in orbit* means it never sits on a surface so the column doesn't apply. Earth always forces an LV regardless of the underlying flag.\n\n\
## See also\n\n\
- [Launch Vehicles](../launch-vehicles/) — surface-to-orbit lifters\n\
- [Research](../research/) — propulsion tech tree\n",
    );
    out
}


/// Format the per-rocket gravity gate as a player-facing cell.
///   * `None` → `Any` (no restriction).
///   * `Some` with `min_g == 0` → `≤ {max} G` (single ceiling — the common shape).
///   * `Some` with `min_g > 0`  → `{min} – {max} G` (defensive: not seen in the
///     shipped dump but the parser supports it via gate intersection).
/// Numeric formatting trims trailing zeros via `fmt_amount`, which matches the
/// payload / time cells one column over.
fn fmt_max_g(gate: Option<&GravityGateStat>) -> String {
    match gate {
        None => "Any".to_string(),
        Some(g) if g.min_g <= 0.0 => format!("≤ {} G", fmt_amount(g.max_g)),
        Some(g) => format!("{} – {} G", fmt_amount(g.min_g), fmt_amount(g.max_g)),
    }
}

fn page_launch_vehicles(locale: &Locale, sirenix: &Sirenix) -> String {
    let id_to_name: BTreeMap<&str, &str> = locale
        .launch_vehicles
        .iter()
        .map(|x| (x.id.as_str(), x.name.as_str()))
        .collect();
    let id_to_desc: BTreeMap<&str, &str> = locale
        .launch_vehicles
        .iter()
        .map(|x| (x.id.as_str(), x.description.as_str()))
        .collect();
    let resource_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    let mut entries: Vec<&LaunchVehicleStat> = sirenix
        .launch_vehicles
        .iter()
        .filter(|lv| id_to_name.get(lv.id.as_str()).map_or(false, |n| !n.is_empty()))
        .collect();
    entries.sort_by(|a, b| {
        a.max_payload
            .partial_cmp(&b.max_payload)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });

    // Categorise by propulsion.  Nuclear-thermal rockets use hydrogen as
    // reaction mass (`id_resource_hydrogen`); everything else is chemical.
    // Some chemical entries have `fuelTypeOnStart=None` in the dump (the
    // field is set at runtime), so chemical is the default bucket.
    let is_nuclear = |lv: &&LaunchVehicleStat| {
        lv.fuel_type_on_start.as_deref() == Some("id_resource_hydrogen")
    };
    let chemical: Vec<&LaunchVehicleStat> = entries.iter().copied().filter(|lv| !is_nuclear(lv)).collect();
    let nuclear: Vec<&LaunchVehicleStat> = entries.iter().copied().filter(is_nuclear).collect();

    let make_row = |lv: &LaunchVehicleStat| -> Vec<String> {
        let display = id_to_name.get(lv.id.as_str()).copied().unwrap_or(lv.id.as_str());
        let desc = id_to_desc.get(lv.id.as_str()).copied().unwrap_or("");
        vec![
            format!(
                "{anchor}**{name}**",
                anchor = anchor_tag("lv", &lv.id),
                name = escape_cell(display)
            ),
            fmt_amount(lv.max_payload),
            fmt_reusability(lv.reusability).into(),
            if lv.can_send_human { "Yes" } else { "No" }.into(),
            fmt_max_g(lv.gravity_gate.as_ref()),
            fmt_build_cost(&lv.build_cost, &resource_name),
            fmt_amount(lv.build_time_days),
            fmt_abbrev(lv.launch_cost),
            fmt_abbrev(lv.maintenance_cost_per_day),
            escape_cell(desc),
        ]
    };
    let headers = [
        "Launch Vehicle",
        "Payload (t)",
        "Reusable",
        "Crew",
        "Max G",
        "Build cost",
        "Time (d)",
        "Launch",
        "Maint",
        "Description",
    ];
    let tooltips = [
        None,
        Some("Max payload to low orbit, in tonnes"),
        Some("Survives reentry and can fly again (Yes / Partial / No)"),
        Some("Crew-rated for human passengers"),
        Some("Maximum surface gravity this rocket can launch from. Earth ≈ 1 G, Mars ≈ 0.38 G, Luna ≈ 0.16 G, Jupiter ≈ 2.5 G."),
        Some("Resources required to construct"),
        Some("Build time in days"),
        Some("Cash fee paid on every launch"),
        Some("Daily maintenance cost while idle on the pad"),
        None,
    ];

    let chem_rows: Vec<Vec<String>> = chemical.iter().map(|lv| make_row(lv)).collect();
    let nuke_rows: Vec<Vec<String>> = nuclear.iter().map(|lv| make_row(lv)).collect();
    let chem_table = md_table_with_tips(&headers, &tooltips, &chem_rows);
    let nuke_table = md_table_with_tips(&headers, &tooltips, &nuke_rows);

    // Alternative-launch-methods table is generated from `sirenix.facilities`
    // filtered to LaunchFacility so the rows stay in sync with the facilities
    // page.  We duplicate the view here (rather than just linking) so the
    // launch-vehicles page is self-contained for players comparing rockets
    // against non-rocket lifters.
    let facility_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let facility_desc: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.description.as_str()))
        .collect();
    let research_name: BTreeMap<&str, &str> = locale
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    let facility_unlocked_by: BTreeMap<&str, &str> = sirenix
        .research
        .iter()
        .filter(|r| r.action == "UnlockFacility")
        .filter_map(|r| r.unlock_target.as_deref().map(|t| (t, r.id.as_str())))
        .collect();

    let mut alt_methods: Vec<&FacilityStat> = sirenix
        .facilities
        .iter()
        .filter(|f| f.facility_type == "LaunchFacility")
        .filter(|f| !f.is_obsolete)
        .collect();
    // Sort by total build-cost amount ascending so the cheapest pad leads.
    let cost_total = |f: &&FacilityStat| -> f64 {
        f.build_cost.iter().map(|c| c.amount).sum::<f64>()
    };
    alt_methods.sort_by(|a, b| {
        cost_total(a)
            .partial_cmp(&cost_total(b))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });

    let alt_row = |f: &FacilityStat| -> Vec<String> {
        let id_no_prefix = f.id.strip_prefix("build_").unwrap_or(&f.id);
        let raw_display = facility_name.get(id_no_prefix).copied().unwrap_or(id_no_prefix);
        let display = smart_title_case(raw_display);
        let name_cell = format!(
            "{anchor}**{link}**",
            anchor = anchor_tag("lv-launch", id_no_prefix),
            link = link_cross_page("facilities", "facility", id_no_prefix, &escape_cell(&display)),
        );
        let time = if f.build_time_days > 0.0 {
            fmt_amount(f.build_time_days)
        } else {
            "—".to_string()
        };
        let workers = if f.workers_required > 0 {
            f.workers_required.to_string()
        } else {
            "—".to_string()
        };
        let energy = if f.energy_consumption > 0.0 {
            fmt_amount(f.energy_consumption)
        } else {
            "—".to_string()
        };
        let maint = if f.maintenance_per_day > 0.0 {
            fmt_abbrev(f.maintenance_per_day)
        } else {
            "—".to_string()
        };
        let prereq_id = facility_unlocked_by
            .get(f.id.as_str())
            .copied()
            .or_else(|| f.research_prereq.as_deref());
        let prereq = prereq_id
            .map(|r| {
                let name = research_name.get(r).copied().unwrap_or(r).to_string();
                link_cross_page("research", "research", r, &escape_cell(&name))
            })
            .unwrap_or_else(|| "—".to_string());
        let desc = facility_desc.get(id_no_prefix).copied().unwrap_or("");
        vec![
            name_cell,
            fmt_build_cost(&f.build_cost, &resource_name),
            time,
            workers,
            energy,
            maint,
            fmt_launch_bonus(f.bonus_data.as_ref()),
            prereq,
            escape_cell(desc),
        ]
    };

    let alt_headers = [
        "Method",
        "Build cost",
        "Time",
        "Workers",
        "Energy",
        "Maint",
        "Launch bonus",
        "Prereq",
        "Description",
    ];
    let alt_tips: [Option<&str>; 9] = [
        None,
        Some("Resources required to construct"),
        Some("Build time in days"),
        Some("On-site population required for full output"),
        Some("Energy consumed per day"),
        Some("Daily maintenance cost"),
        Some("Discount or capacity gain applied to launches that originate here"),
        Some("Research that unlocks this facility"),
        None,
    ];
    let alt_rows: Vec<Vec<String>> = alt_methods.iter().map(|f| alt_row(f)).collect();
    let alt_table = md_table_with_tips(&alt_headers, &alt_tips, &alt_rows);

    format!(
        "# Launch Vehicles\n\n\
Surface-to-orbit lifters. Every spacecraft that's built on a planet's surface\n\
has to ride one of these to reach orbit, and the launch cost paid here is paid\n\
on **every** launch — reusable vehicles amortise their build cost over many\n\
flights.\n\n\
Three propulsion families are unlocked across the tech tree:\n\n\
- **Chemical** rockets — kerosene/RP-1 burned with LOX. The early- and mid-game default.\n\
- **Nuclear-thermal** rockets — hydrogen heated by a fission reactor and expelled as reaction mass. Higher specific impulse for the same payload class; unlocked later in the tech tree.\n\
- **Mechanical / magnetic** launchers — non-rocket systems built as facilities. See [Alternative launch methods](#alternative-launch-methods) below.\n\n\
## Chemical rockets\n\n\
{chem_table}\n\
## Nuclear-thermal rockets\n\n\
{nuke_table}\n\
## Reading the tables\n\n\
- **Max Payload** is the heaviest load (in tonnes) the vehicle can carry to low orbit.\n\
- **Reusable** — *Yes* means the vehicle survives reentry and can fly again; *No* means each launch consumes the vehicle.\n\
- **Crew Rated** — whether the vehicle can carry humans, not just cargo.\n\
- **Max G** is the surface-gravity envelope the rocket can launch from. `Any` means no restriction; `≤ 1.8 G` means the rocket only ignites on bodies with surface gravity at or below 1.8 g (Earth ≈ 1 G, Mars ≈ 0.38 G, Luna ≈ 0.16 G, Jupiter ≈ 2.5 G). Only the Al-Ice rockets carry a gate in the shipped data — everything else launches from anywhere.\n\
- **Launch cost** is the cash fee paid every launch; **Maintenance** is the daily upkeep cost while idle on the pad.\n\n\
## Alternative launch methods\n\n\
The game also models several non-rocket launch systems unlocked through\n\
research and built as facilities at the launch site. Each row links to the\n\
matching entry on the [Facilities](../facilities/) page.\n\n\
{alt_table}\n\
## See also\n\n\
- [Spacecraft](../spacecraft/)\n\
- [Research](../research/) — Launch Vehicles tech category\n"
    )
}

// DifficultyConfig multipliers — sourced from
// `Assets/MonoBehaviour/DifficultyConfig.asset` in the AssetRipper export.
// The asset stores three (EGameDifficulty → BaseMultipliers) entries:
//
//   key 0 → startMoney 1.25, upkeep 0.5, supplyUsage 0.5  (Explorer)
//   key 1 → startMoney 1.0,  upkeep 1.0, supplyUsage 1.0  (Pioneer)
//   key 2 → startMoney 0.75, upkeep 1.5, supplyUsage 1.5  (Veteran)
//
// The base `money` value baked into each StartGameData save is the Pioneer
// (key 1) figure; we multiply it for the other two tiers.
const DIFFICULTY_NAMES: &[&str] = &["Explorer", "Pioneer", "Veteran"];
const DIFFICULTY_MONEY_MULT: &[f64] = &[1.25, 1.0, 0.75];
const DIFFICULTY_UPKEEP_MULT: &[f64] = &[0.5, 1.0, 1.5];
const DIFFICULTY_SUPPLY_MULT: &[f64] = &[0.5, 1.0, 1.5];

/// Map a `StartGameEpoch_*` id to the human-facing name shown on the New
/// Game customization screen.  No locale entry exists for these in the
/// shipped strings; the names mirror the in-game UI labels.
fn epoch_display_name(epoch_id: &str) -> &'static str {
    match epoch_id {
        "StartGameEpoch_Prelude" => "Prelude",
        "StartGameEpoch_EarlyExploration" => "Early Exploration",
        "StartGameEpoch_Colonization" => "Colonization Era",
        "StartGameEpoch_TheExpansion" => "The Expansion",
        "StartGameEpoch_RaceBeyond" => "Race Beyond",
        _ => "Unknown",
    }
}

/// Pull the four-digit year out of an epoch's `startDateString`, which is
/// formatted as `DD.MM.YYYY HH:MM:SS` in the Sirenix dump (note: dots, not
/// slashes — different from the contract `dateStartActive` format).
fn extract_epoch_year(s: &str) -> Option<String> {
    // Two acceptable input shapes from the dump: `01.01.2100 00:00:00` and
    // a bare `01.01.2100`. The first token (date) splits on '.' into
    // [day, month, year].
    let date = s.split_whitespace().next()?;
    let parts: Vec<&str> = date.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[2];
    if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
        Some(year.to_string())
    } else {
        None
    }
}

fn page_corporations(locale: &Locale, sirenix: &Sirenix) -> String {
    // Build research id → display name lookup from locale (e.g.
    // "research_nukeprop_2" → "Solid-core nuclear-thermal engines").
    let research_name: BTreeMap<&str, &str> = locale
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    let mut out = String::from(
        "# Corporations\n\n\
The five playable starting factions in Solar Expanse. Each scenario\n\
(Early Exploration / The Expansion / Colonization Era / Race Beyond)\n\
ships a different pre-built save where every corporation starts with\n\
its own completed research, funding, and fleet. Difficulty further\n\
scales starting money and ongoing costs.\n\n",
    );

    // Build the "Scenarios" reference table — emitted later in the page,
    // after the Corporations-at-a-glance block.  Word choice note: the
    // dropdown above is labelled "Scenario", so the rest of the page uses
    // "Scenario" too (rather than the internal `StartGameEpoch` term).
    let scenarios_section: String = if sirenix.epochs.is_empty() {
        String::new()
    } else {
        let playable_epochs: std::collections::BTreeSet<&str> = sirenix
            .scenario_starts
            .iter()
            .map(|s| s.scenario_id.as_str())
            .collect();
        let epoch_rows: Vec<Vec<String>> = sirenix
            .epochs
            .iter()
            .filter(|e| playable_epochs.contains(e.id.as_str()))
            .map(|e| {
                let name = epoch_display_name(&e.id);
                let mut corps = e.possible_player_companies.clone();
                corps.sort();
                corps.dedup();
                let corp_cell = if corps.is_empty() {
                    "—".to_string()
                } else {
                    corps.join(", ")
                };
                vec![
                    format!("**{}**", name),
                    escape_cell(&corp_cell),
                ]
            })
            .collect();
        let mut section = String::new();
        section.push_str("## Scenarios\n\n");
        section.push_str(
            "Solar Expanse's New Game menu offers four start scenarios in Sol-system play —\n\
Early Exploration, The Expansion, Colonization Era, and Race Beyond — each\n\
with its own roster of playable corporations, all driving the comparison\n\
table above.\n\n\
*The shipped data files carry start-year values that don't match what the\n\
game UI currently shows (start years drift with patches), so they're not\n\
in this table. The names and corp rosters below are stable.*\n\n",
        );
        section.push_str(&md_table(
            &["Scenario", "Playable corporations"],
            &epoch_rows,
        ));
        section.push('\n');
        section
    };

    // Difficulty also modifies upkeep and supply usage, which the
    // comparison table doesn't surface — keep that as a one-line note so
    // the player knows there's more to the difficulty choice than just
    // starting cash.
    out.push_str("Difficulty also scales ongoing upkeep (Explorer ×0.5, Pioneer ×1, Veteran ×1.5) and supply usage by the same factors — not reflected in the table below, which only shows starting state.\n\n");

    // ── Build the JSON blob that powers the interactive comparison table. ──
    // Schema:
    //   { scenarios: [{ id, name, corps: [{ name, starting_money,
    //                                        lv_count, sc_count,
    //                                        research: [{name, category}…] }] }],
    //     difficulties: [{ name, money_multiplier }] }
    //
    // Each research entry is an object so the JS renderer can group rows in
    // the comparison table by sub-branch (the player-facing tech-tree branch
    // labels). `category` is the humanized `researchSubType.name`
    // (`SubBranch_` already stripped on the parser side); items with no
    // matching ResearchStat in the dump fall back to "Other".
    //
    // Corp order inside each scenario matches the locale order
    // (SoleX, NASA, ESA, CNSA, Roscosmos) so the comparison columns line
    // up with the in-game customization screen.
    // Map research id → humanized sub-branch label.  The Sirenix dump
    // stores camelCase sub-branch ids (`LaunchVehicle`, `LifeSupport`,
    // `Chemical`); the tech-tree UI shows the same ids with spaces between
    // words and every word title-cased — so `LaunchVehicle` → `Launch
    // Vehicle`, not `Launch vehicle`.  Single-word ids (`Chemical`,
    // `Spacecraft`) pass through unchanged.
    let humanize_subbranch = |sb: &str| -> String {
        let mut out = String::with_capacity(sb.len() + 4);
        for (i, c) in sb.char_indices() {
            if i > 0 && c.is_uppercase() {
                out.push(' ');
            }
            out.push(c);
        }
        out
    };
    let research_subbranch: BTreeMap<&str, &str> = sirenix
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.subbranch.as_str()))
        .collect();
    // Map facility id (locale keys it WITHOUT the `build_` prefix) → display
    // name.  Display names from locale arrive UPPERCASE (e.g. "NOBLE GAS
    // MINE"); smart_title_case turns them into "Noble Gas Mine" before they
    // reach the JS layer.
    let facility_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let mut scenarios_json: Vec<serde_json::Value> = Vec::new();
    for s in &sirenix.scenario_starts {
        let mut corps_json: Vec<serde_json::Value> = Vec::new();
        for c in &locale.corporations {
            let cs = match s
                .corp_starts
                .iter()
                .find(|cs| cs.company_id.eq_ignore_ascii_case(&c.name))
            {
                Some(cs) => cs,
                None => continue,
            };
            // Build (name, category) tuples, dedup by name, then sort by
            // category-then-name so the JS layer can group adjacent rows
            // without re-sorting.
            let mut entries: Vec<(String, String)> = cs
                .completed_research
                .iter()
                .filter_map(|rid| {
                    // Drop tree-structure category nodes — they aren't
                    // player-visible research and were already filtered
                    // out by the previous version of this page.
                    if rid.starts_with("research_category_") {
                        return None;
                    }
                    let nm = research_name
                        .get(rid.as_str())
                        .copied()
                        .unwrap_or(rid.as_str())
                        .to_string();
                    let cat = research_subbranch
                        .get(rid.as_str())
                        .map(|sb| humanize_subbranch(sb))
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "Other".to_string());
                    Some((nm, cat))
                })
                .collect();
            entries.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
            entries.dedup_by(|a, b| a.0 == b.0);
            let research_json: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|(name, category)| serde_json::json!({
                    "name": name,
                    "category": category,
                }))
                .collect();
            // Resolve each `build_*` id to its display name via locale; fall
            // back to the prefix-stripped raw id (smart-title-cased) when the
            // locale has no entry.  `(id, count)` pairs arrive sorted by id;
            // we re-sort by display name alphabetically so the JS layer can
            // emit rows in user-friendly order.
            let mut facility_entries: Vec<(String, u32)> = cs
                .starting_facilities
                .iter()
                .map(|(fid, count)| {
                    let id_no_prefix = fid.strip_prefix("build_").unwrap_or(fid.as_str());
                    let raw_display = facility_name
                        .get(id_no_prefix)
                        .copied()
                        .unwrap_or(id_no_prefix);
                    (smart_title_case(raw_display), *count)
                })
                .collect();
            facility_entries.sort_by(|a, b| a.0.cmp(&b.0));
            let facilities_json: Vec<serde_json::Value> = facility_entries
                .into_iter()
                .map(|(name, count)| serde_json::json!({
                    "name": name,
                    "count": count,
                }))
                .collect();
            corps_json.push(serde_json::json!({
                "name": c.name,
                "starting_money": cs.starting_money,
                "lv_count": cs.starting_launch_vehicles,
                "sc_count": cs.starting_spacecraft,
                "research": research_json,
                "starting_facilities": facilities_json,
            }));
        }
        scenarios_json.push(serde_json::json!({
            "id": s.scenario_id,
            "name": epoch_display_name(&s.scenario_id),
            "corps": corps_json,
        }));
    }

    let difficulties_json: Vec<serde_json::Value> = (0..DIFFICULTY_NAMES.len())
        .map(|i| {
            serde_json::json!({
                "name": DIFFICULTY_NAMES[i],
                "money_multiplier": DIFFICULTY_MONEY_MULT[i],
            })
        })
        .collect();

    let corp_data_json = serde_json::to_string(&serde_json::json!({
        "scenarios": scenarios_json,
        "difficulties": difficulties_json,
    }))
    .expect("CORP_DATA serialization");

    out.push_str("## Comparison\n\n");
    out.push_str(
        "Pick a scenario and difficulty to compare starting funding, fleet, and completed research across all five corporations side-by-side. Only research held by at least one corporation at the selected scenario is listed.\n\n",
    );
    // <select> elements use The Expansion + Pioneer as defaults; the JS
    // layer reads the current value and re-renders the table on change.
    // Early Exploration starts every corp with zero pre-built facilities,
    // so the comparison's Starting facilities section comes up empty —
    // a poor landing view. The Expansion is the first scenario with a
    // non-trivial facility delta between corps.
    out.push_str("<div class=\"calc\">\n");
    out.push_str("<label>Scenario:\n<select id=\"corp-scenario\">\n");
    for s in &sirenix.scenario_starts {
        let id = &s.scenario_id;
        let name = epoch_display_name(id);
        let selected = if id == "StartGameEpoch_TheExpansion" {
            " selected"
        } else {
            ""
        };
        out.push_str(&format!(
            "<option value=\"{id}\"{selected}>{name}</option>\n"
        ));
    }
    out.push_str("</select>\n</label>\n");
    out.push_str("<label>Difficulty:\n<select id=\"corp-difficulty\">\n");
    for n in DIFFICULTY_NAMES {
        let selected = if *n == "Pioneer" { " selected" } else { "" };
        out.push_str(&format!("<option value=\"{n}\"{selected}>{n}</option>\n"));
    }
    out.push_str("</select>\n</label>\n");
    out.push_str("<div id=\"corp-comparison\"></div>\n");
    out.push_str("</div>\n\n");

    out.push_str(&format!("<script>\nwindow.CORP_DATA = {corp_data_json};\n</script>\n"));
    out.push_str("<script src=\"{{ '/assets/js/corporations.js' | relative_url }}?v={{ site.data.wiki.generated_at }}\"></script>\n\n");

    // ── Flavor-traits block per corp.  Moved below the comparison so the
    //    above-the-fold view is the player-actionable interactive matrix;
    //    the descriptive prose follows for readers who want context.
    out.push_str("## Corporations at a glance\n\n");
    for c in &locale.corporations {
        out.push_str(&format!("### {}\n\n{}\n\n", c.name, c.description));
        let traits = c.traits.replace("\\n", "\n");
        let traits = traits.trim();
        if !traits.is_empty() {
            out.push_str("**Flavor traits (from new-game screen):**\n");
            out.push_str(traits);
            out.push_str("\n\n");
        }
    }

    // Scenarios reference table — comes after the per-corp flavor section
    // so the interactive Comparison + Corporations-at-a-glance content
    // appears first, with the scenario lookup tucked underneath.
    out.push_str(&scenarios_section);

    out.push_str("## See also\n\n- [Research](../research/) — full tech tree across all branches\n");
    out
}

fn page_resources(locale: &Locale, sirenix: &Sirenix) -> String {
    let res_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    let res_desc: BTreeMap<&str, String> = locale
        .resources
        .iter()
        .filter_map(|r| {
            let desc_id = format!("{}_Description", r.id);
            locale
                .resources
                .iter()
                .find(|d| d.id == desc_id)
                .map(|d| (r.id.clone(), d.name.clone()))
        })
        .map(|(k, v)| {
            let leak: &'static str = Box::leak(k.into_boxed_str());
            (leak, v)
        })
        .collect();
    // Map facility id → human name to surface where each resource is produced
    // or consumed.
    let fac_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();

    // Resolve a sirenix FacilityStat id (which carries the `build_` prefix used
    // throughout the dump) to the `(anchor_id, pretty_display)` pair the
    // resources page should link with. The locale keys facilities without the
    // prefix, and the facilities page anchors are built the same way, so we
    // strip the prefix once here and reuse the result for both link sides.
    let resolve_facility = |fid: &str| -> (String, String) {
        let id_no_prefix = fid.strip_prefix("build_").unwrap_or(fid);
        let display = fac_name
            .get(id_no_prefix)
            .copied()
            .unwrap_or(id_no_prefix);
        (id_no_prefix.to_string(), smart_title_case(display))
    };

    // Look up facilities whose structured `produces` data lists `resource_id`.
    // Walking `Facility.produces` (rather than tooltip substring matching) is
    // the only way to avoid false positives like "Exotic Alloy Production" —
    // whose description mentions metal and fissiles as *inputs* — being mis-
    // labelled as a producer of those resources.
    // Returns (anchor_id, pretty_display) pairs so the caller can emit
    // cross-page links into the facilities page.
    let producers_for_resource =
        |resource_id: &str, facilities: &[FacilityStat]| -> Vec<(String, String)> {
            let mut hits: Vec<(String, String)> = facilities
                .iter()
                .filter(|f| f.produces.iter().any(|rc| rc.resource_id == resource_id))
                .map(|f| resolve_facility(&f.id))
                .collect();
            hits.sort_by(|a, b| a.1.cmp(&b.1));
            hits.dedup_by(|a, b| a.1 == b.1);
            hits
        };

    // Mirror of `producers_for_resource` for the structured `consumes` field.
    let consumers_for_resource =
        |resource_id: &str, facilities: &[FacilityStat]| -> Vec<(String, String)> {
            let mut hits: Vec<(String, String)> = facilities
                .iter()
                .filter(|f| f.consumes.iter().any(|rc| rc.resource_id == resource_id))
                .map(|f| resolve_facility(&f.id))
                .collect();
            hits.sort_by(|a, b| a.1.cmp(&b.1));
            hits.dedup_by(|a, b| a.1 == b.1);
            hits
        };

    let mut entries: Vec<&ResourceStat> = sirenix
        .resources
        .iter()
        .filter(|r| r.show_on_ui)
        .collect();
    entries.sort_by(|a, b| {
        // Sort by resource_type so Energy / Human cluster separately, then alphabetic.
        a.resource_type.cmp(&b.resource_type).then_with(|| {
            res_name
                .get(a.id.as_str())
                .copied()
                .unwrap_or(a.id.as_str())
                .cmp(res_name.get(b.id.as_str()).copied().unwrap_or(b.id.as_str()))
        })
    });

    // Earth's per-resource license fees, sourced from the BepInEx mod's
    // runtime ObjectInfo walk. Earth is currently the only body that
    // charges, so we only need that one entry. The mod records each
    // ObjectInfo's `gameObject.name`, which in the live scene happens to
    // be "View" for the body that carries Earth's fees (verified empirically
    // — out of 150 ObjectInfo MonoBehaviours dumped, exactly one has any
    // entries in `resourceMiningLicenseFeePerT`). Match by literal name
    // "Earth" first; fall back to the single body with non-empty fees if
    // the literal match misses. If none of the dump's bodies carry fees
    // (older dump from before the mod rebuild), the column falls back to
    // em-dashes everywhere.
    let earth_fees: Option<&BTreeMap<String, f64>> = sirenix
        .license_fees
        .iter()
        .find(|b| b.body_name == "Earth")
        .or_else(|| {
            sirenix
                .license_fees
                .iter()
                .find(|b| !b.fees_per_t.is_empty())
        })
        .map(|b| &b.fees_per_t);

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|r| {
            let display = res_name.get(r.id.as_str()).copied().unwrap_or(r.id.as_str());
            // License (Earth) — em-dash for resources Earth doesn't charge
            // for, AND for every row when no Earth entry is in the dump.
            // Zero-fee entries are pre-stripped by `parse_body_license_fee`,
            // so a present value here is always a real charge.
            let license_cell = earth_fees
                .and_then(|m| m.get(&r.id))
                .map(|f| fmt_abbrev(*f))
                .unwrap_or_else(|| "—".to_string());
            let price = if r.market_price_base > 0.0 {
                fmt_abbrev(r.market_price_base)
            } else {
                "—".to_string()
            };
            let producers = producers_for_resource(&r.id, &sirenix.facilities);
            let prod_cell = if producers.is_empty() {
                "—".to_string()
            } else {
                producers
                    .iter()
                    .map(|(fid, name)| {
                        link_cross_page("facilities", "facility", fid, &escape_cell(name))
                    })
                    .collect::<Vec<_>>()
                    .join("<br>")
            };
            let consumers = consumers_for_resource(&r.id, &sirenix.facilities);
            let cons_cell = if consumers.is_empty() {
                "—".to_string()
            } else {
                consumers
                    .iter()
                    .map(|(fid, name)| {
                        link_cross_page("facilities", "facility", fid, &escape_cell(name))
                    })
                    .collect::<Vec<_>>()
                    .join("<br>")
            };
            let desc = res_desc
                .get(r.id.as_str())
                .map(|s| s.as_str())
                .unwrap_or("");
            // Inline anchor so other pages (facilities, research) can deep-link to this row.
            let anchor = anchor_tag("resource", &r.id);
            // Icon path mirrors `fmt_build_cost`: filename is the resource id
            // (already stripped of the `id_resource_` prefix by parse_sirenix /
            // parse_locale). The icons live under /docs/images/resources/.
            let icon = format!(
                "<img src=\"../images/resources/{id}.png\" width=\"16\" alt=\"{label}\"/>",
                id = r.id,
                label = escape_cell(display),
            );
            vec![
                format!("{anchor}{icon}&nbsp;**{}**", escape_cell(display)),
                r.resource_type.clone(),
                license_cell,
                price,
                prod_cell,
                cons_cell,
                escape_cell(desc),
            ]
        })
        .collect();
    let table = md_table_with_tips(
        &[
            "Resource",
            "Type",
            "License (Earth, $/t)",
            "Market base ($/t)",
            "Producers",
            "Consumers",
            "Description",
        ],
        &[
            None,
            Some("Normal (physical), Energy (real-time power), or Human (colonists)"),
            Some("Earth licensing fee per tonne extracted. Other planets either don't charge or set their own rates; check in-game per-deposit tooltips for non-Earth values."),
            Some("Base clearing price used as the market price anchor — supply and demand push actual prices around it"),
            Some("Facilities whose structured production data lists this resource as an output"),
            Some("Facilities whose structured production data lists this resource as an input"),
            None,
        ],
        &rows,
    );
    format!(
        "# Resources\n\n\
Resources are produced by facilities, shipped between worlds in spacecraft\n\
cargo holds, traded on the marketplace, and consumed in construction. Three\n\
types exist:\n\n\
- **Normal** — physical materials, the bulk of the economy.\n\
- **Energy** — power; produced and consumed in real time, with limited storage in batteries.\n\
- **Human** — colonists; produced over time by habitats and consumed by jobs.\n\n\
{table}\n\
## Reading the table\n\n\
- **License (Earth, $/t)** is the per-tonne fee Earth charges for extracting each resource. Earth is currently the only body that charges; other planets either don't charge at all or set their own rates per deposit (check the in-game tooltip on each deposit for non-Earth values).\n\
- **Market base ($/t)** is the starting clearing-price anchor used by the global market; supply and demand move actual prices around it.\n\
- **Producers** and **Consumers** are pulled from each facility's structured production data (`refinerData`, `energyProductionData`, `resourcesToMine`, `byproducts`) — not from tooltip text — so refineries don't get mis-credited as producing their inputs. Per-day rates aren't extractable from the static descriptors; the in-game tooltip remains the source of truth for rate numbers.\n\n\
## See also\n\n\
- [Terraforming](../terraforming/) — per-resource thermal / phase constants (boiling and melting points, latent heat, heat capacity, optical depth) that drive the atmosphere sim.\n"
    )
}

/// Render the terraforming page: one row per resource with real thermal /
/// phase constants from `terraformationInfo`. Resources whose info is `None`
/// (the C# all-1.0 placeholder — energy, human, supplies, antimatter, …)
/// are skipped so the table only carries species that actually participate
/// in the atmosphere sim.
///
/// Sorting: alphabetically by player-facing display name (locale-resolved),
/// matching the convention used elsewhere on the wiki.
fn page_terraforming(locale: &Locale, sirenix: &Sirenix) -> String {
    let res_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    // Filter to resources that have real thermal physics. Drop everything
    // else (parse_sirenix already filters the all-1.0 placeholder default).
    let mut entries: Vec<(&ResourceStat, &TerraformationInfoStat)> = sirenix
        .resources
        .iter()
        .filter_map(|r| r.terraformation_info.as_ref().map(|ti| (r, ti)))
        .collect();
    entries.sort_by(|a, b| {
        let na = res_name.get(a.0.id.as_str()).copied().unwrap_or(a.0.id.as_str());
        let nb = res_name.get(b.0.id.as_str()).copied().unwrap_or(b.0.id.as_str());
        na.cmp(nb)
    });

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|(r, ti)| {
            let display = res_name.get(r.id.as_str()).copied().unwrap_or(r.id.as_str());
            let anchor = anchor_tag("terraforming", &r.id);
            // Icon matches the resources page convention.
            let icon = format!(
                "<img src=\"../images/resources/{id}.png\" width=\"16\" alt=\"{label}\"/>",
                id = r.id,
                label = escape_cell(display),
            );
            // Cross-link the name back to the resources page row.
            let name_link = link_cross_page("resources", "resource", &r.id, &escape_cell(display));
            vec![
                format!("{anchor}{icon}&nbsp;**{name_link}**"),
                fmt_phase_temperature(ti.melting_temperature_k),
                fmt_phase_temperature(ti.boiling_temperature_k),
                fmt_terraforming_number(ti.vaporization_latent_heat),
                fmt_terraforming_number(ti.heat_capacity),
                fmt_terraforming_number(ti.optical_depth_parameter),
                fmt_terraforming_number(ti.pressure_triple_point),
            ]
        })
        .collect();
    let table = md_table_with_tips(
        &[
            "Resource",
            "Melting (K / °C)",
            "Boiling (K / °C)",
            "Latent heat (J/mol)",
            "Heat capacity (J/(kg·K))",
            "Optical depth",
            "Triple-point pressure (atm)",
        ],
        &[
            None,
            Some("Phase-change temperature where the resource transitions between solid and liquid. The body's surface temperature must cross this for the resource to melt or freeze."),
            Some("Phase-change temperature where the resource transitions between liquid and gas at reference pressure. Crossing this is what gets a species into (or out of) the atmosphere."),
            Some("Energy required to vaporize one mole of the resource. Drives how strongly evaporation cools the surface and condensation warms it."),
            Some("Specific heat — how much energy the resource absorbs before warming. High values smooth out temperature swings."),
            Some("Greenhouse strength: dimensionless coefficient (formerly `gasIRAbsorbtionCoefficient`) that scales how much outgoing infrared the gas traps."),
            Some("Triple-point pressure: the minimum atmospheric pressure for a stable liquid phase. Below this, the resource sublimates directly between solid and gas."),
        ],
        &rows,
    );

    // ---- Terraforming-facilities table ------------------------------------
    // Surface the facilities that actively modify a body's habitability
    // parameters. Source of truth is `FacilityStat.habitability_deltas`; we
    // skip facilities whose every delta is zero (defensive — the dump
    // currently only carries non-zero entries, but the column is shared with
    // role magnitudes elsewhere so the guard keeps us honest).
    let facility_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let mut terra_facs: Vec<(&FacilityStat, String)> = sirenix
        .facilities
        .iter()
        .filter(|f| !f.is_obsolete)
        .filter(|f| f.habitability_deltas.iter().any(|(_, v)| *v != 0.0))
        .map(|f| {
            let id_no_prefix = f.id.strip_prefix("build_").unwrap_or(&f.id);
            let raw = facility_name
                .get(id_no_prefix)
                .copied()
                .unwrap_or(id_no_prefix);
            let display = smart_title_case(raw);
            (f, display)
        })
        .collect();
    terra_facs.sort_by(|a, b| a.1.cmp(&b.1));

    let facilities_section = if terra_facs.is_empty() {
        String::new()
    } else {
        let fac_rows: Vec<Vec<String>> = terra_facs
            .iter()
            .map(|(f, display)| {
                let id_no_prefix = f.id.strip_prefix("build_").unwrap_or(&f.id);
                let name_link = link_cross_page(
                    "facilities",
                    "facility",
                    id_no_prefix,
                    &escape_cell(display),
                );
                // One row per facility — each delta on its own line inside the
                // Effect cell so a multi-parameter facility (Radiation /
                // Magnetic field) reads cleanly without repeating the name.
                let effects = f
                    .habitability_deltas
                    .iter()
                    .filter(|(_, v)| *v != 0.0)
                    .map(|(label, value)| {
                        let sign = if *value < 0.0 { "−" } else { "+" };
                        format!("{label} {sign}{}", fmt_magnitude_abs(*value))
                    })
                    .collect::<Vec<_>>()
                    .join("<br>");
                vec![format!("**{name_link}**"), effects]
            })
            .collect();
        let fac_table = md_table(&["Facility", "Per-day habitability deltas"], &fac_rows);
        format!(
            "## Terraforming facilities\n\n\
Facilities that actively modify a planet's habitability parameters over time. \
See the [Facilities page](../facilities/) for build cost, prerequisites, and \
other stats.\n\n\
{fac_table}\n"
        )
    };

    format!(
        "# Terraforming\n\n\
Solar Expanse simulates planetary atmospheres and surface conditions based on\n\
per-resource thermal properties. Use these tables to understand:\n\n\
- which resources will vaporize or freeze at a given temperature\n\
- how heat capacity drives atmospheric warming and cooling\n\
- which resources contribute to greenhouse warming (optical depth) and surface heating\n\n\
## Resource thermal properties\n\n\
{table}\n\
{facilities_section}\
## Reading the table\n\n\
- **Melting / Boiling** are the phase-change temperatures the body's average surface temperature must cross to keep the resource solid, liquid, or gas at reference pressure. Both columns show kelvin first with the celsius equivalent in parentheses.\n\
- **Latent heat (J/mol)** is the energy required to vaporize one mole of the resource. It drives how strongly evaporation cools the planet's surface and how strongly condensation warms it — the same constant feeds the Clausius-Clapeyron formula the sim uses to compute saturation pressures from temperature.\n\
- **Heat capacity (J/(kg·K))** is how much energy the resource absorbs before its temperature rises. High values smooth out day/night and seasonal temperature swings, so atmospheres dominated by high-Cp species are stabler.\n\
- **Optical depth** is the dimensionless greenhouse contribution. Higher values trap more outgoing infrared radiation — atmospheres dominated by high-optical-depth species (CO2, water vapor) warm.\n\
- **Triple-point pressure (atm)** is the minimum atmospheric pressure at which a stable liquid phase exists. Below this, the resource sublimates directly between solid and gas (think Mars-pressure CO2 frost).\n\n\
## See also\n\n\
- [Resources](../resources/) — per-resource production / consumption, market prices, and Earth licensing fees.\n\
- [Facilities](../facilities/) — full table of buildings (including the terraforming structures surfaced above), with build costs and prerequisites.\n"
    )
}

/// Format a kelvin temperature for the terraforming table. Shows both
/// kelvin (rounded to the nearest whole degree, matching how the dump
/// stores almost every value as an integer kelvin reading) and the
/// celsius equivalent rounded to the nearest whole degree.
///
/// Water boils at 373 K → 100 °C (373 - 273.15 = 99.85, rounded);
/// CO2 sublimates at 217 K → -56 °C (216.85 rounded).
fn fmt_phase_temperature(k: f64) -> String {
    if !k.is_finite() || k <= 0.0 {
        return "—".to_string();
    }
    let c = k - 273.15;
    let k_str = if (k - k.round()).abs() < 0.05 {
        format!("{:.0}", k)
    } else {
        format!("{:.1}", k)
    };
    // Round celsius to the nearest whole degree for table readability —
    // matches the way real-world phase-change tables print values.
    let c_str = format!("{:.0}", c.round());
    format!("{k_str} / {c_str} °C")
}

/// Compact f64 → string for the thermal-constant cells: integers print
/// without trailing zeros, sub-integer values use up to four significant
/// digits, and very small numbers fall back to scientific notation so
/// optical-depth values like `1e-6` don't render as `0`.
fn fmt_terraforming_number(v: f64) -> String {
    if !v.is_finite() {
        return "—".to_string();
    }
    if v == 0.0 {
        return "0".to_string();
    }
    if v.abs() >= 1.0 && (v - v.round()).abs() < 0.05 {
        return format!("{:.0}", v.round());
    }
    if v.abs() >= 1.0 {
        return format!("{:.2}", v);
    }
    // Sub-1 values: stretch to 4 significant decimal places where useful,
    // but switch to scientific notation when the magnitude is tiny.
    if v.abs() < 0.0001 {
        return format!("{:.1e}", v);
    }
    // For values like 0.0695, 0.00611, 0.002 keep 5 fractional digits then
    // trim trailing zeros so 0.00200 → 0.002.
    let s = format!("{:.5}", v);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

/// Truncate `s` to at most `max_chars` characters, ending at a word boundary
/// and balancing a stray closing `"` if one would be left dangling.
fn truncate_premise(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let cut = s
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let mut head = &s[..cut];
    if let Some(sp) = head.rfind(|c: char| c.is_whitespace()) {
        head = &head[..sp];
    }
    let mut head_owned: String = head.trim_end().to_string();
    while let Some(c) = head_owned.chars().last() {
        if matches!(c, ',' | ';' | ':' | '-' | '—') {
            head_owned.pop();
            head_owned.truncate(head_owned.trim_end().len());
        } else {
            break;
        }
    }
    if head_owned.chars().filter(|c| *c == '"').count() % 2 == 1 {
        if let Some(idx) = head_owned.rfind('"') {
            head_owned.truncate(idx);
            head_owned.truncate(head_owned.trim_end().len());
            while let Some(c) = head_owned.chars().last() {
                if matches!(c, ',' | ';' | ':' | '-' | '—') {
                    head_owned.pop();
                    head_owned.truncate(head_owned.trim_end().len());
                } else {
                    break;
                }
            }
        }
    }
    format!("{}…", head_owned)
}

/// Resolve a contract-objective `target` id to a human-readable label.
fn resolve_objective_target(
    target: &str,
    fac_name: &BTreeMap<&str, &str>,
    resource_name: &BTreeMap<&str, &str>,
    sc_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
    research_name: &BTreeMap<&str, &str>,
) -> String {
    if let Some(rest) = target.strip_prefix("build_") {
        return smart_title_case(fac_name.get(rest).copied().unwrap_or(rest));
    }
    if let Some(rest) = target.strip_prefix("id_resource_") {
        return resource_name.get(rest).copied().unwrap_or(rest).to_string();
    }
    if let Some(rest) = target.strip_prefix("module_") {
        // Force title-case here — `smart_title_case` is a no-op on strings that
        // already contain lowercase letters.
        return title_case_words(&rest.replace('_', " "));
    }
    if target.starts_with("research_") {
        if let Some(nm) = research_name.get(target).copied() {
            return nm.to_string();
        }
        return smart_title_case(&target.trim_start_matches("research_").replace('_', " "));
    }
    // `Spacecraft<N><CodeName>` — sirenix-only objective targets that don't
    // appear in the locale spacecraft list.  Try `research_sc_<lower>` first.
    if let Some(rest) = target.strip_prefix("Spacecraft") {
        let codename: String = rest.chars().skip_while(|c| c.is_ascii_digit()).collect();
        if !codename.is_empty() {
            let key = format!("research_sc_{}", codename.to_lowercase());
            if let Some(nm) = research_name.get(key.as_str()).copied() {
                return nm.to_string();
            }
            return codename;
        }
    }
    if let Some(nm) = sc_name.get(target).copied() {
        return nm.to_string();
    }
    if let Some(nm) = lv_name.get(target).copied() {
        return nm.to_string();
    }
    // Launch-vehicle id that didn't match locale (e.g. `lv_chem_superlarge`):
    // produce a humanized label rather than leaking the raw id.
    if let Some(rest) = target.strip_prefix("lv_") {
        let class = if rest.starts_with("chem") {
            "Chemical"
        } else if rest.starts_with("nuke") {
            "Nuclear"
        } else {
            ""
        };
        let pretty: Vec<String> = rest
            .split('_')
            .filter(|p| !p.is_empty() && *p != "chem" && *p != "nuke")
            .map(|p| {
                let mut chars = p.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect();
        let modifier = pretty.join(" ");
        return if class.is_empty() && modifier.is_empty() {
            "Launch Vehicle".into()
        } else if class.is_empty() {
            format!("{modifier} Launch Vehicle")
        } else if modifier.is_empty() {
            format!("{class} Launch Vehicle")
        } else {
            format!("{class} Launch Vehicle ({modifier})")
        };
    }
    smart_title_case(&target.replace('_', " "))
}

/// Format a single contract objective into a Requirements-column bullet.
fn format_objective(
    o: &ContractObjectiveStat,
    fac_name: &BTreeMap<&str, &str>,
    resource_name: &BTreeMap<&str, &str>,
    sc_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
    research_name: &BTreeMap<&str, &str>,
) -> String {
    let target = o.target.as_deref().map(|t| {
        resolve_objective_target(t, fac_name, resource_name, sc_name, lv_name, research_name)
    });
    // Some objective kinds carry quantity 0 in the source data (the engine
    // treats "any amount" as 0).  Normalize to at least 1 so the rendered
    // table doesn't show nonsensical "Build 0× X" / "Have 0× Y" lines.
    let qty = if o.quantity <= 0.0 { 1.0 } else { o.quantity };
    match (o.kind.as_str(), target.as_deref()) {
        ("BuildFacility", Some(t)) => format!("Build {}× {}", fmt_amount(qty), t),
        ("Possession", Some(t)) => format!("Have {}× {}", fmt_amount(qty), t),
        // Fleet Expansion: "Possess 10" with no target means 10 spacecraft.
        ("Possession", None) => format!("Have {}× Spacecraft", fmt_amount(qty)),
        ("MarketsOffers", Some(t)) | ("MarketPlaceOffers", Some(t)) => {
            format!("Market trade {}× {}", fmt_amount(qty), t)
        }
        ("ChangeHabitabilityParameters", _) => "Adjust habitability parameter".into(),
        ("ChangeDepositParameters", Some(t)) | ("ChangeDeposit", Some(t)) => {
            format!("Survey {} deposit", t)
        }
        ("Exploration", _) | ("ExplorationObject", _) => "Explore".into(),
        ("ExplorationInterstellar", _) => "Explore interstellar".into(),
        ("MakeResearch", Some(t)) => format!("Research: {}", t),
        ("MakeResearch", None) => "Research".into(),
        ("CreateSpaceCraft", Some(t)) => format!("Build spacecraft: {}", t),
        ("CreateSpaceCraft", None) => "Build spacecraft".into(),
        ("CreateVehicle", Some(t)) => format!("Build launch vehicle: {}", t),
        ("CreateVehicle", None) => "Build launch vehicle".into(),
        ("Deliver", Some(t)) => format!("Deliver {}× {}", fmt_amount(qty), t),
        ("Deliver", None) => "Deliver".into(),
        ("DetonateNuclearDevice", _) => "Detonate nuclear device".into(),
        ("MakeEnergyProduction", _) => "Establish energy production".into(),
        ("ScheduleFly", _) => "Schedule a flight".into(),
        ("ScheduleFlyGravityAssist", _) => "Schedule a gravity-assist flight".into(),
        ("ScheduleCyclicalMission", _) => "Schedule a cyclical mission".into(),
        ("SelectLayer", _) => "Select layer".into(),
        (kind, Some(t)) => format!("{}: {}× {}", humanize_kind(kind), fmt_amount(qty), t),
        (kind, None) => humanize_kind(kind),
    }
}

/// Convert an objective kind like `"MakeResearch"` into `"Make research"`.
fn humanize_kind(kind: &str) -> String {
    let mut out = String::with_capacity(kind.len() + 4);
    for (i, c) in kind.char_indices() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
            for l in c.to_lowercase() {
                out.push(l);
            }
        } else if i == 0 {
            for u in c.to_uppercase() {
                out.push(u);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Title-case every word in `s` regardless of its existing case.  Unlike
/// `smart_title_case`, this always applies — used for ids like
/// `module_crew_compartment` that arrive lowercase and need real Title Case.
fn title_case_words(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for c in s.chars() {
        if c.is_alphabetic() {
            if at_word_start {
                for u in c.to_uppercase() {
                    out.push(u);
                }
                at_word_start = false;
            } else {
                for l in c.to_lowercase() {
                    out.push(l);
                }
            }
        } else {
            out.push(c);
            at_word_start = c.is_whitespace() || c == '/' || c == '-' || c == '(';
        }
    }
    out
}

fn page_contracts(locale: &Locale, sirenix: &Sirenix) -> String {
    let contract_name: BTreeMap<&str, &str> = locale
        .contracts
        .iter()
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();
    let contract_premise: BTreeMap<&str, &str> = locale
        .contracts
        .iter()
        .map(|c| (c.id.as_str(), c.description.as_str()))
        .collect();
    let sc_name: BTreeMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    let lv_name: BTreeMap<&str, &str> = locale
        .launch_vehicles
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    let fac_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let resource_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    let research_name: BTreeMap<&str, &str> = locale
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    // Reverse lookup: contract_id → list of contract ids that unlock it via their
    // rewards.  Contracts don't carry an explicit prerequisite field; the
    // dependency lives on the *source* contract as a reward, e.g.
    //   Mars Phase 1.rewards += UnlockContract(parameter1 = "contract_mars_marspreptwo")
    let mut unlocked_by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for c in &sirenix.contracts {
        for u in &c.unlock_rewards {
            if u.starts_with("contract_") {
                unlocked_by.entry(u.as_str()).or_default().push(c.id.as_str());
            }
        }
    }

    // Some contracts aren't unlocked by another contract at all — they're gated
    // by completing a piece of research (fusion power and asteroid pulling, as
    // of writing).  Mirror the contract-DAG construction for research, then
    // fold research-unlocks-contract edges into the depth calculation so those
    // contracts don't end up at Order 0 alongside true starting contracts.
    let mut research_prereqs: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for r in &sirenix.research {
        let entry = research_prereqs.entry(r.id.as_str()).or_default();
        for p in &r.prereqs {
            entry.push(p.as_str());
        }
    }
    // research_id → list of research ids that unlock it (mirror direction of contract_unlocks)
    let mut research_unlockers_of_contract: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for r in &sirenix.research {
        for c in &r.contract_unlocks {
            research_unlockers_of_contract
                .entry(c.as_str())
                .or_default()
                .push(r.id.as_str());
        }
    }

    // Skip non-player tutorial contracts (their wiki value is low).  Heuristic: anything
    // whose name is empty in the locale, or whose id contains "_test".
    let mut entries: Vec<&ContractStat> = sirenix
        .contracts
        .iter()
        .filter(|c| {
            contract_name.get(c.id.as_str()).map_or(false, |n| !n.is_empty())
                && !c.id.contains("_test")
        })
        .collect();
    let entry_ids: std::collections::BTreeSet<&str> =
        entries.iter().map(|c| c.id.as_str()).collect();

    // Topological depth on the research DAG: a research with no prereqs has
    // depth 0; otherwise 1 + max(depth of each prereq).
    let mut research_depth: BTreeMap<&str, u32> = BTreeMap::new();
    fn compute_research_depth<'a>(
        id: &'a str,
        prereqs: &BTreeMap<&'a str, Vec<&'a str>>,
        memo: &mut BTreeMap<&'a str, u32>,
        visiting: &mut std::collections::BTreeSet<&'a str>,
    ) -> u32 {
        if let Some(&d) = memo.get(id) {
            return d;
        }
        if !visiting.insert(id) {
            return 0;
        }
        let d = match prereqs.get(id) {
            None => 0,
            Some(srcs) if srcs.is_empty() => 0,
            Some(srcs) => srcs
                .iter()
                .map(|src| compute_research_depth(src, prereqs, memo, visiting))
                .max()
                .map(|m| m + 1)
                .unwrap_or(0),
        };
        visiting.remove(id);
        memo.insert(id, d);
        d
    }
    for r in &sirenix.research {
        let mut visiting: std::collections::BTreeSet<&str> = Default::default();
        compute_research_depth(r.id.as_str(), &research_prereqs, &mut research_depth, &mut visiting);
    }

    // Topological depth on the contract DAG, with research edges folded in.
    // For each contract c:
    //   depth(c) = 1 + max(
    //     depth(c') for every contract c' that unlocks c,
    //     research_depth(r) for every research r that unlocks c,
    //   )
    // If c has neither contract nor research unlockers, depth(c) = 0.
    let mut depth: BTreeMap<&str, u32> = BTreeMap::new();
    fn compute_depth<'a>(
        id: &'a str,
        unlocked_by: &BTreeMap<&'a str, Vec<&'a str>>,
        research_unlockers: &BTreeMap<&'a str, Vec<&'a str>>,
        research_depth: &BTreeMap<&'a str, u32>,
        memo: &mut BTreeMap<&'a str, u32>,
        visiting: &mut std::collections::BTreeSet<&'a str>,
    ) -> u32 {
        if let Some(&d) = memo.get(id) {
            return d;
        }
        if !visiting.insert(id) {
            // Cycle — break by treating this node as depth 0.
            return 0;
        }
        let contract_max: Option<u32> = unlocked_by.get(id).map(|srcs| {
            srcs.iter()
                .map(|src| {
                    compute_depth(
                        src,
                        unlocked_by,
                        research_unlockers,
                        research_depth,
                        memo,
                        visiting,
                    )
                })
                .max()
                .unwrap_or(0)
        });
        let research_max: Option<u32> = research_unlockers.get(id).map(|rs| {
            rs.iter()
                .map(|r| research_depth.get(r).copied().unwrap_or(0))
                .max()
                .unwrap_or(0)
        });
        let d = match (contract_max, research_max) {
            (None, None) => 0,
            (Some(c), None) => c + 1,
            (None, Some(r)) => r + 1,
            (Some(c), Some(r)) => c.max(r) + 1,
        };
        visiting.remove(id);
        memo.insert(id, d);
        d
    }
    for c in &entries {
        let mut visiting: std::collections::BTreeSet<&str> = Default::default();
        compute_depth(
            c.id.as_str(),
            &unlocked_by,
            &research_unlockers_of_contract,
            &research_depth,
            &mut depth,
            &mut visiting,
        );
    }

    // -----------------------------------------------------------------
    // Path A — Date-locked contracts use their year as Order.
    //
    // A contract with BOTH `is_locked == true` AND a non-empty
    // `date_time_string_start` (format `YYYY-MM-DD HH:MM:SS`, e.g.
    // `2080-01-01 00:00:00`) is not offerable until the in-game date catches
    // up.  We override its depth with the *year* extracted from that string
    // (so Exoplanet Search → Order 2080).  Descendants reachable through
    // `unlock_rewards` get +1, +2, +3 from their parent.
    //
    // The propagation is folded into the same fixed-point pass below — we
    // just seed the override here and let `depth[child] >= max_parent + 1`
    // do the rest.
    fn extract_year(s: &str) -> Option<u32> {
        // Take the first 4 chars, parse as u32.  Format is always
        // `YYYY-MM-DD HH:MM:SS` in the dump.
        let year_str: String = s.chars().take(4).collect();
        year_str.parse::<u32>().ok()
    }
    for c in &sirenix.contracts {
        if !entry_ids.contains(c.id.as_str()) {
            continue;
        }
        if !c.is_locked {
            continue;
        }
        let Some(ts) = c.date_time_string_start.as_deref() else {
            continue;
        };
        let Some(year) = extract_year(ts) else { continue };
        let cur = depth.get(c.id.as_str()).copied().unwrap_or(0);
        if year > cur {
            depth.insert(c.id.as_str(), year);
        }
    }

    // -----------------------------------------------------------------
    // Path B — objective-driven depth floors.
    //
    // A contract whose objectives gate it via research / facilities / resources
    // / spacecraft should inherit a depth ≥ depth(gating-research) + 1 even
    // when its `unlock_rewards` chain leaves it at depth 0.  Without this fix,
    // contracts like Improve Launch Methods (gated by `research_launch_magrail`)
    // and Space Laboratories (gated by `build_lab` which is unlocked by a
    // research node) sit at Order 0 alongside true starting contracts.
    //
    // Rules:
    // - `MakeResearch` + `productItem.name = research_X`  → depth ≥ depth(research_X)+1
    // - `BuildFacility` + `productItem.name = build_X`    → depth ≥ depth(research that unlocks build_X)+1
    // - `Possession` + `productItem.name = build_X`       → same as BuildFacility
    // - `Deliver` + `productItem.name = id_resource_X`    → depth ≥ depth(research for first facility producing X)+1
    // - `Possession` with no `productItem`                → depth ≥ depth(research_sc_iris)+1 (the first spacecraft research)
    //
    // Build the helper maps: facility-id → research that unlocks it, and
    // resource-id → first research that unlocks a producer of that resource.
    let mut research_unlocking_facility: BTreeMap<&str, &str> = BTreeMap::new();
    for r in &sirenix.research {
        if r.action == "UnlockFacility" {
            if let Some(target) = r.unlock_target.as_deref() {
                research_unlocking_facility.entry(target).or_insert(r.id.as_str());
            }
        }
    }
    // For each resource id, find the research depth of the earliest (lowest-
    // depth) research node that unlocks a facility producing that resource.
    let mut research_for_resource: BTreeMap<&str, u32> = BTreeMap::new();
    for f in &sirenix.facilities {
        for p in &f.produces {
            // Find the research that unlocks this facility.
            // Facility ids in the produces map are bare ("lab"), but the
            // `unlock_target` from research uses the "build_<id>" form, so we
            // try both.
            let candidates = [f.id.as_str(), &*format!("build_{}", f.id)];
            for cand in &candidates {
                if let Some(rid) = research_unlocking_facility.get(*cand) {
                    let rd = research_depth.get(*rid).copied().unwrap_or(0);
                    let entry = research_for_resource
                        .entry(p.resource_id.as_str())
                        .or_insert(rd);
                    if rd < *entry {
                        *entry = rd;
                    }
                    break;
                }
            }
        }
    }

    // Compute the objective-gate depth floor for each rendered contract.
    let mut objective_gate: BTreeMap<&str, u32> = BTreeMap::new();
    let iris_research_depth: u32 = research_depth
        .get("research_sc_iris")
        .copied()
        .unwrap_or(0);
    // Universal floor for objective-gated contracts whose target resolves to
    // no research dependency.  In production, several starting facilities
    // (build_lab, build_fuel, etc.) have `research_prereq: null` and similarly
    // their input resources have no research-gated producers.  But the player
    // still needs basic spacecraft + economy to satisfy "Have 5 labs" or
    // "Deliver 100 fuel" — so any contract whose objectives are unlocked-by-
    // economy at all gets floored to `depth(research_sc_iris) + 1`.
    let iris_floor: u32 = iris_research_depth + 1;
    for c in &entries {
        let mut floor: u32 = 0;
        for o in &c.objectives {
            let bump = match (o.kind.as_str(), o.target.as_deref()) {
                ("MakeResearch", Some(t)) if t.starts_with("research_") => {
                    research_depth.get(t).copied().map(|d| d + 1)
                }
                ("BuildFacility", Some(t)) | ("Possession", Some(t))
                    if t.starts_with("build_") =>
                {
                    let research_bump = research_unlocking_facility
                        .get(t)
                        .and_then(|r| research_depth.get(r).copied())
                        .map(|d| d + 1);
                    // Fall back to the iris floor when the facility is a
                    // starter (no research_prereq) — the contract still needs
                    // working spacecraft + economy to satisfy.
                    Some(research_bump.unwrap_or(iris_floor))
                }
                ("Deliver", Some(t)) if t.starts_with("id_resource_") => {
                    let bare = t.strip_prefix("id_resource_").unwrap_or(t);
                    let research_bump = research_for_resource
                        .get(bare)
                        .copied()
                        .map(|d| d + 1);
                    // Resources produced only by starter facilities still
                    // require the player to be flying — apply the iris floor.
                    Some(research_bump.unwrap_or(iris_floor))
                }
                ("Possession", None) => {
                    // Generic possession with no specific product — fall back
                    // to Iris (first spacecraft research) as the floor.
                    Some(iris_floor)
                }
                _ => None,
            };
            if let Some(b) = bump {
                if b > floor {
                    floor = b;
                }
            }
        }
        if floor > 0 {
            objective_gate.insert(c.id.as_str(), floor);
        }
    }
    // Apply the floor.
    for c in &entries {
        let id = c.id.as_str();
        if let Some(&floor) = objective_gate.get(id) {
            let cur = depth.get(id).copied().unwrap_or(0);
            if floor > cur {
                depth.insert(id, floor);
            }
        }
    }

    // -----------------------------------------------------------------
    // Re-propagate forward: descendants of any contract whose depth was just
    // bumped (by Path A or Path B) need their depth pushed forward too.
    // Note: for Path A, we *also* want the year-based override to act like a
    // parent, so descendants increment from the override value.  The existing
    // propagation formula (depth[child] >= depth[parent] + 1) handles that
    // naturally because we wrote the override into `depth` directly.
    loop {
        let mut changed = false;
        for c in &entries {
            let id = c.id.as_str();
            let cur = depth.get(id).copied().unwrap_or(0);
            let mut max_parent = 0u32;
            let mut has_parent = false;
            if let Some(parents) = unlocked_by.get(id) {
                for p in parents {
                    has_parent = true;
                    let pd = depth.get(*p).copied().unwrap_or(0);
                    if pd > max_parent {
                        max_parent = pd;
                    }
                }
            }
            if let Some(rs) = research_unlockers_of_contract.get(id) {
                for r in rs {
                    has_parent = true;
                    let rd = research_depth.get(*r).copied().unwrap_or(0);
                    if rd > max_parent {
                        max_parent = rd;
                    }
                }
            }
            let need = if has_parent { max_parent + 1 } else { cur };
            if need > cur {
                depth.insert(id, need);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // -----------------------------------------------------------------
    // Layer-Asteroid gating fix-up.
    //
    // In-game, every asteroid-themed contract is gated by getting the player
    // out to the asteroid belt — that's not encoded as a contract→contract
    // `unlock_rewards` edge, it's encoded on the *objective* via the
    // `layer: "Asteroid"` field.  Without this fix, contracts like
    // "Asteroid Base" sit at Order 0 because nothing precedes them in
    // unlock_rewards, even though the player can't physically attempt them
    // until they've reached the belt via the Moon/Mars chain.
    //
    // A contract is considered "asteroid-layer" iff it has at least one
    // objective with `layer: "Asteroid"` AND none of its objectives have
    // `layer: "None"`.  In the Sirenix dump, `layer` defaults to "Asteroid"
    // on every objective; the handful of contracts whose authors went out of
    // their way to mark an objective as `layer: "None"` (Humans on Mars,
    // Space Dock) are exactly the bridge contracts that take the player out
    // of the asteroid belt's gating, so we treat them as "non-asteroid" for
    // the purposes of bumping.
    //
    // The fix:
    // 1. Identify the **asteroid gate**: the asteroid-layer contract that
    //    carries a `SelectLayer` objective with `layer: "Asteroid"`.  Only
    //    one contract in production has this objective —
    //    `contract_asteroid_first` (Probing Lutetia) — which is exactly the
    //    in-game contract that asks the player to *physically choose* the
    //    asteroid layer for the first time.  Its computed depth is the gate
    //    depth.  (We don't use "min depth of asteroid-layer contract with a
    //    non-asteroid parent" because the dump's default `layer: "Asteroid"`
    //    bleeds into moon/mars campaign contracts and produces false gates.)
    // 2. Bump every asteroid-layer contract whose current depth is below the
    //    gate up to the gate depth.  This catches both stranded contracts
    //    (no parents — e.g. Asteroid Base) AND contracts whose only parent
    //    is a research node shallower than the gate (e.g. Asteroid Pulling,
    //    unlocked by `research_launch_massengine` at depth ~6 but still
    //    physically gated by reaching the asteroid belt).  Moon/Mars chain
    //    contracts are NOT asteroid-layer (they carry `layer: "None"` or
    //    don't start with `contract_asteroid_`) so they're untouched.
    // 3. Re-propagate forward via a fixed-point pass so descendants of any
    //    bumped contract get their depth recomputed from their parents.
    //
    // This step runs AFTER Path A (year overrides) and Path B (objective-
    // floor) and their joint propagation pass, because the gate itself
    // (Probing Lutetia) is reachable from Humans on Mars via unlock_rewards,
    // and Mars Landing's depth is bumped by Path B (its Hermes research
    // objective).  Running asteroid-gate before that propagation would read
    // a stale gate depth.
    //
    // Build the lookup keyed on ALL contracts (not just rendered entries) so
    // that parents which are filtered out (e.g. `_test` contracts whose
    // locale name is empty) are still classified correctly.
    //
    // The literal `objective_layers` test (contains "Asteroid", no "None")
    // matches nearly every production contract because the Sirenix dump
    // serializes the `Layer` enum's default value ("Asteroid") on every
    // unset objective.  To avoid bumping unrelated tutorial/Moon/Mars
    // contracts, we additionally require the id to start with
    // `contract_asteroid_` — an unambiguous marker for the asteroid-belt
    // contracts the game writers actually authored as such (matches the
    // user-facing examples Asteroid Base, Pulling, Sample, etc.).
    let is_asteroid_layer: BTreeMap<&str, bool> = sirenix
        .contracts
        .iter()
        .map(|c| {
            let asteroid = c.objective_layers.iter().any(|l| l == "Asteroid")
                && !c.has_layer_none_objective
                && c.id.starts_with("contract_asteroid_");
            (c.id.as_str(), asteroid)
        })
        .collect();
    // The gate is the asteroid-layer contract carrying a `SelectLayer`
    // objective.  In production only `contract_asteroid_first` qualifies.
    let gate_depth: Option<u32> = entries
        .iter()
        .filter(|c| *is_asteroid_layer.get(c.id.as_str()).unwrap_or(&false))
        .filter(|c| {
            c.objectives
                .iter()
                .any(|o| o.kind.eq_ignore_ascii_case("SelectLayer"))
        })
        .filter_map(|c| depth.get(c.id.as_str()).copied())
        .min();
    if let Some(gate) = gate_depth {
        for c in &entries {
            if !*is_asteroid_layer.get(c.id.as_str()).unwrap_or(&false) {
                continue;
            }
            // Bump every asteroid-layer contract below the gate.  Contracts
            // already deeper than the gate (e.g. Asteroid Mining at gate+1
            // because Probing Lutetia → Asteroid Mining via unlock_rewards)
            // are left alone — `cur < gate` skips them.  The gate itself
            // also satisfies `cur == gate` so it isn't bumped.  The fixed-
            // point pass below carries any bumps forward to descendants.
            let cur = depth.get(c.id.as_str()).copied().unwrap_or(0);
            if cur < gate {
                depth.insert(c.id.as_str(), gate);
            }
        }
        // Fixed-point re-propagation: a bumped contract's descendants need
        // their depth pushed forward too (depth[child] >= depth[parent] + 1
        // for every contract or research parent).
        loop {
            let mut changed = false;
            for c in &entries {
                let id = c.id.as_str();
                let cur = depth.get(id).copied().unwrap_or(0);
                let mut max_parent = 0u32;
                let mut has_parent = false;
                if let Some(parents) = unlocked_by.get(id) {
                    for p in parents {
                        has_parent = true;
                        let pd = depth.get(*p).copied().unwrap_or(0);
                        if pd > max_parent {
                            max_parent = pd;
                        }
                    }
                }
                if let Some(rs) = research_unlockers_of_contract.get(id) {
                    for r in rs {
                        has_parent = true;
                        let rd = research_depth.get(*r).copied().unwrap_or(0);
                        if rd > max_parent {
                            max_parent = rd;
                        }
                    }
                }
                let need = if has_parent { max_parent + 1 } else { cur };
                if need > cur {
                    depth.insert(id, need);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    // -----------------------------------------------------------------
    // Fix B — orphan general/spacestation contracts get a chain-derived floor.
    //
    // Contracts with id prefix `contract_general_` or `contract_spacestation_`
    // that have no `unlock_rewards` contract parent end up at Path B's iris/
    // research-derived floor, which is typically 2-6 — too shallow because
    // these are mid/late-game side-contracts the player would realistically
    // tackle around the deepest tutorial-chain final contract (Humans on
    // Mars).  Floor them at `depth(contract_mars_marslanding)` so they sort
    // alongside the chain progression a player has reached by then.
    //
    // The `if floor > cur` guard means already-deeper contracts (notably the
    // date-locked Exoplanet Search / interstellar chain at 2080+) are left
    // alone — Fix B never lowers a depth.
    let mars_landing_depth: Option<u32> = depth.get("contract_mars_marslanding").copied();
    if let Some(floor) = mars_landing_depth {
        for c in &entries {
            let id = c.id.as_str();
            if !(id.starts_with("contract_general_") || id.starts_with("contract_spacestation_")) {
                continue;
            }
            let has_contract_parent = unlocked_by
                .get(id)
                .map_or(false, |parents| {
                    parents.iter().any(|p| entry_ids.contains(*p))
                });
            if has_contract_parent {
                continue;
            }
            let cur = depth.get(id).copied().unwrap_or(0);
            if floor > cur {
                depth.insert(id, floor);
            }
        }
        // Final propagation pass: descendants of any contract floored above
        // need their depth pushed forward (children via unlock_rewards or
        // research dependency).
        loop {
            let mut changed = false;
            for c in &entries {
                let id = c.id.as_str();
                let cur = depth.get(id).copied().unwrap_or(0);
                let mut max_parent = 0u32;
                let mut has_parent = false;
                if let Some(parents) = unlocked_by.get(id) {
                    for p in parents {
                        has_parent = true;
                        let pd = depth.get(*p).copied().unwrap_or(0);
                        if pd > max_parent {
                            max_parent = pd;
                        }
                    }
                }
                if let Some(rs) = research_unlockers_of_contract.get(id) {
                    for r in rs {
                        has_parent = true;
                        let rd = research_depth.get(*r).copied().unwrap_or(0);
                        if rd > max_parent {
                            max_parent = rd;
                        }
                    }
                }
                let need = if has_parent { max_parent + 1 } else { cur };
                if need > cur {
                    depth.insert(id, need);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    // Display order: chain-DFS traversal so each campaign chain reads
    // top-to-bottom as a progression instead of getting flattened by depth.
    //
    // Roots (depth-0 contracts) come first; among roots, tutorials lead, sorted
    // by chain length (longest chain first) so the main campaign opens the
    // table.  Non-tutorial roots follow, alphabetically.  For every node, we
    // visit its children (and their subtrees) in (depth, name) order.
    //
    // A `visited` set guarantees that a contract reachable from multiple roots
    // still emits exactly once — first visit wins its position.
    //
    // The Order column still shows topological depth (computed above); only
    // the *row order* changes.

    // Forward map: parent contract id → its child contract ids.
    let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for c in &sirenix.contracts {
        if !entry_ids.contains(c.id.as_str()) {
            continue;
        }
        for u in &c.unlock_rewards {
            if u.starts_with("contract_") && entry_ids.contains(u.as_str()) {
                children.entry(c.id.as_str()).or_default().push(u.as_str());
            }
        }
    }

    // Order children of every node by (depth ascending, display-name ascending).
    for kids in children.values_mut() {
        kids.sort_by(|a, b| {
            let da = depth.get(*a).copied().unwrap_or(0);
            let db = depth.get(*b).copied().unwrap_or(0);
            let na = contract_name.get(*a).copied().unwrap_or(*a);
            let nb = contract_name.get(*b).copied().unwrap_or(*b);
            da.cmp(&db).then_with(|| na.cmp(nb))
        });
        kids.dedup();
    }

    // Chain length (in nodes) reachable from a root following unlock edges.
    // Used to rank tutorial roots so the longest campaign chain leads.
    fn chain_length<'a>(
        id: &'a str,
        children: &BTreeMap<&'a str, Vec<&'a str>>,
        memo: &mut BTreeMap<&'a str, usize>,
        visiting: &mut std::collections::BTreeSet<&'a str>,
    ) -> usize {
        if let Some(&n) = memo.get(id) {
            return n;
        }
        if !visiting.insert(id) {
            return 0;
        }
        let n = match children.get(id) {
            None => 1,
            Some(kids) if kids.is_empty() => 1,
            Some(kids) => 1 + kids
                .iter()
                .map(|k| chain_length(k, children, memo, visiting))
                .max()
                .unwrap_or(0),
        };
        visiting.remove(id);
        memo.insert(id, n);
        n
    }

    // Collect roots (depth-0 entries) and split into tutorial vs non-tutorial.
    let mut tutorial_roots: Vec<&str> = Vec::new();
    let mut other_roots: Vec<&str> = Vec::new();
    for c in &entries {
        let d = depth.get(c.id.as_str()).copied().unwrap_or(0);
        if d != 0 {
            continue;
        }
        if c.id.contains("_tutorial_") {
            tutorial_roots.push(c.id.as_str());
        } else {
            other_roots.push(c.id.as_str());
        }
    }
    let mut chain_len_memo: BTreeMap<&str, usize> = BTreeMap::new();
    tutorial_roots.sort_by(|a, b| {
        let mut va: std::collections::BTreeSet<&str> = Default::default();
        let mut vb: std::collections::BTreeSet<&str> = Default::default();
        let la = chain_length(a, &children, &mut chain_len_memo, &mut va);
        let lb = chain_length(b, &children, &mut chain_len_memo, &mut vb);
        let na = contract_name.get(*a).copied().unwrap_or(*a);
        let nb = contract_name.get(*b).copied().unwrap_or(*b);
        // Longest chain first; alphabetical tiebreak on display name.
        lb.cmp(&la).then_with(|| na.cmp(nb))
    });
    other_roots.sort_by(|a, b| {
        let na = contract_name.get(*a).copied().unwrap_or(*a);
        let nb = contract_name.get(*b).copied().unwrap_or(*b);
        na.cmp(nb)
    });

    let root_order: Vec<&str> = tutorial_roots
        .into_iter()
        .chain(other_roots.into_iter())
        .collect();

    // DFS-emit order from each root.  `visited` guards against double-visits
    // when a node is reachable from multiple parents/roots.
    let mut emit_order: Vec<&str> = Vec::with_capacity(entries.len());
    let mut visited: std::collections::BTreeSet<&str> = Default::default();
    fn dfs_emit<'a>(
        id: &'a str,
        children: &BTreeMap<&'a str, Vec<&'a str>>,
        visited: &mut std::collections::BTreeSet<&'a str>,
        out: &mut Vec<&'a str>,
    ) {
        if !visited.insert(id) {
            return;
        }
        out.push(id);
        if let Some(kids) = children.get(id) {
            for k in kids {
                dfs_emit(k, children, visited, out);
            }
        }
    }
    for root in &root_order {
        dfs_emit(root, &children, &mut visited, &mut emit_order);
    }
    // Safety net: any contract not yet emitted (e.g., orphan whose only
    // unlockers are filtered-out _test contracts so it never showed up as a
    // depth-0 root but also has no live parent) goes at the end, in
    // (depth, name) order — matching the legacy sort for those leftovers.
    let mut leftovers: Vec<&str> = entries
        .iter()
        .map(|c| c.id.as_str())
        .filter(|id| !visited.contains(id))
        .collect();
    leftovers.sort_by(|a, b| {
        let da = depth.get(*a).copied().unwrap_or(0);
        let db = depth.get(*b).copied().unwrap_or(0);
        let na = contract_name.get(*a).copied().unwrap_or(*a);
        let nb = contract_name.get(*b).copied().unwrap_or(*b);
        da.cmp(&db).then_with(|| na.cmp(nb))
    });
    for id in leftovers {
        if visited.insert(id) {
            emit_order.push(id);
        }
    }

    // Reorder `entries` to match `emit_order`.
    let position: BTreeMap<&str, usize> = emit_order
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    entries.sort_by_key(|c| position.get(c.id.as_str()).copied().unwrap_or(usize::MAX));

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|c| {
            let display = contract_name.get(c.id.as_str()).copied().unwrap_or(c.id.as_str());
            let premise = contract_premise.get(c.id.as_str()).copied().unwrap_or("");
            let premise = truncate_premise(premise, 240);

            // Objectives: dedupe identical lines (same kind + target + qty).
            let mut obj_bits: Vec<String> = Vec::new();
            let mut seen_obj: std::collections::BTreeSet<String> = Default::default();
            for o in &c.objectives {
                let line = format_objective(
                    o,
                    &fac_name,
                    &resource_name,
                    &sc_name,
                    &lv_name,
                    &research_name,
                );
                if seen_obj.insert(line.clone()) {
                    obj_bits.push(line);
                }
            }
            let requirements = if obj_bits.is_empty() { "—".to_string() } else { obj_bits.join("<br>") };

            let mut reward_bits: Vec<String> = Vec::new();
            if c.money_reward > 0.0 {
                reward_bits.push(format!("Cash: {}", fmt_abbrev(c.money_reward)));
            }
            for r in &c.resource_grants {
                let label = resource_name
                    .get(r.resource_id.as_str())
                    .copied()
                    .unwrap_or(r.resource_id.as_str());
                reward_bits.push(format!("{} {}", fmt_amount(r.amount), label));
            }
            for f in &c.facility_grants {
                let key = f.strip_prefix("build_").unwrap_or(f);
                let pretty = smart_title_case(fac_name.get(key).copied().unwrap_or(key));
                reward_bits.push(format!("Facility: {}", pretty));
            }
            for s in &c.spacecraft_grants {
                let pretty = sc_name.get(s.as_str()).copied().unwrap_or(s.as_str());
                reward_bits.push(format!("Spacecraft: {}", pretty));
            }
            for l in &c.launch_vehicle_grants {
                let pretty = lv_name.get(l.as_str()).copied().unwrap_or(l.as_str());
                reward_bits.push(format!("Launch Vehicle: {}", pretty));
            }
            for u in &c.unlock_rewards {
                let pretty = contract_name.get(u.as_str()).copied().unwrap_or(u.as_str());
                if pretty != *u && !pretty.is_empty() && !u.contains("_test") {
                    let link = link_same_page("contract", u, &escape_cell(pretty));
                    reward_bits.push(format!("Next: {}", link));
                }
            }
            let rewards = if reward_bits.is_empty() {
                "—".to_string()
            } else {
                reward_bits.join("<br>")
            };

            let anchor = anchor_tag("contract", &c.id);
            let name_body = if c.is_final {
                format!("**{}** *(final)*", escape_cell(display))
            } else {
                format!("**{}**", escape_cell(display))
            };
            let name_cell = format!("{anchor}{name_body}");

            // Prereq column — which contracts must complete before this one
            // is offered.  Built from the reverse-rewards lookup.  Filter the
            // same way `entries` is filtered: drop tutorial/_test contracts
            // and any source with an empty locale display name.  Date-locked
            // contracts (isLocked + dateTimeStringStart) also get a
            // `Year ≥ YYYY` line so the time-gate is visible in-row, not
            // just hidden behind the Order column.
            let mut prereq_parts: Vec<String> = Vec::new();
            if c.is_locked {
                if let Some(ts) = c.date_time_string_start.as_deref() {
                    let year_str: String = ts.chars().take(4).collect();
                    if year_str.len() == 4 && year_str.chars().all(|ch| ch.is_ascii_digit()) {
                        prereq_parts.push(format!("*Year ≥ {year_str}*"));
                    }
                }
            }
            if let Some(srcs) = unlocked_by.get(c.id.as_str()) {
                for src in srcs {
                    if src.contains("_test") {
                        continue;
                    }
                    let pretty = contract_name.get(src).copied().unwrap_or("");
                    if pretty.is_empty() {
                        continue;
                    }
                    prereq_parts.push(link_same_page("contract", src, &escape_cell(pretty)));
                }
            }
            let prereq_cell = if prereq_parts.is_empty() {
                "—".to_string()
            } else {
                prereq_parts.join("<br>")
            };

            let order_cell = depth.get(c.id.as_str()).copied().unwrap_or(0).to_string();

            vec![
                order_cell,
                name_cell,
                prereq_cell,
                requirements,
                rewards,
                escape_cell(&premise),
            ]
        })
        .collect();
    let table = md_table_with_tips(
        &["Order", "Contract", "Prereq", "Requirements", "Rewards", "Premise"],
        &[
            Some("Dependency depth: 0 = no prereq, N = unlocked after an Order N-1 contract"),
            None,
            Some("Contracts that must complete before this one is offered"),
            Some("Objectives that must be completed to claim the rewards"),
            Some("Cash, resources, unlocks, and follow-up contracts granted on completion"),
            None,
        ],
        &rows,
    );
    // Wrap so the global sortable-table JS skips this table — chain-DFS
    // visit order is the meaningful sort and any other reorder breaks it.
    // The Order column itself is hidden via CSS (.no-sort table th:first-child
    // / td:first-child { display: none }) so the player sees a clean table
    // while the depth value stays in the markup for tests + screen readers.
    let table = format!("<div class=\"no-sort\" markdown=\"1\">\n\n{table}\n</div>\n");
    format!(
        "# Contracts\n\n\
Contracts drive progression in Solar Expanse — they're the game's source of\n\
funding alongside resource sales. Each contract is a set of objectives — usually\n\
\"build facility X on body Y\" or \"deliver Z to a destination\" — that pay\n\
out cash, resources, or unlocks when complete. Many contracts also unlock\n\
the next link in a chain (Mars Phase 1 → Mars Phase 2 → …), a new spacecraft,\n\
or a new launch vehicle.\n\n\
For mission planning mechanics (flight planning, gravity assists, cyclical routes), see [Missions](../missions/).\n\n\
{table}\n\
## Reading the table\n\n\
- The **Order** column is the contract's dependency depth in the unlock DAG (0 = no prereq, N = unlocked after an Order N-1 contract). Rows are sorted by **campaign chain** — each starting contract is followed by its full follow-up chain (depth-first), so a tutorial or campaign reads top-to-bottom as a progression instead of jumping around by depth. Tutorial chains come first; non-tutorial roots follow alphabetically.\n\
- A contract marked **(final)** ends a campaign chain.\n\
- **Requirements**: the objectives you have to complete to claim the payout. Body-specific objectives (\"deliver 100 t to Mars\") list the *what* but not the destination — the premise text describes the target.\n\
- **Rewards**: cash, resources, facility / spacecraft / launch-vehicle unlocks, and the next contract in the chain.\n\
- **Premise**: the in-game flavor text introducing the contract.\n"
    )
}

/// Title-case a string if it's all uppercase (e.g., "HYDROPONIC FARM" → "Hydroponic Farm").
/// Leaves mixed-case strings alone.
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

/// Format a facility-side magnitude (role or habitability delta). Unlike
/// `fmt_amount`, sub-unit values keep three significant digits so a sunshade's
/// `-0.006` albedo doesn't collapse to "-0.0". Integer values print without a
/// decimal point. Always returns the absolute value — callers add the sign so
/// they can swap the ASCII `-` for the Unicode `−` consistently.
fn fmt_magnitude_abs(v: f64) -> String {
    let a = v.abs();
    if a == 0.0 {
        return "0".to_string();
    }
    if a == a.trunc() && a < 1e9 {
        return format!("{}", a as i64);
    }
    let raw = if a >= 10.0 {
        format!("{a:.1}")
    } else if a >= 1.0 {
        format!("{a:.2}")
    } else if a >= 0.001 {
        // Three decimals carries the sunshade's -0.006 albedo delta without
        // losing precision; the magnet station's 0.6 just renders as "0.6".
        format!("{a:.3}")
    } else {
        // Very small fractions (e.g. 1e-4) — drop to scientific notation so
        // we don't render "0.000" and lose the value entirely.
        return format!("{a:.1e}");
    };
    // Strip trailing zeros to keep cells tight ("0.600" → "0.6", "2.50" → "2.5").
    let trimmed = raw.trim_end_matches('0').trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed
    }
}

/// Render the `specialAbilityFacilityNew` enum + its parameter into a single
/// player-facing cell. The enum is the source of truth for the kind of role
/// (CrewCapacity, Lab, Mining, EnergyProduction, …) and the parameter is the
/// magnitude. We collapse them into "<friendly label> <number>" so the table
/// can carry one column instead of two. A few enum values arrive as combined
/// strings like "EnergyProduction, EnergyStorage" — we pass them through with
/// minimal massaging because the in-game UI uses the same phrasing.
fn fmt_facility_role(role: Option<&str>, magnitude: f64) -> String {
    let Some(r) = role else { return "—".to_string() };
    if r.is_empty() {
        return "—".to_string();
    }
    let label: String = match r {
        "CrewCapacity" => "Crew".into(),
        "Refiner" => "Refiner".into(),
        "Mining" => "Mining".into(),
        "SpaceMirrorOrShade" => "Mirror/Shade".into(),
        "ConstructionEquipment" => "Construction".into(),
        "Lab" => "Lab".into(),
        "BuildSpacecraft" => "Spacecraft assembly".into(),
        "EnergyProduction" => "Power output".into(),
        "EnergyStorage" => "Power storage".into(),
        "EnergyProduction, EnergyStorage" => "Power / storage".into(),
        "Bonus" => "Bonus".into(),
        other => other.to_string(),
    };
    // Magnitude meaning varies by role. We always render it numerically; the
    // role label provides the unit context. Zero magnitudes typically mean
    // "the magnitude lives on a dynamic subclass" (e.g. launch facilities mark
    // themselves as Bonus with param 0 because the actual bonus is in
    // bonusData); show just the label in that case. Negative values get a
    // Unicode minus for consistency with terraforming deltas.
    if magnitude == 0.0 {
        label
    } else if magnitude < 0.0 {
        format!("{} −{}", label, fmt_magnitude_abs(magnitude))
    } else {
        format!("{} {}", label, fmt_magnitude_abs(magnitude))
    }
}

/// Render the launch-method `bonusData` object into a player-facing phrase.
/// The dump uses three enum values:
///   * `LaunchCost` — flat discount to the launch cost. Param = percent.
///   * `LaunchCostOptionInPlanMission` — per-mission planner discount option.
///   * `SpaceCraftInPlanMission` — provides a fake spacecraft option in plan.
/// We never surface the raw enum name.
fn fmt_launch_bonus(bd: Option<&(String, f64)>) -> String {
    let Some((kind, param)) = bd else { return "—".to_string() };
    let pct = fmt_amount(*param);
    match kind.as_str() {
        "LaunchCost" => format!("−{pct}% launch cost"),
        "LaunchCostOptionInPlanMission" => format!("−{pct}% in plan-mission"),
        "SpaceCraftInPlanMission" => format!("+{pct} payload option"),
        // Unknown future enum value: render the parameter without leaking the
        // raw enum name (kept short so the column stays narrow).
        _ => format!("+{pct}"),
    }
}

/// Render the habitability-parameter deltas as a comma-separated list of
/// signed entries. We prefer the Unicode minus (U+2212) for negative numbers
/// — it's typographically nicer and used elsewhere in the wiki. Positive
/// values get an explicit `+` so the direction of each delta is unambiguous.
fn fmt_habitability_deltas(deltas: &[(String, f64)]) -> String {
    if deltas.is_empty() {
        return "—".to_string();
    }
    deltas
        .iter()
        .map(|(label, value)| {
            let sign = if *value < 0.0 { "−" } else { "+" };
            format!("{label} {sign}{}", fmt_magnitude_abs(*value))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render the habitability *build constraints* — the per-body gates a
/// facility imposes on where it can be constructed. The dump exposes these
/// as `(parameter, min, max)` triples on
/// `canBuildParameter.terraformParameterCanBuild.list`.
///
/// We pick a player-facing label per parameter. Two special-cased pressure
/// ranges are recognised because the game treats the 0.0001 atm threshold as
/// the vacuum/atmosphere boundary:
///   * `min == 0` and `max <= 0.0001` → "Vacuum only"
///   * `min >= 0.0001` and `max >= 2` → "Atmosphere required"
/// Anything else falls back to a generic `{Parameter} {min}–{max}` so we
/// never lie about an unfamiliar range.
fn fmt_habitat_constraint(c: &HabitatConstraintStat) -> String {
    // Trim up to 4 decimals and drop trailing zeros — keeps cells narrow while
    // still showing the 0.0001 thresholds verbatim when we fall back.
    let fmt = |n: f64| -> String {
        if n == 0.0 {
            return "0".to_string();
        }
        let s = format!("{:.4}", n);
        // Strip trailing zeros and the dot if the number is integral.
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    };
    // Object-kind gates are encoded with a synthetic `ObjectType:<Kind>`
    // parameter; we render them as a body-kind requirement rather than a
    // numeric range. Currently only "Asteroid" is used in the dump.
    if let Some(kind) = c.parameter.strip_prefix("ObjectType:") {
        return match kind {
            "Asteroid" => "Asteroid only".to_string(),
            other => format!("{other} only"),
        };
    }
    match c.parameter.as_str() {
        "Pressure" => {
            if c.min == 0.0 && c.max <= 0.0001 {
                "Vacuum only".to_string()
            } else if c.min >= 0.0001 && c.max >= 2.0 {
                "Atmosphere required".to_string()
            } else {
                format!("Pressure {}–{}", fmt(c.min), fmt(c.max))
            }
        }
        // For every other parameter (Temperature, Gravity, Radiation, Water,
        // Composition, InternalFlux) we render generically rather than guess
        // at a player-friendly label. The label itself is the raw enum name
        // straight from the dump.
        param => format!("{} {}–{}", param, fmt(c.min), fmt(c.max)),
    }
}

fn fmt_habitat_constraints(cs: &[HabitatConstraintStat]) -> String {
    if cs.is_empty() {
        return "—".to_string();
    }
    cs.iter()
        .map(fmt_habitat_constraint)
        .collect::<Vec<_>>()
        .join("<br>")
}

fn page_facilities(locale: &Locale, sirenix: &Sirenix) -> String {
    let facility_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let facility_desc: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.description.as_str()))
        .collect();
    let resource_name: BTreeMap<&str, &str> = locale
        .resources
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    let research_name: BTreeMap<&str, &str> = locale
        .research
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();

    // Build a facility_id → research_id map by walking the research nodes that
    // declare an UnlockFacility action.  The facility's own `lockByHelpNotUse`
    // field is set only for a handful of facilities, but every researched
    // facility has a corresponding research with parameter1 = build_<id>.
    let facility_unlocked_by: BTreeMap<&str, &str> = sirenix
        .research
        .iter()
        .filter(|r| r.action == "UnlockFacility")
        .filter_map(|r| r.unlock_target.as_deref().map(|t| (t, r.id.as_str())))
        .collect();

    let mut ground: Vec<&FacilityStat> = Vec::new();
    let mut orbital: Vec<&FacilityStat> = Vec::new();
    for f in &sirenix.facilities {
        if f.is_obsolete {
            continue;
        }
        // FacilitySegment entries are interstellar-ship-construction stages —
        // narrative one-offs, not buildable production facilities.  Skip them.
        if f.facility_type == "FacilitySegment" {
            continue;
        }
        // Spacecraft-payload modules (engine, crew, mining-rig, etc.) come in via
        // SpaceModuleDescriptor.  Treat them as spacecraft payload, not facilities;
        // skip them on the Facilities page.
        if f.descriptor == "Orbital" {
            continue;
        }
        // Drop entries without a player-facing locale name; their data is incomplete.
        let id_no_prefix = f.id.strip_prefix("build_").unwrap_or(&f.id);
        if !locale
            .facilities
            .iter()
            .any(|lf| lf.id == id_no_prefix && !lf.name.is_empty())
        {
            continue;
        }
        // Split by where the facility can be placed.  Surface → ground table,
        // Orbit → orbital table.  A facility with BOTH ("Surface, Orbit") is
        // emitted in BOTH tables — player wants to see it in whichever
        // context they're scanning.  The anchor uniqueness was handled
        // upstream by emitting `<a id="...">` only on the first occurrence.
        let p = f.placement.as_str();
        let surface_ok = p.contains("Surface") || p.contains("SurfaceAndAsteroid");
        let orbit_ok = p.contains("Orbit");
        if surface_ok {
            ground.push(f);
        }
        if orbit_ok {
            orbital.push(f);
        }
        if !surface_ok && !orbit_ok {
            // Placement empty / unknown — default to ground rather than drop.
            ground.push(f);
        }
    }
    let sorter = |a: &&FacilityStat, b: &&FacilityStat| {
        a.facility_type.cmp(&b.facility_type).then(a.id.cmp(&b.id))
    };
    ground.sort_by(sorter);
    orbital.sort_by(sorter);

    let row_for = |f: &FacilityStat,
                   facility_name: &BTreeMap<&str, &str>,
                   research_name: &BTreeMap<&str, &str>,
                   resource_name: &BTreeMap<&str, &str>|
     -> Vec<String> {
        let id_no_prefix = f.id.strip_prefix("build_").unwrap_or(&f.id);
        let raw_display = facility_name.get(id_no_prefix).copied().unwrap_or(id_no_prefix);
        let mut display = smart_title_case(raw_display);
        // Disambiguate tier variants whose locale display name collides with
        // the small variant — `icemine` and `icemine_big` both resolve to
        // "Water Ice Extractor" (same for metalmine / metalmine_big etc.).
        // Append "(Large)" so the player can tell the two rows apart.
        if id_no_prefix.ends_with("_big") || id_no_prefix.ends_with("-big") {
            display.push_str(" (Advanced)");
        }
        // Prefer the reverse-lookup (which research unlocks this facility?) over
        // the facility's own `lockByHelpNotUse` field — the former is set for
        // every researched facility, the latter only for a few.
        let prereq_id = facility_unlocked_by
            .get(f.id.as_str())
            .copied()
            .or_else(|| f.research_prereq.as_deref());
        let prereq = prereq_id
            .map(|r| {
                let name = research_name.get(r).copied().unwrap_or(r).to_string();
                link_cross_page("research", "research", r, &escape_cell(&name))
            })
            .unwrap_or_else(|| "—".to_string());
        let workers = if f.workers_required > 0 {
            f.workers_required.to_string()
        } else {
            "—".to_string()
        };
        let energy = if f.energy_consumption > 0.0 {
            format!("{}", fmt_amount(f.energy_consumption))
        } else {
            "—".to_string()
        };
        let maint = if f.maintenance_per_day > 0.0 {
            fmt_abbrev(f.maintenance_per_day)
        } else {
            "—".to_string()
        };
        let desc = facility_desc.get(id_no_prefix).copied().unwrap_or("");
        let name_cell = format!(
            "{anchor}**{name}**",
            anchor = anchor_tag("facility", id_no_prefix),
            name = escape_cell(&display),
        );
        let time = if f.build_time_days > 0.0 {
            fmt_amount(f.build_time_days)
        } else {
            "—".to_string()
        };
        let role = fmt_facility_role(f.role.as_deref(), f.role_magnitude);
        let launch_bonus = fmt_launch_bonus(f.bonus_data.as_ref());
        let terraforming = fmt_habitability_deltas(&f.habitability_deltas);
        let habitat_req = fmt_habitat_constraints(&f.habitat_constraints);
        vec![
            name_cell,
            f.facility_type.clone(),
            fmt_build_cost(&f.build_cost, resource_name),
            time,
            role,
            workers,
            energy,
            maint,
            launch_bonus,
            terraforming,
            habitat_req,
            prereq,
            escape_cell(desc),
        ]
    };

    let header = [
        "Facility",
        "Type",
        "Build cost",
        "Time",
        "Role",
        "Workers",
        "Energy",
        "Maint",
        "Launch bonus",
        "Terraforming",
        "Habitat req.",
        "Prereq",
        "Description",
    ];

    let ground_rows: Vec<Vec<String>> = ground
        .iter()
        .map(|f| row_for(f, &facility_name, &research_name, &resource_name))
        .collect();
    let orbital_rows: Vec<Vec<String>> = orbital
        .iter()
        .map(|f| row_for(f, &facility_name, &research_name, &resource_name))
        .collect();

    let header_tips: [Option<&str>; 13] = [
        None,
        Some("Facility category — Production / Mining / Power / Habitation / …"),
        Some("Resources required to construct"),
        Some("Days required to construct"),
        Some("Primary on-site role and its magnitude (crew capacity, research rate, mining rate, etc.)"),
        Some("On-site population required for full output"),
        Some("Energy consumed per day"),
        Some("Daily maintenance cost"),
        Some("Bonus granted to launches that originate here"),
        Some("Per-day deltas applied to the planet's habitability parameters"),
        Some("Habitability constraints — pressure/temperature/etc. ranges this facility requires on the body it's built on."),
        Some("Research that unlocks this facility"),
        None,
    ];
    let ground_table = md_table_with_tips(&header, &header_tips, &ground_rows);
    let orbital_table = md_table_with_tips(&header, &header_tips, &orbital_rows);

    // Build per-table filter blocks: text search + a checkbox per distinct
    // facility type in that table.  Types are sorted alphabetically and
    // humanized (`LaunchFacility` → `Launch Facility`).
    let humanize_type = |t: &str| -> String {
        let mut out = String::with_capacity(t.len() + 4);
        for (i, c) in t.char_indices() {
            if i > 0 && c.is_uppercase() { out.push(' '); }
            out.push(c);
        }
        out
    };
    let build_filter_block = |id_suffix: &str, facs: &[&FacilityStat]| -> String {
        let mut types: Vec<&str> = facs.iter().map(|f| f.facility_type.as_str()).collect();
        types.sort();
        types.dedup();
        let mut block = String::new();
        block.push_str(&format!(
            "<div class=\"facility-filter\" data-table=\"{id_suffix}\">\n",
        ));
        block.push_str(&format!(
            "<label>Filter: <input class=\"facility-filter-search\" data-table=\"{id_suffix}\" type=\"search\" placeholder=\"facility name…\" autocomplete=\"off\"></label>\n",
        ));
        for t in &types {
            block.push_str(&format!(
                "<label><input type=\"checkbox\" class=\"facility-type-filter\" data-table=\"{id_suffix}\" value=\"{t}\" checked> {label}</label>\n",
                t = t,
                label = humanize_type(t),
            ));
        }
        block.push_str("</div>\n\n");
        block
    };
    let ground_filter = build_filter_block("ground", &ground);
    let orbital_filter = build_filter_block("orbital", &orbital);

    format!(
        "# Facilities\n\n\
Facilities are the buildings and modules you place on planets, moons, asteroids,\n\
and in orbit. Each consumes power and workers, may require a research\n\
prerequisite, and either produces, processes, or enables something — power\n\
plants generate energy, refineries turn ore into refined metal, mines extract\n\
raw resources, etc.\n\n\
Facilities are split into two families:\n\n\
- **Ground facilities** sit on a body's surface. They use local workers and\n\
  may need atmospheric conditions to function.\n\
- **Orbital modules** attach to a space station or shipyard in orbit. They\n\
  don't need a habitable surface, but you have to build the station first.\n\n\
## Ground facilities\n\n\
{ground_filter}{ground_table}\n\
## Orbital modules\n\n\
{orbital_filter}{orbital_table}\n\
## Reading the table\n\n\
- **Type** is the gameplay category — *Production*, *Mining*, *Storage*, *Power*, *Habitat*, etc. The Solar Expanse UI groups facilities by type when you open the build menu.\n\
- **Time** is the build duration in days.\n\
- **Role** combines the facility's primary on-site role with its magnitude — *Crew 100* for a habitat, *Lab 1* for a research lab, *Mining 10* for a heavy mine, *Mirror/Shade −0.006* for a sunshade's albedo delta, etc. Facilities with no specific role show `—`.\n\
- **Workers** is the on-site population the facility needs to operate at full output. Most facilities throttle when understaffed.\n\
- **Energy/day** is the running energy demand. Power facilities show this as `—`; everything else is a consumer.\n\
- **Maintenance** is the per-day cash upkeep while the facility is active.\n\
- **Launch bonus** appears only for launch facilities — it describes the discount or capacity gain applied to launches that originate from this facility.\n\
- **Terraforming** lists the per-day deltas the facility applies to a body's habitability parameters (temperature, atmosphere, gravity, radiation, magnetic field). Empty for everything except a handful of dedicated terraforming structures.\n\
- **Habitat req.** is a hard prerequisite on the body itself — \"Vacuum only\" for mass drivers, \"Atmosphere required\" for magnetic launch rails, gravity/radiation/pressure envelopes for habitats, etc. The game blocks construction if the body's reading is outside the listed range. `—` means the facility has no body-side requirement.\n\
- **Research prereq** is the research that unlocks construction; `—` means it's available from the start (or the prereq lives outside the standard `lockByHelpNotUse` field, which a few specialist facilities use).\n\n\
<script src=\"{{{{ '/assets/js/facilities.js' | relative_url }}}}?v={{{{ site.github.build_revision | default: 'dev' }}}}\" defer></script>\n"
    )
}

fn fmt_work_hours(h: f64) -> String {
    if h <= 0.0 {
        "—".into()
    } else if h >= 1_000_000.0 {
        format!("{:.1}M", h / 1_000_000.0)
    } else if h >= 1_000.0 {
        format!("{:.0}k", h / 1_000.0)
    } else {
        format!("{:.0}", h)
    }
}

/// Convert a snake_case id fragment into Title Case words.
fn humanize_id_fragment(s: &str) -> String {
    let cleaned = s.replace('_', " ");
    let mut out = String::with_capacity(cleaned.len());
    let mut cap_next = true;
    for c in cleaned.chars() {
        if c.is_whitespace() {
            out.push(c);
            cap_next = true;
        } else if cap_next && c.is_alphabetic() {
            for u in c.to_uppercase() { out.push(u); }
            cap_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Resolve a single UnlockBonus `bonus_components` token to a player-facing label.
/// `build_*` → facility display name; `id_Rocket_*` / `lv_*` → LV name;
/// `spacecraft_*` → spacecraft name; `module_*` / `eng_*` / `cargo_*` →
/// humanized fragment; sentinel "All"/"Facility"/"LV" → readable label.
fn resolve_bonus_component(
    target: &str,
    facility_name: &BTreeMap<&str, &str>,
    spacecraft_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
) -> String {
    if let Some(key) = target.strip_prefix("build_") {
        let name = facility_name.get(key).copied().unwrap_or(key);
        return smart_title_case(name);
    }
    if target.starts_with("id_Rocket_") || target.starts_with("lv_") {
        if let Some(name) = lv_name.get(target).copied() {
            return smart_title_case(name);
        }
        let frag = target
            .strip_prefix("id_Rocket_")
            .or_else(|| target.strip_prefix("lv_"))
            .unwrap_or(target);
        return humanize_id_fragment(frag);
    }
    if target.starts_with("spacecraft_") {
        if let Some(name) = spacecraft_name.get(target).copied() {
            return smart_title_case(name);
        }
        let frag = target.strip_prefix("spacecraft_").unwrap_or(target);
        return humanize_id_fragment(frag);
    }
    for prefix in &["module_", "eng_", "cargo_"] {
        if let Some(frag) = target.strip_prefix(prefix) {
            return humanize_id_fragment(frag);
        }
    }
    match target {
        "All" => "(all)".to_string(),
        "Facility" => "(all facilities)".to_string(),
        "LV" => "(all launch vehicles)".to_string(),
        other => humanize_id_fragment(other),
    }
}

/// Map the era tier from `ResearchStat.stage` (0/1/2) to a player-facing label.
/// Values outside the known range fall through to "Early" so the column never
/// renders an empty cell.
fn era_label(stage: u8) -> &'static str {
    match stage {
        0 => "Early",
        1 => "Mid",
        2 => "Late",
        _ => "Early",
    }
}

/// Render a single `SecondaryUnlockStat` as one line of the Unlocks cell.
/// Bonuses look like `+20 Component thrust`; facility/spacecraft/LV unlocks
/// look like `Facility: Habitat Dome` (linked when an anchor exists).
fn fmt_secondary_unlock(
    s: &SecondaryUnlockStat,
    facility_name: &BTreeMap<&str, &str>,
    facility_anchored: &std::collections::BTreeSet<&str>,
    spacecraft_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
) -> String {
    match s.action.as_str() {
        "UnlockBonus" => {
            let kind = if s.bonus.is_empty() { "Bonus".to_string() } else { humanize_kind(&s.bonus) };
            let sign = if s.bonus_parameter < 0.0 { "" } else { "+" };
            format!("{sign}{} {}", fmt_amount(s.bonus_parameter), kind)
        }
        "UnlockFacility" => {
            let key = s.target.strip_prefix("build_").unwrap_or(&s.target);
            let resolved_name = facility_name.get(key).copied();
            let pretty = match resolved_name {
                Some(name) if !name.is_empty() => smart_title_case(name),
                _ => {
                    if key.contains('_') || key.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()) {
                        title_case_words(&key.replace('_', " "))
                    } else {
                        smart_title_case(key)
                    }
                }
            };
            if facility_anchored.contains(key) {
                let link = link_cross_page("facilities", "facility", key, &format!("**{pretty}**"));
                format!("Facility: {link}")
            } else {
                format!("Facility: **{pretty}**")
            }
        }
        "UnlockSpacecraftType" => {
            let pretty = spacecraft_name.get(s.target.as_str()).copied().unwrap_or(s.target.as_str());
            let link = link_cross_page("spacecraft", "spacecraft", &s.target, &format!("**{pretty}**"));
            format!("Spacecraft: {link}")
        }
        "UnlockVehicleType" => {
            let pretty = lv_name.get(s.target.as_str()).copied().unwrap_or(s.target.as_str());
            let link = link_cross_page("launch-vehicles", "lv", &s.target, &format!("**{pretty}**"));
            format!("Launch Vehicle: {link}")
        }
        other => other.to_string(),
    }
}

fn fmt_research_unlock(
    r: &ResearchStat,
    facility_name: &BTreeMap<&str, &str>,
    facility_anchored: &std::collections::BTreeSet<&str>,
    spacecraft_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
) -> String {
    let primary = fmt_research_unlock_primary(r, facility_name, facility_anchored, spacecraft_name, lv_name);
    if r.secondary_unlocks.is_empty() {
        return primary;
    }
    let mut lines: Vec<String> = Vec::with_capacity(r.secondary_unlocks.len() + 1);
    if primary != "—" {
        lines.push(primary);
    }
    for s in &r.secondary_unlocks {
        lines.push(fmt_secondary_unlock(s, facility_name, facility_anchored, spacecraft_name, lv_name));
    }
    if lines.is_empty() { "—".into() } else { lines.join("<br>") }
}

fn fmt_research_unlock_primary(
    r: &ResearchStat,
    facility_name: &BTreeMap<&str, &str>,
    facility_anchored: &std::collections::BTreeSet<&str>,
    spacecraft_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
) -> String {
    match r.action.as_str() {
        "UnlockFacility" => {
            let target = r.unlock_target.as_deref().unwrap_or("");
            // Facility unlock targets are full ids like "build_outpost"; locale ids are bare ("outpost").
            let key = target.strip_prefix("build_").unwrap_or(target);
            // Some research nodes carry `UnlockFacility` actions whose target
            // is a spacecraft module (e.g. `module_crew_compartment`,
            // `asteroid_engine_module`) — the facilities page filters those
            // out, so the would-be anchor `facility-…` doesn't exist.  Only
            // emit a cross-page link when an anchor is actually rendered.
            let resolved_name = facility_name.get(key).copied();
            let pretty = match resolved_name {
                Some(name) if !name.is_empty() => smart_title_case(name),
                _ => {
                    // No locale name; humanize the raw id.
                    if key.contains('_')
                        || key.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit())
                    {
                        title_case_words(&key.replace('_', " "))
                    } else {
                        smart_title_case(key)
                    }
                }
            };
            if facility_anchored.contains(key) {
                let link = link_cross_page("facilities", "facility", key, &format!("**{pretty}**"));
                format!("Builds {link}")
            } else {
                format!("Builds **{pretty}**")
            }
        }
        "UnlockSpacecraftType" => {
            let target = r.unlock_target.as_deref().unwrap_or("");
            let pretty = spacecraft_name.get(target).copied().unwrap_or(target);
            let link = link_cross_page("spacecraft", "spacecraft", target, &format!("**{pretty}**"));
            format!("Spacecraft: {link}")
        }
        "UnlockVehicleType" => {
            let target = r.unlock_target.as_deref().unwrap_or("");
            let pretty = lv_name.get(target).copied().unwrap_or(target);
            let link = link_cross_page("launch-vehicles", "lv", target, &format!("**{pretty}**"));
            format!("Launch Vehicle: {link}")
        }
        "UnlockBonus" => match &r.bonus_kind {
            Some(b) => {
                let comps = if r.bonus_components.is_empty() {
                    "".to_string()
                } else {
                    let names: Vec<String> = r
                        .bonus_components
                        .iter()
                        .map(|t| resolve_bonus_component(t, facility_name, spacecraft_name, lv_name))
                        .collect();
                    format!(" on {}", names.join(", "))
                };
                // fmt_amount emits a leading `-` for negatives; only prepend `+` for non-negatives.
                let sign = if r.bonus_amount < 0.0 { "" } else { "+" };
                format!("{sign}{} {}{}", fmt_amount(r.bonus_amount), b, comps)
            }
            None => "Bonus".into(),
        },
        "UnlockUIElement" | "UnlockContract" | "None" => "—".into(),
        other => other.into(),
    }
}

fn pretty_branch(b: &str) -> &str {
    match b {
        "Engineering" => "Engineering",
        "Biotech" => "Biotech",
        "Physics" => "Physics",
        other => other,
    }
}

fn page_research(locale: &Locale, sirenix: &Sirenix) -> String {
    let name_for = |id: &str| -> String {
        for r in &locale.research {
            if r.id == id {
                return r.name.clone();
            }
        }
        id.to_string()
    };
    let desc_for = |id: &str| -> String {
        for r in &locale.research {
            if r.id == id {
                return r.description.clone();
            }
        }
        String::new()
    };

    let facility_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let spacecraft_name: BTreeMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    let lv_name: BTreeMap<&str, &str> = locale
        .launch_vehicles
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();

    // Mirror page_facilities' filter so we only emit cross-page links when an
    // anchor will actually be rendered there.  Orbital descriptors (e.g. crew
    // / engine modules) and FacilitySegments are skipped by the facilities
    // page, so linking them would 404.
    let facility_anchored: std::collections::BTreeSet<&str> = sirenix
        .facilities
        .iter()
        .filter(|f| !f.is_obsolete)
        .filter(|f| f.facility_type != "FacilitySegment")
        .filter(|f| f.descriptor != "Orbital")
        .map(|f| f.id.strip_prefix("build_").unwrap_or(&f.id))
        .filter(|id| {
            locale
                .facilities
                .iter()
                .any(|lf| lf.id == *id && !lf.name.is_empty())
        })
        .collect();

    // Every research node a player can interact with goes on the page.  `showInTree`
    // is the game's in-tree-header flag, not a "should-this-be-public" flag.
    let visible: Vec<&ResearchStat> = sirenix.research.iter().collect();

    // Humanize CamelCase sub-branch ids the same way the corp comparison
    // does — `LaunchVehicle` → `Launch Vehicle` — so the H2 headers below
    // match the player-facing tech-tree section names.
    let humanize_sub = |sb: &str| -> String {
        let mut out = String::with_capacity(sb.len() + 4);
        for (i, c) in sb.char_indices() {
            if i > 0 && c.is_uppercase() { out.push(' '); }
            out.push(c);
        }
        out
    };

    // Bucket directly by sub-branch.  The game's tech-tree UI doesn't show
    // the top-level branch (Engineering/Physics/Biotech) division — it
    // displays the sub-branches as the top-level categories — so the wiki
    // mirrors that.  Items from the same sub-branch across different
    // top-level branches (e.g. Spacecraft research that sits under both
    // Engineering and Biotech) merge into a single section.
    let mut by_sub: BTreeMap<String, Vec<&ResearchStat>> = BTreeMap::new();
    for r in &visible {
        if r.subbranch.is_empty() { continue; }
        by_sub
            .entry(humanize_sub(&r.subbranch))
            .or_default()
            .push(*r);
    }

    let mut out = String::from(
        "# Research\n\n\
The tech tree drives progression. Every research node has a work-hours cost,\n\
zero or more prerequisite research nodes, and unlocks something — a new\n\
facility, spacecraft, launch vehicle, or a numeric bonus on existing\n\
equipment. Sections below match the sub-branches the game shows in the\n\
research tree (Computing, Chemical Propulsion, Spacecraft, …).\n\n",
    );

    {
        for (sub, items) in &by_sub {
            out.push_str(&format!("## {}\n\n", sub));
            {
            let mut items_sorted = items.clone();
            items_sorted.sort_by(|a, b| {
                a.work_hours
                    .partial_cmp(&b.work_hours)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.id.cmp(&b.id))
            });
            let rows: Vec<Vec<String>> = items_sorted
                .iter()
                .map(|r| {
                    let display = name_for(&r.id);
                    let prereqs = if r.prereqs.is_empty() {
                        "—".to_string()
                    } else {
                        r.prereqs
                            .iter()
                            .map(|p| link_same_page("research", p, &escape_cell(&name_for(p))))
                            .collect::<Vec<_>>()
                            .join("<br>")
                    };
                    let name_cell = format!(
                        "{anchor}**{name}**",
                        anchor = anchor_tag("research", &r.id),
                        name = escape_cell(&display)
                    );
                    vec![
                        name_cell,
                        fmt_work_hours(r.work_hours),
                        era_label(r.stage).to_string(),
                        prereqs,
                        fmt_research_unlock(r, &facility_name, &facility_anchored, &spacecraft_name, &lv_name),
                        escape_cell(&desc_for(&r.id)),
                    ]
                })
                .collect();
            out.push_str(&md_table_with_tips(
                &["Research", "Cost (h)", "Era", "Prereqs", "Unlocks", "Description"],
                &[
                    None,
                    Some("Cost in work-hours; divide by your labs' research output to get the actual research time in days"),
                    Some("Tech tree era — broad progression tier of this research."),
                    None,
                    None,
                    None,
                ],
                &rows,
            ));
            out.push('\n');
            }
        }
    }

    out.push_str(
        "## Reading the table\n\n\
- **Cost** is in work hours and is divided by your laboratories' research output to get the actual research time in days.\n\
- **Era** is the broad tech-tree tier: *Early* / *Mid* / *Late*. The tree gates entire eras behind milestone nodes, so most early choices are independent and later choices depend on a long chain of early ones.\n\
- **Prerequisites** must be completed before the node becomes available.\n\
- **Unlocks** — *Builds X* means the node makes a new facility constructable; *Spacecraft / Launch Vehicle* means the node unlocks a new ship or lifter; numeric bonuses apply to listed components. A research node can stack multiple unlocks — the primary unlock comes first, then any additional bonuses or facilities/spacecraft/launch vehicles it also unlocks appear on follow-on lines in the same cell.\n\n\
## See also\n\n\
- [Spacecraft](../spacecraft/) — propulsion research feeds directly into these\n\
- [Launch Vehicles](../launch-vehicles/)\n",
    );
    out
}

/// Derive a player-facing achievement name from its `id_achievement_<CamelCase>`
/// id.  No locale entries exist for achievements in `en-US.csv`, so we
/// best-effort humanize the id by:
///   1. stripping the `id_achievement_` prefix;
///   2. inserting a space at every `lowercase → Uppercase` boundary.
///
/// A small explicit override table fixes up the handful of ids where the
/// game concatenated lowercase connector words (`of`, `on`, `a`) with the
/// preceding capitalized word — pure heuristic splitting on those produced
/// false positives ("Beg in Terraforming", "Moonbase Alph a"), so we hand-pin
/// the known offenders instead.  When the dump adds new ids the heuristic
/// continues to apply and the wiki gracefully degrades to mildly-imperfect
/// names rather than mangled ones.
///
/// If the id doesn't start with the expected prefix, return it unchanged
/// — we'd rather show a slightly-ugly id than silently lose information.
fn humanize_achievement_id(id: &str) -> String {
    let core = id.strip_prefix("id_achievement_").unwrap_or(id);
    if core == id {
        // Caller passed something that didn't carry our prefix; preserve it
        // verbatim rather than emit an empty string.
        return id.to_string();
    }
    // Hand-curated overrides for ids where CamelCase splitting alone produces
    // awkward results (lowercase connector words concatenated to the
    // preceding word).  Keys are the *core* — i.e. the id with the
    // `id_achievement_` prefix already stripped.
    match core {
        "HumansonMars" => return "Humans on Mars".to_string(),
        "OnWindsofSunshine" => return "On Winds of Sunshine".to_string(),
        "FancyWayofThrowingRocks" => return "Fancy Way of Throwing Rocks".to_string(),
        "ThePowerofaStar" => return "The Power of a Star".to_string(),
        "DoAstronautsDreamofElectricShip" => {
            return "Do Astronauts Dream of Electric Ship".to_string()
        }
        _ => {}
    }
    // Fallback: insert spaces at lowercase→Uppercase boundaries.
    let mut out = String::with_capacity(core.len() + 4);
    let mut prev: Option<char> = None;
    for c in core.chars() {
        if let Some(p) = prev {
            if p.is_lowercase() && c.is_uppercase() {
                out.push(' ');
            }
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

/// Steam achievements page.  Renders one section per source_type
/// (`contract` / `spacecraft` / `launch_vehicle`), each with a stat table
/// keyed on (achievement name, link to its trigger row, optional condition).
///
/// Source-name resolution is locale-first:
///   * contract → `locale.contracts[].name`
///   * spacecraft → `locale.spacecraft[].name`
///   * launch_vehicle → `locale.launch_vehicles[].name`
/// If the locale lookup fails (very rare — every id in the dump is in the
/// locale) we fall back to the raw source id, but we never emit a raw
/// `id_achievement_*` for the achievement column itself: the locale's
/// `achievement_*` keys don't exist, so the renderer always uses
/// `humanize_achievement_id` unless the upstream parser stamped a real
/// `name` on the `AchievementStat`.
///
/// The "How to earn" / "Trigger" cell is a markdown link to the trigger's
/// row on its dedicated page (../contracts/, ../spacecraft/, etc.) so a
/// reader can jump straight to the contract description or spacecraft
/// stats.
///
/// The Condition column carries year deadlines and required prior
/// contracts parsed from each binding's `conditions[]` array.  It only
/// renders for source types that actually carry conditions in the dump:
/// contracts.  Spacecraft and LV tables omit it entirely because every
/// such binding has an empty `conditions[]`.
///
/// Sections with zero rows are omitted entirely (no point in showing
/// "## By launch vehicle" if no LV currently binds an achievement).
fn page_achievements(locale: &Locale, sirenix: &Sirenix) -> String {
    let contract_name: BTreeMap<&str, &str> = locale
        .contracts
        .iter()
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();
    let sc_name: BTreeMap<&str, &str> = locale
        .spacecraft
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();
    let lv_name: BTreeMap<&str, &str> = locale
        .launch_vehicles
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();

    // The player-facing achievement name: prefer the locale-derived `name`
    // on the AchievementStat if non-empty, otherwise humanize the id.
    let render_name = |a: &AchievementStat| -> String {
        if !a.name.is_empty() {
            a.name.clone()
        } else {
            humanize_achievement_id(&a.id)
        }
    };

    // Resolve the source's player-facing name; fall back to the raw id
    // only as a last resort (and log it in the rendered cell so a wiki
    // reader can still recognize it).
    let resolve_name = |source_type: &str, source_id: &str| -> String {
        let name = match source_type {
            "contract" => contract_name.get(source_id).copied(),
            "spacecraft" => sc_name.get(source_id).copied(),
            "launch_vehicle" => lv_name.get(source_id).copied(),
            _ => None,
        };
        match name.filter(|s| !s.is_empty()) {
            Some(n) => n.to_string(),
            None => source_id.to_string(),
        }
    };

    // Build the "how to earn / trigger" cell as a markdown link to the
    // source row on its dedicated page.
    let render_source_link = |source_type: &str, source_id: &str, label: &str| -> String {
        let display = format!("{} ({})", resolve_name(source_type, source_id), label);
        let escaped = escape_cell(&display);
        match source_type {
            "contract" => link_cross_page("contracts", "contract", source_id, &escaped),
            "spacecraft" => link_cross_page("spacecraft", "spacecraft", source_id, &escaped),
            "launch_vehicle" => link_cross_page("launch-vehicles", "lv", source_id, &escaped),
            _ => escaped,
        }
    };

    // Render the `conditions[]` array as a human-readable cell.  Year
    // deadlines become "By year 2400"; required prior contracts become
    // markdown links to that contract's row on /contracts/.  Multiple
    // conditions on a single achievement join with "<br>".
    let render_conditions = |conds: &[AchievementConditionStat]| -> String {
        let bits: Vec<String> = conds
            .iter()
            .filter_map(|c| {
                if !c.required_contract.is_empty() {
                    let pretty = resolve_name("contract", &c.required_contract);
                    let link = link_cross_page(
                        "contracts",
                        "contract",
                        &c.required_contract,
                        &escape_cell(&pretty),
                    );
                    Some(format!("After {link}"))
                } else if c.before_year > 0 {
                    Some(format!("By year {}", c.before_year))
                } else {
                    None
                }
            })
            .collect();
        if bits.is_empty() {
            "—".to_string()
        } else {
            bits.join("<br>")
        }
    };

    let mut out = String::new();
    out.push_str(
        "# Achievements\n\n\
Steam achievements available in **Solar Expanse**, with how to earn each.\n\
Sourced from the game's `ContractDefinition` and `SpacecraftType` tables —\n\
every binding here corresponds to an in-game trigger that awards the\n\
achievement.  Each contract / spacecraft name links to its row on the\n\
relevant page.\n\n",
    );

    // A section renders either two columns (Achievement + Trigger) or three
    // (+ Condition) depending on `with_conditions`.  Spacecraft / LV bind
    // unconditional achievements in the dump, so they get the 2-col layout.
    let section = |out: &mut String,
                   header: &str,
                   label: &str,
                   filter_type: &str,
                   trigger_header: &str,
                   with_conditions: bool| {
        let rows: Vec<Vec<String>> = sirenix
            .achievements
            .iter()
            .filter(|a| a.source_type == filter_type)
            .map(|a| {
                let how = render_source_link(&a.source_type, &a.source_id, label);
                let mut row = vec![escape_cell(&render_name(a)), how];
                if with_conditions {
                    row.push(render_conditions(&a.conditions));
                }
                row
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        out.push_str(&format!("## {header}\n\n"));
        let headers: &[&str] = if with_conditions {
            &["Achievement", trigger_header, "Condition"]
        } else {
            &["Achievement", trigger_header]
        };
        out.push_str(&md_table(headers, &rows));
        out.push('\n');
    };

    section(
        &mut out,
        "By contract",
        "contract",
        "contract",
        "How to earn (contract)",
        true,
    );
    section(
        &mut out,
        "By spacecraft",
        "spacecraft",
        "spacecraft",
        "Trigger spacecraft",
        false,
    );
    section(
        &mut out,
        "By launch vehicle",
        "launch vehicle",
        "launch_vehicle",
        "Trigger launch vehicle",
        false,
    );

    out.push_str(
        "## Notes\n\n\
- Some contracts bind more than one achievement (e.g. *Interstellar 2* awards\n  both *To Infinity* and *Wanderlust* — the latter only if completed before\n  the year 2400).  Each binding appears as its own row.\n\
- Spacecraft-bound achievements typically fire the first time you operate a\n  craft of that propulsion class — building, fueling, or launching one\n  depending on the achievement.\n\
- The Condition column lists the extra requirements parsed from each\n  binding's `conditions[]` array — typically a year deadline or a\n  prerequisite contract.  \"—\" means the achievement fires the moment the\n  parent contract is completed, with no further constraint.\n\
- Achievement names are derived from the in-game id when no localized\n  display name is available; the in-game UI may polish the wording further.\n\n\
## See also\n\n\
- [Contracts](../contracts/) — full contract list and dependency chain\n\
- [Spacecraft](../spacecraft/)\n\
- [Launch Vehicles](../launch-vehicles/)\n",
    );
    out
}

fn page_missions(_locale: &Locale, _sirenix: &Sirenix) -> String {
    // The Missions page is a *Plan Mission* primer: destination →
    // spacecraft → cargo → launch vehicle → flight plan, plus the
    // in-game mission types. The contracts list lives on its own page
    // (/contracts/) — this page links there but does not duplicate it.
    String::from(
        "# Missions\n\n\
This page covers two related concepts, both of which the game calls\n\
\"missions\" depending on context.\n\n\
1. **Contracts** — the in-game *Contracts* tab. See [Contracts](../contracts/)\n\
   for the full list and dependency chain.\n\
2. **Flight missions** — an individual scheduled trip you plan in Plan\n\
   Mission (Earth → Mars on day N).  Flight missions are runtime state,\n\
   not static data — see the **planning flow** section below for how to\n\
   set one up.\n\n\
## Planning flow\n\n\
Plan Mission walks you through five steps:\n\n\
1. **Destination** — pick the target body (and landing type if applicable).\n\
2. **Spacecraft** — pick the craft to send.\n\
3. **Cargo** — pick the payload.\n\
4. **Launch Vehicle** — pick the lifter (only required for missions launching from a planet's surface).\n\
5. **Flight Plan** — pick the launch and arrival windows from the porkchop plot.\n\n\
### Mission types (from in-game UI)\n\n\
| Type | Notes |\n\
| --- | --- |\n\
| **Direct** | Single Hohmann-style transfer to the destination. |\n\
| **Gravity Assist** | Uses another body's gravity to bend the trajectory and save Δv. The game lets you choose whether cargo drops at the assist target or continues on. |\n\
| **Cyclical** | A repeating supply route between two or more bodies. |\n\
| **Asteroid Pulling** | Specialised mission to push an asteroid into a different orbit using an Asteroid Engine Module. |\n\
| **Probe Deployment** | Drops a small probe at a destination (typically the first thing you send anywhere). |\n\n\
For launch-window timing for any destination, see [Launch Windows](../celestial-bodies/launch-windows.md).\n\n\
## See also\n\n\
- [Contracts](../contracts/)\n\
- [Spacecraft](../spacecraft/)\n\
- [Launch Vehicles](../launch-vehicles/)\n\
- [Launch Windows](../celestial-bodies/launch-windows.md)\n"
    )
}

/// Map a habitability field's raw scenario-start value to a player-facing
/// formatted string. Wraps `fmt_opt` for the numeric formatting choices
/// (decimals shown vary by field magnitude).
fn fmt_habit(v: f64, places: usize) -> String {
    if !v.is_finite() {
        "—".to_string()
    } else if v == 0.0 {
        "0".to_string()
    } else {
        format!("{v:.places$}")
    }
}

/// Per-body initial-habitability snapshot page. One section per resolved
/// planet/moon (asteroids and other numerically-keyed bodies are filtered
/// out — they don't appear in `PlanetarySystemDescriptor.tabObjectInfoData`
/// and showing a wall of "186" rows isn't useful).
///
/// For each body we emit a comparison table — one row per scenario — with
/// the player-facing habitability columns. This makes it easy to see, for
/// example, that Mars has roughly the same temperature in all three
/// populated scenarios but its water level varies as terraforming progresses
/// across the timeline.
///
/// Early Exploration's StartGameData (testStartGAme) doesn't carry a
/// populated `habitabilityParameters` block in the current dump, so its
/// row reads "data unavailable" rather than zeros.
fn page_scenario_state(sirenix: &Sirenix) -> String {
    if sirenix.scenario_starts.is_empty() {
        return "# Initial habitability per scenario\n\n_No scenario data available._\n".into();
    }

    // Numeric body ids that aren't in the tabObjectInfoData mapping
    // (asteroids, dwarf-planet placeholders) end up with body_name equal to
    // the stringified id. Filter those out — the named planet/moon list is
    // what players want to compare.
    let is_resolved_name =
        |b: &ScenarioBodyHabitabilityStat| -> bool { b.body_name.parse::<i32>().is_err() };

    // Stable ordering for the Sol-system bodies: planets in distance order,
    // then their moons grouped after the parent. We reuse the existing
    // PLANETS + moons_by_parent definitions for canonical order.
    let mut ordered_bodies: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let push = |name: &str,
                out: &mut Vec<String>,
                seen: &mut std::collections::HashSet<String>| {
        if seen.insert(name.to_string()) {
            out.push(name.to_string());
        }
    };
    let moons = moons_by_parent();
    for planet in PLANETS {
        push(planet, &mut ordered_bodies, &mut seen);
        if let Some((_, ms)) = moons.iter().find(|(p, _)| p == planet) {
            for m in ms {
                push(m, &mut ordered_bodies, &mut seen);
            }
        }
    }

    // Find which bodies actually appear in *any* scenario's snapshot. Some
    // bodies (e.g. moons of the outer planets that aren't loaded in the
    // earlier-timeline scenes) may be absent from some scenarios.
    let mut present: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for s in &sirenix.scenario_starts {
        for b in &s.body_habitability {
            if is_resolved_name(b) {
                present.insert(b.body_name.clone());
            }
        }
    }

    // Build the per-body sections. For each body, render one table comparing
    // all four scenarios.
    let scenario_order = [
        "StartGameEpoch_EarlyExploration",
        "StartGameEpoch_TheExpansion",
        "StartGameEpoch_Colonization",
        "StartGameEpoch_RaceBeyond",
    ];

    let mut out = String::from(
        "# Initial habitability per scenario\n\n\
Every Solar Expanse scenario ships with a pre-built save that pins every\n\
body's starting environmental state. This page lays those values out side\n\
by side so the four start dates can be compared at a glance — useful for\n\
spotting how much of Mars's water has already been delivered by the time\n\
the *Colonization Era* scenario opens, or how Venus's temperature has\n\
budged across the timeline.\n\n\
**Reading the tables.** Each row is one scenario. Values are pulled\n\
directly from the StartGameData's `ObjectInfoSaves[].habitabilityParameters`\n\
block — the same values the game reads at scenario load. Units (inferred\n\
from the well-known planets):\n\n\
| Column | Unit |\n\
| --- | --- |\n\
| Temperature | °C |\n\
| Pressure | Earth atmospheres |\n\
| Gravity | m/s² |\n\
| Water | 0–1 fraction |\n\
| Radiation | game-specific scale (Earth ≈ 1) |\n\
| Magnetic field | game-specific scale (Earth ≈ 40) |\n\
| Albedo | 0–1 surface reflectivity |\n\
| Composition | 0–1 atmospheric composition score |\n\
| Day–night ΔT | °C |\n\n\
*Note: the Early Exploration save (testStartGAme in the dump) doesn't carry\n\
a populated habitabilityParameters block, so its row reads \"—\" across the\n\
board.*\n\n",
    );

    for body_name in &ordered_bodies {
        if !present.contains(body_name) {
            continue;
        }
        // Find this body's snapshot in each scenario.
        let rows: Vec<Vec<String>> = scenario_order
            .iter()
            .filter_map(|epoch_id| {
                let scenario = sirenix
                    .scenario_starts
                    .iter()
                    .find(|s| s.scenario_id == *epoch_id)?;
                let label = epoch_display_name(epoch_id);
                let body = scenario
                    .body_habitability
                    .iter()
                    .find(|b| &b.body_name == body_name);
                match body {
                    Some(b) => Some(vec![
                        format!("**{label}**"),
                        fmt_habit(b.temperature, 1),
                        fmt_habit(b.pressure, 3),
                        fmt_habit(b.gravity, 2),
                        fmt_habit(b.water, 3),
                        fmt_habit(b.radiation, 2),
                        fmt_habit(b.magnetic_field, 2),
                        fmt_habit(b.albedo, 2),
                        fmt_habit(b.composition, 3),
                        fmt_habit(b.temperature_swings, 1),
                    ]),
                    None => Some(vec![
                        format!("**{label}**"),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                    ]),
                }
            })
            .collect();
        out.push_str(&format!("## {body_name}\n\n"));
        out.push_str(&md_table(
            &[
                "Scenario",
                "Temp (°C)",
                "Pressure",
                "Gravity",
                "Water",
                "Radiation",
                "Magnetic",
                "Albedo",
                "Composition",
                "Day–night ΔT",
            ],
            &rows,
        ));
        out.push('\n');
    }

    out.push_str("## See also\n\n");
    out.push_str("- [Celestial Bodies overview](README.md)\n");
    out.push_str("- [Planets](planets.md)\n");
    out.push_str("- [Moons](moons.md)\n");
    out
}

fn page_root() -> String {
    String::from(
        "# Solar Expanse Wiki\n\n\
A player-facing reference for **[Solar Expanse](https://store.steampowered.com/app/1369700/)** —\n\
the realistic solar-system management game by Maciej Miąsik / TJ Entertainment.\n\n\
This wiki is built from the game's own localization files and asset bundles, so\n\
the names, descriptions, and stat tables here match exactly what you see in-game.\n\n\
## Contents\n\n\
| Section | What's in it |\n\
| --- | --- |\n\
| **[Celestial Bodies](celestial-bodies/)** | The Sun, planets, moons, asteroids, comets, and exoplanet systems. |\n\
| [Exoplanets](exoplanets/) | Trappist-1, Kepler-90, Tau Ceti, and Proxima Centauri — the four destination systems reachable via a generation ship. |\n\
| [Spacecraft](spacecraft/) | Interplanetary craft — Iris, Selene, Stratos, Hermes, Centaur, Athena, Prometheus, Hephaistos, Ariane, Cronos, Nike, Sirius, Zeus. |\n\
| [Launch Vehicles](launch-vehicles/) | Surface-to-orbit lifters — Albatross, Pelican, Magpie, Condor, Teratorn. |\n\
| [Facilities](facilities/) | Ground buildings and orbital modules — power, mining, refining, habitats, life support, etc. |\n\
| [Research](research/) | Tech tree — chemical, electric, nuclear, fusion propulsion, life support, materials, computing. |\n\
| [Missions](missions/) | Mission planning — Plan Mission walk-through, mission types, launch-window pointer. |\n\
| [Contracts](contracts/) | Story and freelance contracts — the in-game Contracts tab — that drive progression. |\n\
| [Achievements](achievements/) | Steam achievements and how to earn each — keyed to contracts, spacecraft, and launch vehicles. |\n\
| [Resources](resources/) | The 20+ resource types — water, metals, fissiles, He-3, supplies, exotic alloys. |\n\
| [Asteroid Taxonomy](asteroid-taxonomy/) | The five asteroid classes (Carbon, Dark, Helium-3, Metal, Stone) and the per-class resource roll table the game uses when you mine a deposit. |\n\
| [Terraforming](terraforming/) | Per-resource thermal / phase constants — boiling and melting points, latent heat, heat capacity, optical depth — that drive the atmosphere sim. |\n\
| [Corporations](corporations/) | Playable starting factions — SoleX, NASA, ESA, CNSA, Roscosmos. |\n\n\
## How to use this wiki\n\n\
- **Find data fast.** Bodies (planets, moons, asteroids, comets, exoplanets) live under [Celestial Bodies](celestial-bodies/) — radius, semi-major axis, eccentricity, inclination, parent. Fleet planning lives under [Spacecraft](spacecraft/) and [Launch Vehicles](launch-vehicles/) — dry mass, cargo, fuel, thrust, exhaust velocity, build cost. What-to-build prompts and the workforce / energy / resource math behind each structure are on [Facilities](facilities/). The tech tree — costs, prereqs, and what each node unlocks — is on [Research](research/).\n\
- **Plan progression.** [Contracts](contracts/) is the in-game contracts tab, ordered by their root tree, with rewards and follow-on links. [Missions](missions/) walks the Plan Mission flow and points at launch-window data. [Achievements](achievements/) lists every Steam achievement keyed to the contract, spacecraft, or launch vehicle that earns it.\n\
- **Compare scenario starts.** [Corporations](corporations/) is a side-by-side table of the playable factions — starting cash, starting research, starting fleet, starting facilities — so you can pick the run you want.\n\
- **Understand the economy.** [Resources](resources/) lists every resource (water, metals, fissiles, He-3, supplies, exotic alloys), what produces it, what consumes it, and per-body mining license fees.  [Asteroid Taxonomy](asteroid-taxonomy/) shows the resource roll table for each of the five asteroid classes (Carbon, Dark, Helium-3, Metal, Stone) so you know what mining a given asteroid will yield.\n\
- **Tables are sortable.** Click any column header to sort by that column; click again to reverse.  Hover a column header for a tooltip explaining its units or source data.\n\
- **Calculator.** Several pages embed a small Calculator that computes a fleet's total payload and crew capacity for trip planning — change the inputs and the totals update live.\n\n\
## Contributing\n\n\
Almost every page is generated from the game's own files; direct edits get\n\
overwritten when the pipeline reruns. Fixes belong in the [generator code](https://github.com/stockmaj/solar-expanse-wiki/tree/main/extract).\n\
See [CONTRIBUTING](CONTRIBUTING.md) for details.\n\n\
## Credits\n\n\
- **Solar Expanse** © Maciej Miąsik / TJ Entertainment.\n\
- Wiki text is generated from the game's English localization and is presented\n\
  here for reference purposes only.\n",
    )
}

/// Humanize a `planet_*` taxonomy id from the game's `GeneratedPlanetType`
/// table into a player-friendly label. Examples:
///   `planet_rocky_volcanic` → `"Rocky volcanic"`
///   `planet_gas_gasgiant`   → `"Gas giant"`
///   `planet_gas_ice`        → `"Gas-ice giant"` (terminology consistent with the genre)
///   `planet_rocky_eyeballHot` → `"Rocky eyeball hot"`
/// The locale.json file has no friendlier mapping for these ids, so the
/// renderer derives one structurally: strip the `planet_` prefix, split on
/// underscores, split camelCase tokens (eyeballHot, desertCold), then
/// lowercase everything and capitalize only the first word. The
/// awkward `gas_gasgiant` compound is collapsed up-front to a single
/// "gas giant" before tokenization.
fn humanize_planet_type(id: &str) -> String {
    let core = id.strip_prefix("planet_").unwrap_or(id);
    // Collapse the redundant `gas_gasgiant` pair to a single "gas giant"
    // — the dump has both `gas_gasgiant` and `gas_ice` under the `planet_gas_*`
    // family, and the former reads doubly when split naively.
    let core = core.replace("gas_gasgiant", "gas giant");
    let mut words: Vec<String> = Vec::new();
    for raw in core.split(|c: char| c == '_' || c == ' ') {
        if raw.is_empty() { continue; }
        for w in split_camel(raw).split_whitespace() {
            words.push(w.to_ascii_lowercase());
        }
    }
    // Capitalize the first word only — these are short tag-style labels, not titles.
    if let Some(first) = words.first_mut() {
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            *first = c.to_ascii_uppercase().to_string() + chars.as_str();
        }
    }
    words.join(" ")
}

/// Split a camelCase token into space-separated lowercase words. Used to
/// turn `eyeballHot` → `"eyeball Hot"` so `humanize_planet_type` can re-join
/// after lowercasing.
fn split_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_ascii_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Render the exoplanet systems page. One H2 per system with a body-data
/// table; pulls exclusively from `sirenix.exoplanet_systems` so the page is
/// data-first and survives `git checkout HEAD -- docs/celestial-bodies/`
/// (which restores the Sol bodies but leaves this fresh file alone).
fn page_exoplanets_systems(sirenix: &Sirenix) -> String {
    if sirenix.exoplanet_systems.is_empty() {
        return "# Exoplanet Systems\n\n\
            Exoplanet system data is not available — re-run the extraction pipeline against a current sirenix-dump.json.\n"
            .into();
    }

    let mut out = String::new();
    out.push_str("# Exoplanet Systems\n\n");
    out.push_str(
        "Four star systems reachable only through interstellar travel, via a\n\
         generation ship constructed in the late game. Each section below lists\n\
         every body the game generates in the destination system, with its mass,\n\
         radius, and orbital elements around the host star.\n\n\
         _Semi-major axis here is measured around the host star, not the Sun._\n\n",
    );

    for sys in &sirenix.exoplanet_systems {
        let star_label = match &sys.second_star_type {
            Some(s2) => format!("{} + {}", sys.star_type, s2),
            None => sys.star_type.clone(),
        };
        out.push_str(&format!(
            "## {name}\n\n\
             **Host star:** {star} · **System age:** {age} · **Bodies:** {count}\n\n",
            name = sys.name,
            star = star_label,
            age = sys.system_age,
            count = sys.bodies.len(),
        ));
        let rows: Vec<Vec<String>> = sys
            .bodies
            .iter()
            .map(|b| {
                vec![
                    format!("**{}**", b.name),
                    humanize_planet_type(&b.planet_type),
                    format!("{:.4}", b.semi_major_axis_au),
                    format!("{:.4}", b.eccentricity),
                    format!("{:.2}", b.inclination_deg),
                    fmt_mass(Some(b.mass_1e24_kg)),
                    fmt_radius(Some(b.radius_km)),
                ]
            })
            .collect();
        let table = md_table(
            &[
                "Body",
                "Type",
                "Semi-major axis (AU)",
                "Eccentricity",
                "Inclination (°)",
                "Mass (×10²⁴ kg)",
                "Radius (km)",
            ],
            &rows,
        );
        out.push_str(&table);
        out.push('\n');
    }

    out.push_str(
        "## See also\n\n\
         - [Celestial Bodies overview](../celestial-bodies/) — Sol's planets, moons, asteroids, comets.\n\
         - [Planets](../celestial-bodies/planets.md) — the nine Sol planets.\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md_table_renders_header_separator_and_rows() {
        let table = md_table(
            &["Col A", "Col B"],
            &[
                vec!["a1".into(), "b1".into()],
                vec!["a2".into(), "b2".into()],
            ],
        );
        assert_eq!(
            table,
            "| Col A | Col B |\n| --- | --- |\n| a1 | b1 |\n| a2 | b2 |\n"
        );
    }

    #[test]
    fn fmt_mass_picks_precision_by_magnitude() {
        assert_eq!(fmt_mass(None), "—");
        assert_eq!(fmt_mass(Some(0.0)), "—");
        assert_eq!(fmt_mass(Some(1.06e-7)), "1.06e-7");
        assert_eq!(fmt_mass(Some(5.972)), "5.972");
        assert_eq!(fmt_mass(Some(1898.1)), "1898.1");
    }

    #[test]
    fn fmt_radius_rounds_large_and_keeps_decimals_for_small() {
        assert_eq!(fmt_radius(None), "—");
        assert_eq!(fmt_radius(Some(0.5)), "0.5");
        assert_eq!(fmt_radius(Some(99.4)), "99.4");
        assert_eq!(fmt_radius(Some(6378.14)), "6378");
    }

    #[test]
    fn moon_distance_km_divides_orbit_universal_by_1000() {
        // Moon at 2.57 OrbitUniversal units ⇒ 0.00257 AU ⇒ ~384,400 km
        let moon = Body {
            name: "Moon".into(),
            parent: Some("Earth".into()),
            mass_1e24_kg: Some(0.07342),
            radius_km: None,
            semi_major_axis_au: Some(2.57),
            eccentricity: Some(0.0),
            inclination_deg: Some(0.0),
            perihelion_au: Some(2.57),
            longitude_deg: None,
            omega_lc_deg: None,
            omega_uc_deg: None,
            body_type: None,
            orbit_data_source: Some("OrbitUniversal".into()),
            asteroid_class: None,
        };
        let km = moon_distance_km(&moon).unwrap();
        let expected = 2.57 * AU_IN_KM / 1000.0;
        assert!((km - expected).abs() < 0.01, "got {km}, expected {expected}");
        // sanity-check the real Moon is at ~384,400 km
        assert!((km - 384_400.0).abs() < 200.0, "got {km}");
    }

    #[test]
    fn moon_distance_km_does_not_scale_solar_body_values() {
        let planet = Body {
            name: "Mercury".into(),
            parent: None,
            mass_1e24_kg: Some(0.33),
            radius_km: Some(2440.0),
            semi_major_axis_au: Some(0.387),
            eccentricity: Some(0.2056),
            inclination_deg: Some(7.0),
            perihelion_au: None,
            longitude_deg: None,
            omega_lc_deg: None,
            omega_uc_deg: None,
            body_type: Some(0),
            orbit_data_source: Some("SolarBody".into()),
            asteroid_class: None,
        };
        // 0.387 AU is ~57.9M km when not scaled
        let km = moon_distance_km(&planet).unwrap();
        assert!((km - 0.387 * AU_IN_KM).abs() < 1.0);
    }

    #[test]
    fn escape_cell_escapes_pipes_and_collapses_newlines() {
        assert_eq!(escape_cell("a | b\nc"), "a \\| b c");
    }

    #[test]
    fn fmt_abbrev_handles_k_m_b_with_clean_rounding() {
        assert_eq!(fmt_abbrev(0.0), "0");
        assert_eq!(fmt_abbrev(50.0), "50");
        assert_eq!(fmt_abbrev(999.0), "999");
        assert_eq!(fmt_abbrev(6000.0), "6k");
        assert_eq!(fmt_abbrev(6500.0), "6.5k");
        assert_eq!(fmt_abbrev(750_000.0), "750k");
        assert_eq!(fmt_abbrev(1_000_000.0), "1M");
        assert_eq!(fmt_abbrev(20_000_000.0), "20M");
        assert_eq!(fmt_abbrev(1_898_100.0), "1.9M");
        assert_eq!(fmt_abbrev(1_000_000_000.0), "1B");
    }

    fn fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![
                CelestialBody { id: "Mercury".into(), name: "Mercury".into() },
                CelestialBody { id: "Ceres".into(), name: "1 Ceres".into() },
                CelestialBody { id: "Wild 2".into(), name: "81P Wild ".into() },
            ],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    fn fixture_body(name: &str) -> Body {
        Body {
            name: name.into(),
            parent: None,
            mass_1e24_kg: None,
            radius_km: None,
            semi_major_axis_au: None,
            eccentricity: None,
            inclination_deg: None,
            perihelion_au: None,
            longitude_deg: None,
            omega_lc_deg: None,
            omega_uc_deg: None,
            body_type: None,
            orbit_data_source: None,
            asteroid_class: None,
        }
    }

    #[test]
    fn body_lookup_resolves_taxonomy_id_via_display_name() {
        let locale = fixture_locale();
        let stats = Stats {
            bodies: vec![
                fixture_body("Mercury"),
                fixture_body("1 Ceres"),
                fixture_body("81P Wild "),
            ],
        };
        let ctx = WikiCtx::build(&locale, &stats);

        // Direct match works
        assert_eq!(ctx.body("Mercury").unwrap().name, "Mercury");
        // Asteroid: taxonomy id "Ceres" → display "1 Ceres" → matches GO "1 Ceres"
        assert_eq!(ctx.body("Ceres").unwrap().name, "1 Ceres");
        // Comet: taxonomy id "Wild 2" → display "81P Wild " (trailing space) → trimmed match
        assert_eq!(ctx.body("Wild 2").unwrap().name, "81P Wild ");
    }

    // ---------- Audit fix #2: contracts page never leaks raw internal IDs ----------

    fn contracts_fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![
                NameDesc { id: "spacecraft_chem_small".into(), name: "Iris".into(), description: String::new() },
                NameDesc { id: "spacecraft_chem_large".into(), name: "Stratos".into(), description: String::new() },
                NameDesc { id: "spacecraft_electric_small".into(), name: "Hermes".into(), description: String::new() },
            ],
            launch_vehicles: vec![
                NameDesc { id: "lv_chem_seadragon".into(), name: "Albatross".into(), description: String::new() },
                NameDesc { id: "lv_chemadvanced".into(), name: "Condor".into(), description: String::new() },
            ],
            research: vec![
                ResearchEntry { id: "research_sc_helios".into(), category: "sc".into(), name: "Stratos".into(), description: String::new() },
                ResearchEntry { id: "research_sc_iris".into(), category: "sc".into(), name: "Iris".into(), description: String::new() },
                ResearchEntry { id: "research_sc_hermes".into(), category: "sc".into(), name: "Hermes".into(), description: String::new() },
            ],
            corporations: vec![],
            contracts: vec![
                NameDesc {
                    id: "contract_tutorial_moonlanding".into(),
                    name: "Lunar Landing".into(),
                    description: "It's time for a grand return to the moon.".into(),
                },
                NameDesc {
                    // Carries the `_test` substring; mirrors production where the
                    // locale name is empty for tutorial-test contracts.
                    id: "contract_tutorial_moonlandingMultiModuleDeliverTest".into(),
                    name: String::new(),
                    description: String::new(),
                },
                NameDesc {
                    id: "contract_mars_terraform_water".into(),
                    name: "Blue Mars".into(),
                    description: "Bringing enough water to Mars will make it possible for many organisms to thrive and help regulate temperature. Doing it will require a large-scale operation, but with enough persistence and good planning, we'll wipe the name Red Planet\" from books.\"".into(),
                },
                NameDesc {
                    id: "contract_general_fleet".into(),
                    name: "Fleet Expansion".into(),
                    description: "We need more ships.".into(),
                },
                NameDesc {
                    id: "contract_downstream".into(),
                    name: "Downstream Contract".into(),
                    description: "Downstream of the test contract.".into(),
                },
                NameDesc {
                    id: "contract_tutorial_firstorbit".into(),
                    name: "First Orbit".into(),
                    description: "First orbit.".into(),
                },
                NameDesc {
                    id: "contract_tutorial_moonorbit".into(),
                    name: "Explore Luna".into(),
                    description: "Explore Luna.".into(),
                },
                NameDesc {
                    id: "contract_tutorial_marsorbit".into(),
                    name: "Explore Mars".into(),
                    description: "Explore Mars.".into(),
                },
                NameDesc {
                    id: "contract_tutorial_spacedock".into(),
                    name: "Space Dock".into(),
                    description: "Space Dock.".into(),
                },
                NameDesc {
                    id: "contract_asteroid_sample".into(),
                    name: "Asteroid Sample".into(),
                    description: "Sample asteroid.".into(),
                },
                NameDesc {
                    id: "contract_asteroid_mining".into(),
                    name: "Asteroid Mining".into(),
                    description: "Mine asteroid.".into(),
                },
                NameDesc {
                    id: "contract_multi_parent".into(),
                    name: "Multi Parent".into(),
                    description: "Reached from two roots.".into(),
                },
                NameDesc {
                    id: "contract_root_a".into(),
                    name: "Root Alpha".into(),
                    description: "Non-tutorial root A.".into(),
                },
                NameDesc {
                    id: "contract_root_b".into(),
                    name: "Root Beta".into(),
                    description: "Non-tutorial root B.".into(),
                },
            ],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    fn make_contract(id: &str, objectives: Vec<ContractObjectiveStat>, unlock_rewards: Vec<String>) -> ContractStat {
        ContractStat {
            id: id.into(),
            is_locked: false,
            is_final: false,
            objectives,
            money_reward: 0.0,
            unlock_rewards,
            facility_grants: vec![],
            spacecraft_grants: vec![],
            launch_vehicle_grants: vec![],
            resource_grants: vec![],
            date_start_active: None,
            date_time_string_start: None,
            years_to_expire: 0.0,
            objective_layers: vec![],
            has_layer_none_objective: false,
        }
    }

    fn make_contract_with_layers(
        id: &str,
        unlock_rewards: Vec<String>,
        objective_layers: Vec<String>,
    ) -> ContractStat {
        ContractStat {
            id: id.into(),
            is_locked: false,
            is_final: false,
            objectives: vec![],
            money_reward: 0.0,
            unlock_rewards,
            facility_grants: vec![],
            spacecraft_grants: vec![],
            launch_vehicle_grants: vec![],
            resource_grants: vec![],
            date_start_active: None,
            date_time_string_start: None,
            years_to_expire: 0.0,
            objective_layers,
            has_layer_none_objective: false,
        }
    }

    fn make_contract_with_none_layer(
        id: &str,
        unlock_rewards: Vec<String>,
        objective_layers: Vec<String>,
    ) -> ContractStat {
        let mut c = make_contract_with_layers(id, unlock_rewards, objective_layers);
        c.has_layer_none_objective = true;
        c
    }

    fn obj(kind: &str, quantity: f64, target: Option<&str>) -> ContractObjectiveStat {
        ContractObjectiveStat {
            kind: kind.into(),
            quantity,
            target: target.map(|s| s.into()),
        }
    }

    #[test]
    fn make_research_renders_research_display_name_no_quantity() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_tutorial_moonlanding",
                vec![obj("MakeResearch", 0.0, Some("research_sc_helios"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(page.contains("Stratos"), "page should mention research display name:\n{page}");
        assert!(!page.contains("research_sc_helios"), "raw research id leaked:\n{page}");
        assert!(!page.contains("0×"), "zero-quantity leaked:\n{page}");
        assert!(!page.contains("MakeResearch"), "raw kind leaked:\n{page}");
    }

    #[test]
    fn create_spacecraft_renders_spacecraft_display_name() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_tutorial_moonlanding",
                vec![obj("CreateSpaceCraft", 0.0, Some("Spacecraft3Helios"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            !page.contains("Spacecraft3Helios"),
            "raw spacecraft objective id leaked:\n{page}"
        );
        assert!(!page.contains("0×"), "zero-quantity leaked:\n{page}");
        assert!(
            page.contains("Helios") || page.contains("Stratos"),
            "spacecraft display name missing:\n{page}"
        );
    }

    #[test]
    fn create_vehicle_renders_launch_vehicle_display_name() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_tutorial_moonlanding",
                vec![obj("CreateVehicle", 0.0, Some("lv_chem_superlarge"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            !page.contains("lv_chem_superlarge"),
            "raw lv objective id leaked:\n{page}"
        );
        assert!(!page.contains("0×"), "zero-quantity leaked:\n{page}");
        assert!(
            page.contains("Launch Vehicle") || page.contains("Chemical") || page.contains("Albatross"),
            "no launch-vehicle display name:\n{page}"
        );
    }

    #[test]
    fn prereq_column_omits_or_renames_test_contracts() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_moonlandingMultiModuleDeliverTest",
                    vec![],
                    vec!["contract_downstream".into()],
                ),
                make_contract("contract_downstream", vec![], vec![]),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            !page.contains("moonlandingMultiModuleDeliverTest"),
            "raw _test id leaked into prereq column:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Downstream Contract"))
            .expect("Downstream Contract row present");
        assert!(
            !row.contains("[](#"),
            "empty-name prereq link leaked: {row}"
        );
    }

    #[test]
    fn premise_truncation_does_not_end_mid_quoted_string() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract("contract_mars_terraform_water", vec![], vec![])],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            !page.contains("Red Planet\" fr"),
            "premise truncates mid-word after stray closing quote:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Blue Mars"))
            .expect("Blue Mars row present");
        let quote_count = row.chars().filter(|c| *c == '"').count();
        assert_eq!(
            quote_count % 2,
            0,
            "row has unbalanced quotes: {row}"
        );
    }

    #[test]
    fn possession_with_no_target_renders_spacecraft_label() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_general_fleet",
                vec![obj("Possession", 10.0, None)],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Fleet Expansion"))
            .expect("Fleet Expansion row present");
        assert!(
            row.contains("Spacecraft"),
            "Possess-without-target should label as Spacecraft: {row}"
        );
    }

    #[test]
    fn deliver_module_target_renders_friendly_label() {
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_tutorial_moonlanding",
                vec![obj("Deliver", 1.0, Some("module_crew_compartment"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            page.contains("Crew Compartment"),
            "module target should render Title Case:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Lunar Landing"))
            .expect("row present");
        assert!(
            !row.contains("× crew compartment"),
            "raw lowercased module target leaked: {row}"
        );
    }

    // ---------- Page-polish fixes ----------

    /// Build a launch-windows fixture context with Earth, Mars, and Cruithne.
    /// Cruithne's semi-major axis (0.998 AU) is the textbook near-resonance
    /// case where the synodic-period formula blows up to a ~294-year interval.
    fn launch_windows_fixture() -> (Locale, Stats) {
        let locale = Locale {
            celestial_bodies: vec![
                CelestialBody { id: "Earth".into(), name: "Earth".into() },
                CelestialBody { id: "Mars".into(), name: "Mars".into() },
                CelestialBody { id: "Cruithne".into(), name: "Cruithne".into() },
            ],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let make = |name: &str, a: f64| Body {
            name: name.into(),
            parent: None,
            mass_1e24_kg: None,
            radius_km: None,
            semi_major_axis_au: Some(a),
            eccentricity: Some(0.0),
            inclination_deg: Some(0.0),
            perihelion_au: None,
            longitude_deg: Some(0.0),
            omega_lc_deg: None,
            omega_uc_deg: None,
            body_type: None,
            orbit_data_source: Some("SolarBody".into()),
            asteroid_class: None,
        };
        let stats = Stats {
            bodies: vec![
                make("Earth", 1.0),
                make("Mars", 1.524),
                make("Cruithne", 0.998),
            ],
        };
        (locale, stats)
    }

    #[test]
    fn launch_windows_flags_near_resonance_bodies() {
        let (locale, stats) = launch_windows_fixture();
        let ctx = WikiCtx::build(&locale, &stats);
        let page = page_launch_windows(&ctx);
        let cruithne_row = page
            .lines()
            .find(|l| l.contains("**Cruithne**"))
            .expect("Cruithne row present in launch-windows table");
        assert!(
            cruithne_row.contains("(near-resonance"),
            "Cruithne row should carry the near-resonance marker: {cruithne_row}"
        );
        let mars_row = page
            .lines()
            .find(|l| l.contains("**Mars**"))
            .expect("Mars row present");
        assert!(
            !mars_row.contains("near-resonance"),
            "Mars row should not be flagged: {mars_row}"
        );
        // The Practical reading bullet must explain the artifact.
        assert!(
            page.contains("near-resonance"),
            "Practical reading should reference the near-resonance artifact"
        );
    }

    // (Legacy `exoplanet_row_renders_five_columns_for_populated_body` removed:
    // the Bodies-nav URL now delegates to page_exoplanets_systems, whose own
    // tests cover the rendered shape.)

    // ---------- Navigation fixes: cross-page links wherever they apply ----------

    fn nav_fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![
                NameDesc { id: "spacecraft_chem_small".into(), name: "Iris".into(), description: "Small chemical craft.".into() },
            ],
            launch_vehicles: vec![],
            research: vec![
                ResearchEntry { id: "research_sc_iris".into(), category: "sc".into(), name: "Iris".into(), description: String::new() },
                ResearchEntry { id: "research_lifesup_1".into(), category: "lifesup".into(), name: "Crewed Flight".into(), description: String::new() },
            ],
            corporations: vec![],
            contracts: vec![
                NameDesc {
                    id: "contract_root".into(),
                    name: "Root Contract".into(),
                    description: "Root.".into(),
                },
                NameDesc {
                    id: "contract_asteroid_mining".into(),
                    name: "Asteroid Mining".into(),
                    description: "Follow-up.".into(),
                },
            ],
            resources: vec![
                ResourceEntry { id: "energy".into(), name: "Energy".into() },
                ResourceEntry { id: "energy_Description".into(), name: "The lifeblood of modern infrastructure.".into() },
            ],
            facilities: vec![
                Facility {
                    id: "geothermal".into(),
                    name: "Geothermal Power".into(),
                    description: "Produces Energy from underground heat.".into(),
                },
            ],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    fn make_sc_stat(id: &str, built_in_orbit: bool) -> SpacecraftStat {
        SpacecraftStat {
            id: id.into(),
            engine_module: None,
            engine_type: "chemical".into(),
            mass: 1.0,
            cargo_capacity: 2.0,
            fuel_capacity: 20.0,
            reusability: 0.0,
            needs_launch_vehicle: false,
            built_in_orbit,
            can_be_built_by_player: true,
            build_cost: vec![],
            build_time_days: 30.0,
            launch_cost: 1000.0,
        }
    }

    fn make_research(id: &str, action: &str, target: Option<&str>) -> ResearchStat {
        ResearchStat {
            id: id.into(),
            work_hours: 1_000_000.0,
            branch: "Engineering".into(),
            subbranch: "Spacecraft".into(),
            prereqs: vec![],
            action: action.into(),
            unlock_target: target.map(|s| s.into()),
            bonus_kind: None,
            bonus_amount: 0.0,
            bonus_components: vec![],
            show_in_tree: false,
            contract_unlocks: vec![],
            stage: 0,
            secondary_unlocks: vec![],
        }
    }

    fn make_resource_stat(id: &str, resource_type: &str) -> ResourceStat {
        ResourceStat {
            id: id.into(),
            resource_type: resource_type.into(),
            market_price_base: 11.5,
            show_on_ui: true,
            can_be_left_on_object: true,
            terraformation_info: None,
        }
    }

    #[test]
    fn resources_page_links_producers_to_facility_anchors() {
        // Energy is produced by Geothermal Power — the producers cell should
        // render a markdown link to the facility's anchor on the facilities page.
        // Source of truth is the structured `produces` field, not tooltip text.
        let locale = nav_fixture_locale();
        let mut geo = facility_stat("geothermal", "Power");
        geo.produces = vec![ResourceCost {
            resource_id: "energy".into(),
            amount: 400.0,
        }];
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("energy", "Energy")],
            facilities: vec![geo],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        assert!(
            page.contains("[Geothermal Power](../facilities/#facility-geothermal)"),
            "producers cell should link facility name to facilities-page anchor:\n{page}"
        );
    }

    #[test]
    fn resources_page_emits_anchor_per_row() {
        // Other pages (e.g. facilities) link back to resources; each row needs
        // a stable inline anchor so those links land somewhere.
        let locale = nav_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("energy", "Energy")],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        assert!(
            page.contains("<a id=\"resource-energy\"></a>"),
            "resource row should carry an inline anchor tag:\n{page}"
        );
    }

    /// Locale fixture for the resources-page Producers/Consumers/icon tests.
    /// Adds an `alloy` resource entry plus a polymer-production facility so
    /// the structured-data lookups have something to find.
    fn resources_fixture_locale() -> Locale {
        let mut locale = nav_fixture_locale();
        locale.resources.push(ResourceEntry {
            id: "alloy".into(),
            name: "Alloy".into(),
        });
        locale.resources.push(ResourceEntry {
            id: "alloy_Description".into(),
            name: "Standard structural alloy.".into(),
        });
        locale.facilities.push(Facility {
            id: "polymerproduction".into(),
            name: "Polymers Factory".into(),
            description: "Manufactures polymers from carbon and alloy stock.".into(),
        });
        locale
    }

    #[test]
    fn resource_row_includes_icon_image_tag() {
        // Each resource cell should lead with the resource icon (mirrors the
        // icon + name pattern used by `fmt_build_cost`).
        let locale = resources_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        assert!(
            page.contains("<img src=\"../images/resources/alloy.png\""),
            "Alloy row should render the resource icon image:\n{page}"
        );
    }

    #[test]
    fn resource_row_consumers_column_lists_consumer_facilities() {
        // A facility whose structured `consumes` includes a resource id must
        // show up in that resource's Consumers cell.
        let locale = resources_fixture_locale();
        let mut polymers = facility_stat("polymerproduction", "Production");
        polymers.consumes = vec![ResourceCost {
            resource_id: "alloy".into(),
            amount: 0.1,
        }];
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            facilities: vec![polymers],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        let alloy_row = page
            .lines()
            .find(|l| l.contains("resource-alloy"))
            .expect("Alloy row present:\n");
        assert!(
            alloy_row.contains("Polymers Factory"),
            "Alloy row's Consumers cell should list Polymers Factory:\n{alloy_row}"
        );
        assert!(
            alloy_row.contains("../facilities/#facility-polymerproduction"),
            "Consumer entry should link to the facility's anchor on the facilities page:\n{alloy_row}"
        );
    }

    #[test]
    fn resource_row_consumers_column_renders_dash_when_no_consumer() {
        // A resource consumed by no facility shows a dash in the Consumers cell.
        let locale = resources_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            facilities: vec![],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        // The page must declare a Consumers header (so we can validate column ordering).
        assert!(
            page.contains("Consumers"),
            "page should expose a Consumers column header:\n{page}"
        );
        let alloy_row = page
            .lines()
            .find(|l| l.contains("resource-alloy"))
            .expect("Alloy row present:\n");
        // Resource | Type | License (Earth) | Market base | Producers | Consumers | Description
        let cells: Vec<&str> = alloy_row.split('|').collect();
        // Pipe-split rows have leading + trailing empty cells: ["", " Resource ", ..., ""].
        // With seven data columns we expect 9 entries.
        assert_eq!(
            cells.len(),
            9,
            "row should have seven columns (Resource, Type, License (Earth), Market base, Producers, Consumers, Description):\n{alloy_row}"
        );
        // Consumers is column 6 (1-indexed data columns: 1=Resource, 2=Type,
        // 3=License, 4=Market, 5=Producers, 6=Consumers, 7=Description), so
        // cells[6] in the split (since cells[0] is the leading empty).
        let consumers_cell = cells[6].trim();
        assert_eq!(
            consumers_cell, "—",
            "Consumers cell should render an em-dash when nothing consumes the resource:\n{alloy_row}"
        );
    }

    #[test]
    fn resources_page_keeps_market_base_column_alongside_license() {
        // Both columns coexist: Market base is the global clearing-price
        // anchor (`marketClearingPriceBase`), License (Earth) is the
        // per-tonne licensing fee Earth charges. They're different signals.
        let locale = resources_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        assert!(
            page.contains("Market base ($/t)"),
            "page should still expose the 'Market base ($/t)' header:\n{page}"
        );
        assert!(
            page.contains("License (Earth, $/t)"),
            "page should expose the new 'License (Earth, $/t)' header:\n{page}"
        );
    }

    #[test]
    fn resources_page_renders_earth_license_fee_for_matching_resource() {
        // When the BepInEx mod has run and populated license_fees with an
        // Earth entry, the License (Earth, $/t) column should show the fee
        // for any resource Earth charges for. The fee comes from the
        // body whose body_name == "Earth"; lookups are by resource id.
        let locale = resources_fixture_locale();
        let mut fees: BTreeMap<String, f64> = BTreeMap::new();
        fees.insert("alloy".to_string(), 30.0);
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            license_fees: vec![BodyLicenseFeeStat {
                body_name: "Earth".to_string(),
                fees_per_t: fees,
            }],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        let alloy_row = page
            .lines()
            .find(|l| l.contains("resource-alloy"))
            .expect("Alloy row present:\n");
        let cells: Vec<&str> = alloy_row.split('|').collect();
        // License (Earth) is data column 3 → cells[3] after the leading empty.
        let license_cell = cells[3].trim();
        assert_eq!(
            license_cell, "30",
            "License (Earth) cell should render Earth's fee for alloy:\n{alloy_row}"
        );
    }

    #[test]
    fn resources_page_renders_dash_for_non_earth_resource() {
        // Earth doesn't charge a license fee for every resource. Resources
        // not listed in Earth's fees_per_t map render as em-dash, NOT as
        // zero, so the column distinguishes "no charge" from "$0".
        let locale = resources_fixture_locale();
        let mut fees: BTreeMap<String, f64> = BTreeMap::new();
        fees.insert("steel".to_string(), 25.0);
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            license_fees: vec![BodyLicenseFeeStat {
                body_name: "Earth".to_string(),
                fees_per_t: fees,
            }],
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        let alloy_row = page
            .lines()
            .find(|l| l.contains("resource-alloy"))
            .expect("Alloy row present:\n");
        let cells: Vec<&str> = alloy_row.split('|').collect();
        let license_cell = cells[3].trim();
        assert_eq!(
            license_cell, "—",
            "License (Earth) should be em-dash for resources Earth doesn't fee:\n{alloy_row}"
        );
    }

    #[test]
    fn resources_page_license_column_renders_dashes_when_no_dump_data() {
        // When the dump predates the mod rebuild (license_fees is empty),
        // every row's License (Earth) cell must show em-dash. The page
        // must NOT panic — the column gracefully degrades.
        let locale = resources_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![make_resource_stat("alloy", "Normal")],
            // license_fees deliberately omitted → empty Vec.
            ..Default::default()
        };
        let page = page_resources(&locale, &sirenix);
        assert!(
            page.contains("License (Earth, $/t)"),
            "License column header should be present even with empty data:\n{page}"
        );
        let alloy_row = page
            .lines()
            .find(|l| l.contains("resource-alloy"))
            .expect("Alloy row present:\n");
        let cells: Vec<&str> = alloy_row.split('|').collect();
        let license_cell = cells[3].trim();
        assert_eq!(
            license_cell, "—",
            "License (Earth) cell should fall back to em-dash when license_fees is empty:\n{alloy_row}"
        );
    }

    // ---------- Terraforming page ----------

    fn water_ti() -> TerraformationInfoStat {
        // Real values from the Sirenix dump for water.
        TerraformationInfoStat {
            optical_depth_parameter: 0.002,
            heat_capacity: 1860.0,
            vaporization_latent_heat: 50000.0,
            boiling_temperature_k: 373.0,
            melting_temperature_k: 220.0,
            pressure_triple_point: 0.00611,
        }
    }

    #[test]
    fn terraforming_page_renders_water_row_with_thermal_constants() {
        // Water's thermal / phase constants should show up verbatim with
        // a kelvin-and-celsius pair for the phase-change columns and a
        // human-readable name from the locale.
        let mut locale = resources_fixture_locale();
        locale.resources.push(ResourceEntry {
            id: "water".into(),
            name: "Water".into(),
        });
        let mut water = make_resource_stat("water", "Normal");
        water.terraformation_info = Some(water_ti());
        let sirenix = Sirenix {
            resources: vec![water],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        // Header is present.
        assert!(page.starts_with("# Terraforming"), "page must start with title:\n{page}");
        // Water row anchor + display name.
        let water_row = page
            .lines()
            .find(|l| l.contains("terraforming-water"))
            .expect("Water row present:\n");
        assert!(water_row.contains("Water"), "row must use locale display name:\n{water_row}");
        // Kelvin temperatures appear, with celsius pair (373 K → ~100 °C,
        // 220 K → ~-53 °C). We assert on the kelvin substring and the °C
        // pair so the row is unambiguously the temperature row.
        assert!(
            water_row.contains("373") && water_row.contains("100"),
            "boiling K and °C must both appear (373 K / 100 °C):\n{water_row}"
        );
        assert!(
            water_row.contains("220") && water_row.contains("-53"),
            "melting K and °C must both appear (220 K / -53 °C):\n{water_row}"
        );
        // Latent heat, heat capacity, optical depth, triple-point pressure
        // all surface.
        assert!(water_row.contains("50000"), "latent heat 50000:\n{water_row}");
        assert!(water_row.contains("1860"), "heat capacity 1860:\n{water_row}");
        assert!(water_row.contains("0.002"), "optical depth 0.002:\n{water_row}");
        assert!(water_row.contains("0.00611"), "triple-point pressure 0.00611:\n{water_row}");
    }

    #[test]
    fn terraforming_page_skips_resources_without_thermal_info() {
        // Resources whose terraformation_info is None (energy, human,
        // supplies, …) must not show up on the page. Otherwise the page
        // would clutter with all-1.0 placeholder rows.
        let locale = resources_fixture_locale();
        let sirenix = Sirenix {
            resources: vec![
                make_resource_stat("energy", "Energy"), // no TI
                make_resource_stat("alloy", "Normal"),  // no TI either
            ],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        assert!(
            !page.contains("terraforming-energy"),
            "energy row must not appear when TI is None:\n{page}"
        );
        assert!(
            !page.contains("terraforming-alloy"),
            "alloy row must not appear when TI is None:\n{page}"
        );
    }

    #[test]
    fn terraforming_page_orders_resources_alphabetically_by_display_name() {
        // Two resources with TI: hydrogen and water. Hydrogen sorts before
        // Water alphabetically, so its row should appear first on the page.
        let mut locale = resources_fixture_locale();
        locale.resources.push(ResourceEntry { id: "hydrogen".into(), name: "Hydrogen".into() });
        locale.resources.push(ResourceEntry { id: "water".into(), name: "Water".into() });
        let mut hydrogen = make_resource_stat("hydrogen", "Normal");
        hydrogen.terraformation_info = Some(TerraformationInfoStat {
            optical_depth_parameter: 0.0001,
            heat_capacity: 14320.0,
            vaporization_latent_heat: 449.0,
            boiling_temperature_k: 20.0,
            melting_temperature_k: 14.0,
            pressure_triple_point: 0.0695,
        });
        let mut water = make_resource_stat("water", "Normal");
        water.terraformation_info = Some(water_ti());
        let sirenix = Sirenix {
            // Insert water first to prove the page sorts independently of dump order.
            resources: vec![water, hydrogen],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        let h_idx = page.find("terraforming-hydrogen").expect("hydrogen row");
        let w_idx = page.find("terraforming-water").expect("water row");
        assert!(h_idx < w_idx, "Hydrogen must sort before Water in the table");
    }

    #[test]
    fn terraforming_page_emits_reading_the_table_section() {
        // Player-facing column descriptions live in a "Reading the table"
        // section below the data — same convention as page_resources.
        let locale = resources_fixture_locale();
        let mut water = make_resource_stat("water", "Normal");
        water.terraformation_info = Some(water_ti());
        let sirenix = Sirenix {
            resources: vec![water],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        assert!(
            page.contains("## Reading the table"),
            "missing reading section:\n{page}"
        );
        assert!(
            page.contains("Optical depth"),
            "missing optical-depth explanation:\n{page}"
        );
        assert!(
            page.contains("Heat capacity"),
            "missing heat-capacity explanation:\n{page}"
        );
    }

    #[test]
    fn terraforming_page_cross_links_to_resources_page() {
        // The terraforming page should point readers back to the per-resource
        // page so they can find facilities that produce / consume each species.
        let locale = resources_fixture_locale();
        let mut water = make_resource_stat("water", "Normal");
        water.terraformation_info = Some(water_ti());
        let sirenix = Sirenix {
            resources: vec![water],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        assert!(
            page.contains("../resources/"),
            "page must link back to the resources page:\n{page}"
        );
    }

    #[test]
    fn terraforming_page_surfaces_facility_with_habitability_deltas() {
        // A facility carrying habitability_deltas should appear on the
        // terraforming page in a dedicated "Terraforming facilities" section,
        // with a row per parameter delta (or a single multi-line row).
        let mut locale = facility_fixture_locale();
        // Also include the resource set the page is otherwise built from so
        // the resource thermal table can still render.
        locale
            .resources
            .push(ResourceEntry { id: "water".into(), name: "Water".into() });
        let mut mag = facility_stat("build_terraform_magnet", "Terraformation");
        mag.habitability_deltas = vec![
            ("Radiation".into(), -0.6),
            ("Magnetic field".into(), 0.6),
        ];
        let sirenix = Sirenix {
            facilities: vec![mag],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        // Section header is present.
        assert!(
            page.contains("## Terraforming facilities"),
            "page must include a Terraforming facilities section:\n{page}"
        );
        // The facility's player-facing display name should appear in the new section.
        let after_header = page
            .split("## Terraforming facilities")
            .nth(1)
            .expect("section header present");
        assert!(
            after_header.contains("Magnetic Field Generator"),
            "magnet facility must appear in the new section:\n{after_header}"
        );
        // Both labels and signed magnitudes for its deltas must be rendered.
        assert!(
            after_header.contains("Radiation"),
            "section should mention Radiation delta:\n{after_header}"
        );
        assert!(
            after_header.contains("Magnetic field"),
            "section should mention Magnetic field delta:\n{after_header}"
        );
        // Magnitudes appear with proper sign — Unicode minus for the negative.
        assert!(
            after_header.contains("0.6"),
            "delta magnitude 0.6 should appear:\n{after_header}"
        );
        assert!(
            after_header.contains("−0.6") || after_header.contains("-0.6"),
            "negative Radiation delta should be signed:\n{after_header}"
        );
        // The facility name should link back to its row on the facilities page.
        // anchor_id slugifies the id, so `terraform_magnet` → `terraform-magnet`.
        assert!(
            after_header.contains("../facilities/#facility-terraform-magnet"),
            "facility name should link to its anchor on the facilities page:\n{after_header}"
        );
    }

    #[test]
    fn terraforming_page_skips_facilities_with_no_deltas() {
        // Facilities with empty (or all-zero) habitability_deltas should not
        // appear in the Terraforming facilities section.
        let locale = facility_fixture_locale();
        let f_empty = facility_stat("build_habitat", "Habitation");
        let mut f_zero = facility_stat("build_lab", "Other");
        f_zero.habitability_deltas = vec![("Radiation".into(), 0.0)];
        let sirenix = Sirenix {
            facilities: vec![f_empty, f_zero],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        let after_header = page
            .split("## Terraforming facilities")
            .nth(1)
            .unwrap_or("");
        assert!(
            !after_header.contains("**Habitat**"),
            "habitat (no deltas) should not appear:\n{after_header}"
        );
        assert!(
            !after_header.contains("**Research Lab**"),
            "facility with only zero-magnitude deltas should not appear:\n{after_header}"
        );
    }

    #[test]
    fn terraforming_page_orders_facilities_alphabetically() {
        // Two facilities with deltas — alphabetical by display name. We seed
        // a "Atmospheric Greenhouse" facility before "Magnetic Field Generator"
        // in the dump but the page should show Atmospheric first.
        let mut locale = facility_fixture_locale();
        locale.facilities.push(Facility {
            id: "terraform_atmosphere".into(),
            name: "Atmospheric Greenhouse".into(),
            description: "Builds atmosphere.".into(),
        });
        let mut mag = facility_stat("build_terraform_magnet", "Terraformation");
        mag.habitability_deltas = vec![("Magnetic field".into(), 0.6)];
        let mut atmo = facility_stat("build_terraform_atmosphere", "Terraformation");
        atmo.habitability_deltas = vec![("Pressure".into(), 0.001)];
        let sirenix = Sirenix {
            // Insert magnet first so the page must sort.
            facilities: vec![mag, atmo],
            ..Default::default()
        };
        let page = page_terraforming(&locale, &sirenix);
        let after_header = page
            .split("## Terraforming facilities")
            .nth(1)
            .expect("section present");
        let atmo_idx = after_header
            .find("Atmospheric Greenhouse")
            .expect("atmosphere row");
        let mag_idx = after_header
            .find("Magnetic Field Generator")
            .expect("magnet row");
        assert!(
            atmo_idx < mag_idx,
            "Atmospheric Greenhouse must sort before Magnetic Field Generator:\n{after_header}"
        );
    }

    #[test]
    fn contracts_next_chain_renders_as_link() {
        // "Next: <ContractName>" in the Rewards cell should be a markdown link
        // to the follow-up contract's same-page anchor.
        let locale = nav_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_root",
                    vec![],
                    vec!["contract_asteroid_mining".into()],
                ),
                make_contract("contract_asteroid_mining", vec![], vec![]),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            page.contains("Next: [Asteroid Mining](#contract-contract-asteroid-mining)"),
            "Rewards `Next:` should be a same-page link:\n{page}"
        );
    }

    #[test]
    fn corporations_page_pulls_traits_from_locale_struct() {
        let locale = Locale {
            celestial_bodies: vec![],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![Corporation {
                id: "sentinel_corp".into(),
                name: "Sentinel Industries".into(),
                description: "Watches the inner system.".into(),
                traits: "● Sentinel trait line".into(),
            }],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let sirenix = Sirenix::default();
        let page = page_corporations(&locale, &sirenix);
        assert!(
            page.contains("● Sentinel trait line"),
            "page should render the locale's traits string verbatim:\n{page}"
        );
        assert!(
            page.contains("Sentinel Industries"),
            "page should render the corp name from locale:\n{page}"
        );
    }

    #[test]
    fn spacecraft_page_links_to_research_unlock() {
        // Spacecraft row should include a cross-page link to the research that
        // unlocks the craft.  `research_sc_iris` unlocks `spacecraft_chem_small`.
        let locale = nav_fixture_locale();
        let sirenix = Sirenix {
            spacecraft: vec![make_sc_stat("spacecraft_chem_small", false)],
            research: vec![make_research("research_sc_iris", "UnlockSpacecraftType", Some("spacecraft_chem_small"))],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Iris"))
            .expect("Iris row present:\n");
        assert!(
            row.contains("../research/#research-research-sc-iris"),
            "Iris row should link to its research-unlock anchor:\n{row}"
        );
    }

    #[test]
    fn spacecraft_page_has_requires_lv_column_header_and_tooltip() {
        // The Spacecraft page should expose a "Requires LV" column with
        // an explanatory tooltip — players need to know which craft are
        // Earth-launchable on their own (none) vs. surface-launchable
        // elsewhere (most non-chemical craft) vs. built in orbit.
        let locale = nav_fixture_locale();
        let sirenix = Sirenix {
            spacecraft: vec![make_sc_stat("spacecraft_chem_small", false)],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        assert!(
            page.contains("Requires LV"),
            "spacecraft page missing 'Requires LV' column header:\n{page}"
        );
        // Tooltip should reference Earth specifically because Earth is the
        // hard-coded `Company.mainObjectInfo` that always requires an LV.
        assert!(
            page.contains("Earth"),
            "Requires LV column tooltip should mention Earth:\n{page}"
        );
    }

    #[test]
    fn spacecraft_page_iris_row_says_any_body_when_needs_launch_vehicle() {
        // Iris (spacecraft_chem_small) has needs_launch_vehicle=true in the
        // dump — it needs an LV from *any* planet/moon, not just Earth.
        let locale = nav_fixture_locale();
        let mut iris = make_sc_stat("spacecraft_chem_small", false);
        iris.needs_launch_vehicle = true;
        let sirenix = Sirenix {
            spacecraft: vec![iris],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Iris"))
            .expect("Iris row present");
        assert!(
            row.contains("Any body") || row.contains("Required from any"),
            "Iris row should advertise LV required from any body:\n{row}"
        );
    }

    #[test]
    fn spacecraft_page_stratos_row_says_earth_only_when_self_launching() {
        // Stratos (spacecraft_chem_large) has needs_launch_vehicle=false in
        // the dump — it can self-launch from Luna/Mars/asteroids but Earth's
        // special-case still forces an LV.
        let mut locale = nav_fixture_locale();
        locale.spacecraft.push(NameDesc {
            id: "spacecraft_chem_large".into(),
            name: "Stratos".into(),
            description: "Large chemical craft.".into(),
        });
        let stratos = make_sc_stat("spacecraft_chem_large", false);
        // make_sc_stat already defaults needs_launch_vehicle=false; assert
        // the precondition for clarity.
        assert!(!stratos.needs_launch_vehicle);
        let sirenix = Sirenix {
            spacecraft: vec![stratos],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Stratos"))
            .expect("Stratos row present");
        assert!(
            row.contains("Earth only"),
            "Stratos row should advertise LV required from Earth only:\n{row}"
        );
    }

    #[test]
    fn spacecraft_page_spawned_orbital_payload_container_dashes_launch_vehicle() {
        // The Orbital Payload Container has empty build_cost AND
        // build_time_days == 0 — the game spawns it from a launch facility
        // (elevator / mass driver / spin launch / catapult) rather than
        // launching it conventionally, so the "Launch vehicle" column
        // shouldn't claim "Any body" / "Earth only" / etc.
        let mut locale = nav_fixture_locale();
        locale.spacecraft.push(NameDesc {
            id: "spacecraft_capsule".into(),
            name: "Orbital Payload Container".into(),
            description: "Spawned payload.".into(),
        });
        let mut opc = make_sc_stat("spacecraft_capsule", false);
        opc.needs_launch_vehicle = true; // dump value
        opc.build_cost = vec![]; // spawned, not built
        opc.build_time_days = 0.0;
        let sirenix = Sirenix {
            spacecraft: vec![opc],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Orbital Payload Container"))
            .expect("OPC row present");
        // Row format: | name | mass | cargo | fuel | thrust | exhaust | reusable | built_at | launch_vehicle | build_cost | time | desc |
        // built_at is "—" already; the LV column should follow suit.
        let cells: Vec<&str> = row.split('|').map(|c| c.trim()).collect();
        // index 9 is the 9th cell after the leading empty (Launch vehicle column)
        let lv_cell = cells.get(9).expect("LV cell present");
        assert_eq!(
            *lv_cell, "—",
            "spawned-not-built craft should dash the LV column; row:\n{row}"
        );
    }

    #[test]
    fn spacecraft_page_orbital_build_row_says_built_in_orbit() {
        // Orbital-build craft never sit on a planetary surface so the LV
        // column should communicate that, not parrot the underlying flag.
        let mut locale = nav_fixture_locale();
        locale.spacecraft.push(NameDesc {
            id: "spacecraft_fusion_large".into(),
            name: "Zeus".into(),
            description: "Orbital fusion ship.".into(),
        });
        let zeus = make_sc_stat("spacecraft_fusion_large", /* built_in_orbit */ true);
        let sirenix = Sirenix {
            spacecraft: vec![zeus],
            ..Default::default()
        };
        let page = page_spacecraft(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Zeus"))
            .expect("Zeus row present");
        assert!(
            row.contains("Built in orbit") || row.contains("Orbital build"),
            "orbital-build craft should declare 'Built in orbit' in the LV column:\n{row}"
        );
    }

    #[test]
    fn corporations_page_includes_epoch_timeline_section() {
        let locale = Locale {
            celestial_bodies: vec![],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let sirenix = Sirenix {
            epochs: vec![
                EpochStat {
                    id: "StartGameEpoch_Prelude".into(),
                    start_date_string: "01.01.1959 00:00:00".into(),
                    is_locked: false,
                    possible_player_companies: vec!["NASA".into()],
                },
                EpochStat {
                    id: "StartGameEpoch_EarlyExploration".into(),
                    start_date_string: "01.01.2000 00:00:00".into(),
                    is_locked: false,
                    possible_player_companies: vec!["NASA".into(), "ESA".into()],
                },
                EpochStat {
                    id: "StartGameEpoch_Colonization".into(),
                    start_date_string: "01.01.2100 00:00:00".into(),
                    is_locked: true,
                    possible_player_companies: vec!["NASA".into()],
                },
                EpochStat {
                    id: "StartGameEpoch_TheExpansion".into(),
                    start_date_string: "01.01.2200 00:00:00".into(),
                    is_locked: true,
                    possible_player_companies: vec!["NASA".into()],
                },
                EpochStat {
                    id: "StartGameEpoch_RaceBeyond".into(),
                    start_date_string: "01.01.2300 00:00:00".into(),
                    is_locked: true,
                    possible_player_companies: vec!["NASA".into()],
                },
            ],
            // scenario_starts drives the "playable in Sol-system" filter for
            // the epoch table — only epoch ids present here are rendered.
            scenario_starts: vec![
                ScenarioStartStat { scenario_id: "StartGameEpoch_EarlyExploration".into(), corp_starts: vec![], body_habitability: vec![] },
                ScenarioStartStat { scenario_id: "StartGameEpoch_TheExpansion".into(), corp_starts: vec![], body_habitability: vec![] },
                ScenarioStartStat { scenario_id: "StartGameEpoch_Colonization".into(), corp_starts: vec![], body_habitability: vec![] },
                ScenarioStartStat { scenario_id: "StartGameEpoch_RaceBeyond".into(), corp_starts: vec![], body_habitability: vec![] },
            ],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        // Timeline lists the four Sol-system epochs; Prelude is filtered out
        // because it isn't mapped in PlanetarySystem_Realistic (not playable
        // from the New Game menu).
        assert!(!page.contains("Prelude"), "Prelude should be filtered out — not playable in Sol-system\n{page}");
        assert!(page.contains("Early Exploration"), "missing Early Exploration:\n{page}");
        assert!(page.contains("Colonization Era"), "missing Colonization Era:\n{page}");
        assert!(page.contains("The Expansion"), "missing The Expansion:\n{page}");
        assert!(page.contains("Race Beyond"), "missing Race Beyond:\n{page}");
        // Internal id may appear in `value="..."` attributes (option tags
        // need a stable key) but must NOT leak into any visible cell.
        for line in page.lines() {
            if line.starts_with("| ") && line.contains("StartGameEpoch_") {
                panic!("internal epoch id leaked into a table cell: {line}");
            }
        }
        // Year column was dropped — shipped startDateString doesn't match
        // the years displayed in the game UI, so we don't surface it.
        assert!(
            !page.contains("| Start year |"),
            "Start year column should not render: data is unreliable"
        );
    }

    /// Shared fixture: five-corp locale matching the in-game customization
    /// roster, so corp ordering in the comparison view is deterministic.
    fn corp_compare_locale() -> Locale {
        let corp = |id: &str, name: &str| Corporation {
            id: id.into(),
            name: name.into(),
            description: format!("{name} description."),
            traits: String::new(),
        };
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![
                ResearchEntry { id: "research_chem_main1".into(), category: "chem".into(), name: "Kerolox".into(), description: String::new() },
                ResearchEntry { id: "research_chem_main2".into(), category: "chem".into(), name: "Hydrolox".into(), description: String::new() },
                ResearchEntry { id: "research_lv_main1".into(), category: "lv".into(), name: "Small LV".into(), description: String::new() },
                ResearchEntry { id: "research_sc_iris".into(), category: "sc".into(), name: "Iris SC".into(), description: String::new() },
            ],
            corporations: vec![
                corp("solex", "SoleX"),
                corp("nasa", "NASA"),
                corp("esa", "ESA"),
                corp("cnsa", "CNSA"),
                corp("roscosmos", "Roscosmos"),
            ],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    #[test]
    fn corporations_page_renders_all_four_epoch_scenario_labels() {
        // Once Early Exploration is a real scenario, the page must surface the
        // human-facing labels for all four pre-built saves — no internal ids,
        // and no stale "Early-Exploration scenarios are procedural" copy.
        let locale = corp_compare_locale();
        let mk = |epoch: &str, money: f64| ScenarioStartStat {
            scenario_id: epoch.into(),
            corp_starts: vec![CorpStartStat {
                company_id: "Solex".into(),
                starting_money: money,
                completed_research: vec!["research_chem_main1".into()],
                starting_launch_vehicles: 0,
                starting_spacecraft: 0,
                ..Default::default()
            }],
            body_habitability: vec![],
        };
        let sirenix = Sirenix {
            scenario_starts: vec![
                mk("StartGameEpoch_EarlyExploration", 5_000_000.0),
                mk("StartGameEpoch_TheExpansion", 27_200_000.0),
                mk("StartGameEpoch_Colonization", 33_700_000.0),
                mk("StartGameEpoch_RaceBeyond", 42_125_000.0),
            ],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        for label in ["Early Exploration", "The Expansion", "Colonization Era", "Race Beyond"] {
            assert!(page.contains(label), "missing {label}:\n{page}");
        }
        // Internal epoch ids may appear only inside the data-binding
        // (the <option value=…> attribute and the JS CORP_DATA blob), never
        // as bare text. Strip those out and confirm no leakage in prose.
        let prose: String = page
            .lines()
            .filter(|l| !l.contains("window.CORP_DATA")
                && !l.starts_with("<option ")
                && !l.contains("<select"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !prose.contains("StartGameEpoch_"),
            "internal epoch id leaked into player-facing prose:\n{prose}"
        );
        // The default-selected scenario must be The Expansion. Early
        // Exploration starts every corp with zero pre-built facilities, so
        // the Starting facilities section in the comparison table comes up
        // empty there. The Expansion is the first scenario where the
        // matrix has data to compare.
        assert!(
            page.contains("value=\"StartGameEpoch_TheExpansion\" selected"),
            "The Expansion should be the default-selected option:\n{page}"
        );
        // No other scenario should carry the selected attribute.
        for other in ["StartGameEpoch_EarlyExploration", "StartGameEpoch_Colonization", "StartGameEpoch_RaceBeyond"] {
            assert!(
                !page.contains(&format!("value=\"{other}\" selected")),
                "only The Expansion should be selected, but {other} is:\n{page}"
            );
        }
        // Scenario order in the dropdown must be Early → Expansion → Colonization → RaceBeyond.
        let pos = |needle: &str| page.find(needle).unwrap_or(usize::MAX);
        let p_early = pos("value=\"StartGameEpoch_EarlyExploration\"");
        let p_exp = pos("value=\"StartGameEpoch_TheExpansion\"");
        let p_col = pos("value=\"StartGameEpoch_Colonization\"");
        let p_race = pos("value=\"StartGameEpoch_RaceBeyond\"");
        assert!(p_early < p_exp && p_exp < p_col && p_col < p_race,
            "expected dropdown order Early < Expansion < Colonization < RaceBeyond, got {:?}",
            (p_early, p_exp, p_col, p_race));
    }

    #[test]
    fn corporations_page_shows_solex_early_exploration_starting_cash() {
        // The Early Exploration save (testStartGAme in the dump) starts SoleX
        // with $5M. That value must appear in the JSON blob the JS layer reads.
        let locale = corp_compare_locale();
        let sirenix = Sirenix {
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_EarlyExploration".into(),
                corp_starts: vec![CorpStartStat {
                    company_id: "Solex".into(),
                    starting_money: 5_000_000.0,
                    completed_research: vec!["research_chem_main1".into()],
                    starting_launch_vehicles: 0,
                    starting_spacecraft: 0,
                    ..Default::default()
                }],
                body_habitability: vec![],
            }],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        // The CORP_DATA JSON blob carries `starting_money` verbatim.
        assert!(
            page.contains("\"starting_money\":5000000"),
            "expected SoleX Early Exploration starting cash in CORP_DATA blob:\n{page}"
        );
    }

    #[test]
    fn solex_has_fewer_research_in_early_exploration_than_expansion() {
        // Pre-built saves get richer with each successive epoch. Early
        // Exploration should ship fewer completed research items than The
        // Expansion for the same corp — a sanity check that the new scenario
        // surfaces a smaller starting tree (and matches the in-game
        // progression curve).
        let locale = corp_compare_locale();
        let sirenix = Sirenix {
            scenario_starts: vec![
                ScenarioStartStat {
                    scenario_id: "StartGameEpoch_EarlyExploration".into(),
                    corp_starts: vec![CorpStartStat {
                        company_id: "Solex".into(),
                        starting_money: 5_000_000.0,
                        completed_research: vec!["research_chem_main1".into()],
                        starting_launch_vehicles: 0,
                        starting_spacecraft: 0,
                        ..Default::default()
                    }],
                    body_habitability: vec![],
                },
                ScenarioStartStat {
                    scenario_id: "StartGameEpoch_TheExpansion".into(),
                    corp_starts: vec![CorpStartStat {
                        company_id: "Solex".into(),
                        starting_money: 27_200_000.0,
                        completed_research: vec![
                            "research_chem_main1".into(),
                            "research_chem_main2".into(),
                            "research_lv_main1".into(),
                            "research_sc_iris".into(),
                        ],
                        starting_launch_vehicles: 2,
                        starting_spacecraft: 4,
                        ..Default::default()
                    }],
                    body_habitability: vec![],
                },
            ],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        // Parse the embedded JSON blob and walk its `scenarios` array so the
        // assertion isn't sensitive to serde_json's key ordering.
        let blob_pos = page.find("window.CORP_DATA = ").expect("CORP_DATA blob present");
        let blob_tail = &page[blob_pos + "window.CORP_DATA = ".len()..];
        let blob_end = blob_tail.find("};").expect("CORP_DATA blob ends with `};`");
        let blob_json = format!("{}{}", &blob_tail[..blob_end], "}");
        let parsed: serde_json::Value = serde_json::from_str(&blob_json)
            .unwrap_or_else(|e| panic!("CORP_DATA JSON parse failed: {e}\n{blob_json}"));
        let scenarios = parsed["scenarios"].as_array().expect("scenarios array");
        let find_corp_research_len = |epoch_id: &str| -> usize {
            let scen = scenarios
                .iter()
                .find(|s| s["id"].as_str() == Some(epoch_id))
                .unwrap_or_else(|| panic!("missing scenario {epoch_id} in {scenarios:?}"));
            let solex = scen["corps"]
                .as_array()
                .expect("corps array")
                .iter()
                .find(|c| c["name"].as_str() == Some("SoleX"))
                .expect("SoleX corp");
            solex["research"].as_array().expect("research array").len()
        };
        let early_n = find_corp_research_len("StartGameEpoch_EarlyExploration");
        let expansion_n = find_corp_research_len("StartGameEpoch_TheExpansion");
        assert!(
            early_n < expansion_n,
            "Early Exploration ({early_n}) should ship fewer SoleX research items than The Expansion ({expansion_n})"
        );
        assert_eq!(early_n, 1, "Early Exploration SoleX should have 1 research");
        assert_eq!(expansion_n, 4, "The Expansion SoleX should have 4 research");
    }

    #[test]
    fn corporations_corp_data_research_entries_carry_category_objects() {
        // Each completed-research entry in the CORP_DATA blob must be a
        // `{name, category}` object — not a bare string — so the JS renderer
        // can group rows by sub-branch in the comparison table.
        let locale = corp_compare_locale();
        // Two pieces of research with distinct sub-branches. The locale alone
        // can't carry the sub-branch (it lives on the ResearchStat in the
        // Sirenix dump), so we set them up here.
        let mut chem = make_research("research_chem_main1", "UnlockBonus", None);
        chem.subbranch = "Chemical".into();
        let mut iris = make_research("research_sc_iris", "UnlockSpacecraftType", Some("spacecraft_chem_small"));
        iris.subbranch = "Spacecraft".into();
        let sirenix = Sirenix {
            research: vec![chem, iris],
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_EarlyExploration".into(),
                corp_starts: vec![CorpStartStat {
                    company_id: "Solex".into(),
                    starting_money: 5_000_000.0,
                    completed_research: vec![
                        "research_chem_main1".into(),
                        "research_sc_iris".into(),
                    ],
                    starting_launch_vehicles: 0,
                    starting_spacecraft: 0,
                    ..Default::default()
                }],
                body_habitability: vec![],
            }],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        // Parse the CORP_DATA blob.
        let blob_pos = page.find("window.CORP_DATA = ").expect("CORP_DATA blob present");
        let blob_tail = &page[blob_pos + "window.CORP_DATA = ".len()..];
        let blob_end = blob_tail.find("};").expect("CORP_DATA blob ends with `};`");
        let blob_json = format!("{}{}", &blob_tail[..blob_end], "}");
        let parsed: serde_json::Value = serde_json::from_str(&blob_json)
            .unwrap_or_else(|e| panic!("CORP_DATA JSON parse failed: {e}\n{blob_json}"));
        let solex = &parsed["scenarios"][0]["corps"][0];
        let research = solex["research"].as_array().expect("research array");
        assert_eq!(research.len(), 2, "SoleX should have 2 research entries");
        for entry in research {
            assert!(
                entry.is_object(),
                "research entry must be an object, got {entry}"
            );
            assert!(
                entry["name"].is_string(),
                "research entry missing string `name`: {entry}"
            );
            assert!(
                entry["category"].is_string(),
                "research entry missing string `category`: {entry}"
            );
        }
        // The Iris row should carry the "Spacecraft" sub-branch label.
        let iris_entry = research
            .iter()
            .find(|e| e["name"].as_str() == Some("Iris SC"))
            .expect("Iris SC entry present");
        assert_eq!(iris_entry["category"].as_str(), Some("Spacecraft"));
        // The Kerolox row should carry "Chemical" (humanized to player label).
        let kero_entry = research
            .iter()
            .find(|e| e["name"].as_str() == Some("Kerolox"))
            .expect("Kerolox entry present");
        assert_eq!(kero_entry["category"].as_str(), Some("Chemical"));
    }

    #[test]
    fn corporations_corp_data_humanizes_multi_word_subbranch() {
        // CamelCase sub-branch ids like `LaunchVehicle` must humanize to
        // "Launch Vehicle" before they reach the CORP_DATA blob.
        let locale = corp_compare_locale();
        let mut r = make_research("research_lv_main1", "UnlockVehicleType", Some("lv_chem_small"));
        r.subbranch = "LaunchVehicle".into();
        let sirenix = Sirenix {
            research: vec![r],
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_EarlyExploration".into(),
                corp_starts: vec![CorpStartStat {
                    company_id: "Solex".into(),
                    starting_money: 5_000_000.0,
                    completed_research: vec!["research_lv_main1".into()],
                    starting_launch_vehicles: 0,
                    starting_spacecraft: 0,
                    ..Default::default()
                }],
                body_habitability: vec![],
            }],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        assert!(
            page.contains("\"category\":\"Launch Vehicle\""),
            "expected humanized 'Launch Vehicle' category in blob:\n{page}"
        );
    }

    #[test]
    fn corporations_corp_data_unknown_subbranch_buckets_as_other() {
        // A research id missing from the Sirenix research list (rare — e.g.
        // dump skew) must still appear in CORP_DATA with category "Other".
        let locale = corp_compare_locale();
        let sirenix = Sirenix {
            // Intentionally empty research list — the completed_research id
            // below has no matching ResearchStat.
            research: vec![],
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_EarlyExploration".into(),
                corp_starts: vec![CorpStartStat {
                    company_id: "Solex".into(),
                    starting_money: 5_000_000.0,
                    completed_research: vec!["research_chem_main1".into()],
                    starting_launch_vehicles: 0,
                    starting_spacecraft: 0,
                    ..Default::default()
                }],
                body_habitability: vec![],
            }],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);
        assert!(
            page.contains("\"category\":\"Other\""),
            "expected fallback 'Other' category in blob:\n{page}"
        );
    }

    #[test]
    fn corporations_corp_data_includes_starting_facilities_with_names_and_counts() {
        // Each corp in the CORP_DATA blob must carry a `starting_facilities`
        // array of `{name, count}` objects — name resolved via locale.facilities,
        // count copied from the parsed Sirenix value.  This is what
        // corporations.js needs to render the "Starting facilities" section.
        let mut locale = corp_compare_locale();
        // Match the locale.json schema: facility ids are stored WITHOUT the
        // `build_` prefix (the dump uses `build_noblegasmine`, locale uses
        // `noblegasmine`).  gen_pages strips `build_` before lookup.
        locale.facilities = vec![
            Facility { id: "noblegasmine".into(), name: "NOBLE GAS MINE".into(),
                description: String::new() },
            Facility { id: "metalmine".into(), name: "METAL MINE".into(),
                description: String::new() },
        ];
        let sirenix = Sirenix {
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_TheExpansion".into(),
                corp_starts: vec![CorpStartStat {
                    company_id: "ESA".into(),
                    starting_money: 25_700_000.0,
                    completed_research: vec![],
                    starting_launch_vehicles: 0,
                    starting_spacecraft: 0,
                    starting_facilities: vec![
                        ("build_noblegasmine".into(), 3),
                        ("build_metalmine".into(), 1),
                    ],
                }],
                body_habitability: vec![],
            }],
            ..Default::default()
        };
        let page = page_corporations(&locale, &sirenix);

        // Parse the CORP_DATA blob and walk to ESA.starting_facilities.
        let blob_pos = page.find("window.CORP_DATA = ").expect("CORP_DATA blob present");
        let blob_tail = &page[blob_pos + "window.CORP_DATA = ".len()..];
        let blob_end = blob_tail.find("};").expect("CORP_DATA blob ends with `};`");
        let blob_json = format!("{}{}", &blob_tail[..blob_end], "}");
        let parsed: serde_json::Value = serde_json::from_str(&blob_json)
            .unwrap_or_else(|e| panic!("CORP_DATA JSON parse failed: {e}\n{blob_json}"));
        let esa = &parsed["scenarios"][0]["corps"]
            .as_array()
            .expect("corps array")
            .iter()
            .find(|c| c["name"].as_str() == Some("ESA"))
            .expect("ESA corp present")
            .clone();
        let facs = esa["starting_facilities"]
            .as_array()
            .expect("starting_facilities array");
        // Two distinct facility kinds for ESA in this fixture.
        assert_eq!(facs.len(), 2, "expected 2 starting-facility kinds, got {facs:?}");
        // Find each by resolved name.  Display name comes from locale (uppercase),
        // smart-title-cased by gen_pages → "Noble Gas Mine".
        let by_name: std::collections::HashMap<&str, i64> = facs
            .iter()
            .map(|f| {
                (
                    f["name"].as_str().expect("name string"),
                    f["count"].as_i64().expect("count int"),
                )
            })
            .collect();
        assert_eq!(by_name.get("Noble Gas Mine"), Some(&3),
            "expected Noble Gas Mine ×3, got {by_name:?}");
        assert_eq!(by_name.get("Metal Mine"), Some(&1),
            "expected Metal Mine ×1, got {by_name:?}");
    }

    #[test]
    fn contracts_next_chain_skips_test_followups() {
        // A contract whose unlock_rewards reference a tutorial `_test` follow-up
        // must not render a `Next:` link to it (the target row is filtered out,
        // so the anchor never appears).  Regression for audit fix #2.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_tutorial_moonlandingMultiModuleDeliverTest".into()],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert!(
            !page.contains("contract-contract-tutorial-moonlandingmultimoduledelivertest"),
            "dead `Next:` link to a filtered _test contract leaked:\n{page}"
        );
        assert!(
            !page.contains("moonlandingMultiModuleDeliverTest"),
            "raw _test id rendered in Next column:\n{page}"
        );
    }

    #[test]
    fn research_page_renders_era_column_for_each_stage() {
        // The Research table gains an "Era" column between Cost and Prereqs,
        // mapping stage 0/1/2 → Early / Mid / Late.  A stage=1 entry must
        // render "Mid" in its row.
        let locale = nav_fixture_locale();
        let mut mid = make_research("research_lifesup_1", "UnlockFacility", Some("geothermal"));
        mid.stage = 1;
        let sirenix = Sirenix {
            research: vec![mid],
            ..Default::default()
        };
        let page = page_research(&locale, &sirenix);
        // Header is present (uses md_table_with_tips' span wrapper).
        assert!(
            page.contains(">Era<") || page.contains(" Era "),
            "page missing Era column header:\n{page}"
        );
        // The stage=1 row renders "Mid".
        assert!(
            page.contains("| Mid |"),
            "page missing Mid cell for stage=1 research:\n{page}"
        );
    }

    #[test]
    fn research_page_renders_era_late_for_stage_two() {
        let locale = nav_fixture_locale();
        let mut late = make_research("research_lifesup_1", "UnlockFacility", Some("geothermal"));
        late.stage = 2;
        let sirenix = Sirenix {
            research: vec![late],
            ..Default::default()
        };
        let page = page_research(&locale, &sirenix);
        assert!(
            page.contains("| Late |"),
            "page missing Late cell for stage=2 research:\n{page}"
        );
    }

    #[test]
    fn research_page_renders_stacked_secondary_bonuses_in_unlocks_cell() {
        // A research with two secondary bonuses (e.g. fusion-prop III: thrust +20
        // and power production +25) should render both lines in the Unlocks
        // cell, in addition to its primary unlock.
        let locale = nav_fixture_locale();
        let mut r = make_research("research_lifesup_1", "UnlockFacility", Some("geothermal"));
        r.secondary_unlocks = vec![
            SecondaryUnlockStat {
                action: "UnlockBonus".into(),
                target: String::new(),
                bonus: "ComponentThrust".into(),
                bonus_parameter: 20.0,
            },
            SecondaryUnlockStat {
                action: "UnlockBonus".into(),
                target: String::new(),
                bonus: "PowerProduction".into(),
                bonus_parameter: 25.0,
            },
        ];
        let sirenix = Sirenix {
            research: vec![r],
            ..Default::default()
        };
        let page = page_research(&locale, &sirenix);
        assert!(
            page.contains("+20 Component thrust"),
            "page missing first secondary bonus:\n{page}"
        );
        assert!(
            page.contains("+25 Power production"),
            "page missing second secondary bonus:\n{page}"
        );
        // Both bonuses live in the same Unlocks cell, joined by `<br>`.
        assert!(
            page.contains("+20 Component thrust<br>+25 Power production"),
            "secondary bonuses not stacked into a single cell with <br>:\n{page}"
        );
    }

    #[test]
    fn research_page_renders_secondary_facility_unlock() {
        // Lifesup II type research: primary unlock is a bonus, secondary
        // unlock is a facility (build_habitatdome).  The Unlocks cell should
        // show the facility name (resolved via locale fixture) on its own line.
        let mut locale = nav_fixture_locale();
        locale.facilities.push(Facility {
            id: "habitatdome".into(),
            name: "Habitat Dome".into(),
            description: "Mid-tier habitat.".into(),
        });
        let mut r = make_research("research_lifesup_1", "UnlockBonus", None);
        r.bonus_kind = Some("BuildSpeed".into());
        r.bonus_amount = 25.0;
        r.bonus_components = vec!["Facility".into()];
        r.secondary_unlocks = vec![SecondaryUnlockStat {
            action: "UnlockFacility".into(),
            target: "build_habitatdome".into(),
            bonus: String::new(),
            bonus_parameter: 0.0,
        }];
        let sirenix = Sirenix {
            research: vec![r],
            ..Default::default()
        };
        let page = page_research(&locale, &sirenix);
        assert!(
            page.contains("Facility: ") && page.contains("Habitat Dome"),
            "page missing secondary facility unlock label:\n{page}"
        );
    }

    #[test]
    fn research_unlockfacility_without_real_facility_does_not_emit_dead_link() {
        // Some research nodes carry an `UnlockFacility` action whose target is
        // actually a spacecraft module (no facility with that id exists in
        // locale).  The page must not render a broken `facilities/#facility-…`
        // link to it.
        let locale = nav_fixture_locale();
        let sirenix = Sirenix {
            research: vec![make_research(
                "research_lifesup_1",
                "UnlockFacility",
                Some("module_crew_compartment"),
            )],
            ..Default::default()
        };
        let page = page_research(&locale, &sirenix);
        assert!(
            !page.contains("facilities/#facility-module-crew-compartment"),
            "page emitted dead `facilities/#facility-module-…` anchor:\n{page}"
        );
        // The label should still render — just not as a link.
        assert!(
            page.contains("Crew Compartment") || page.contains("module_crew_compartment"),
            "module target label missing entirely:\n{page}"
        );
    }

    #[test]
    fn fmt_magnitude_abs_preserves_subunit_precision() {
        // Integer round-trips without a decimal point.
        assert_eq!(fmt_magnitude_abs(100.0), "100");
        // Sunshade's albedo delta must not collapse to "0.0".
        assert_eq!(fmt_magnitude_abs(-0.006), "0.006");
        // Trailing zeros are trimmed so "0.600" → "0.6".
        assert_eq!(fmt_magnitude_abs(0.6), "0.6");
        // Mid-range fractions keep two decimals (then trim).
        assert_eq!(fmt_magnitude_abs(2.5), "2.5");
        // Zero is zero.
        assert_eq!(fmt_magnitude_abs(0.0), "0");
    }

    // ---------- Facilities page: build time / launch bonus / role / terraforming ----------

    fn facility_fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![
                Facility {
                    id: "habitat".into(),
                    name: "Habitat".into(),
                    description: "Houses colonists.".into(),
                },
                Facility {
                    id: "launch_elevator".into(),
                    name: "Space Elevator".into(),
                    description: "Cable to orbit.".into(),
                },
                Facility {
                    id: "launch_pad".into(),
                    name: "Launchpad".into(),
                    description: "Standard rocket pad.".into(),
                },
                Facility {
                    id: "terraform_magnet".into(),
                    name: "Magnetic Field Generator".into(),
                    description: "Generates a planetary magnetic field.".into(),
                },
                Facility {
                    id: "lab".into(),
                    name: "Research Lab".into(),
                    description: "Conducts research.".into(),
                },
                Facility {
                    id: "launch_magrails".into(),
                    name: "Magnetic Launch Rails".into(),
                    description: "Long ramp built atop suitable terrain.".into(),
                },
            ],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    fn facility_stat(id: &str, ftype: &str) -> FacilityStat {
        FacilityStat {
            id: id.into(),
            descriptor: "Ground".into(),
            placement: "Surface".into(),
            facility_type: ftype.into(),
            build_cost: vec![],
            maintenance_per_day: 0.0,
            workers_required: 0,
            energy_consumption: 0.0,
            research_prereq: None,
            is_obsolete: false,
            can_be_scrapped: false,
            can_be_turned_off: false,
            build_time_days: 0.0,
            bonus_data: None,
            role: None,
            role_magnitude: 0.0,
            habitability_deltas: vec![],
            habitat_constraints: vec![],
            produces: vec![],
            consumes: vec![],
        }
    }

    #[test]
    fn facilities_page_renders_build_time_column() {
        let locale = facility_fixture_locale();
        let mut f = facility_stat("build_lab", "Other");
        f.build_time_days = 200.0;
        let sirenix = Sirenix {
            facilities: vec![f],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        // Header column for build time present.
        assert!(
            page.contains("Time"),
            "facilities page is missing a Time/build-duration column:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Research Lab"))
            .expect("Research Lab row present");
        assert!(row.contains("200"), "row should show 200-day build time: {row}");
    }

    #[test]
    fn facilities_page_renders_launch_bonus_for_launch_facility() {
        let locale = facility_fixture_locale();
        let mut elev = facility_stat("build_launch_elevator", "LaunchFacility");
        elev.bonus_data = Some(("LaunchCostOptionInPlanMission".into(), 10.0));
        let mut pad = facility_stat("build_launch_pad", "LaunchFacility");
        pad.bonus_data = Some(("LaunchCost".into(), 10.0));
        let sirenix = Sirenix {
            facilities: vec![elev, pad],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let elev_row = page
            .lines()
            .find(|l| l.contains("Space Elevator"))
            .expect("Space Elevator row present");
        // The launch-bonus column must surface a player-facing phrasing — never
        // the raw enum name. "LaunchCostOptionInPlanMission" is the source enum;
        // it must NOT appear verbatim in the row.
        assert!(
            !elev_row.contains("LaunchCostOptionInPlanMission"),
            "raw enum leaked: {elev_row}"
        );
        // The row should describe the discount in everyday words.
        assert!(
            elev_row.to_lowercase().contains("launch") || elev_row.contains("%"),
            "launch-bonus column should describe the bonus: {elev_row}"
        );
        // The bonus magnitude (10) should appear somewhere on the row.
        assert!(elev_row.contains("10"), "magnitude missing: {elev_row}");
    }

    #[test]
    fn facilities_page_renders_role_magnitude_for_habitat() {
        let locale = facility_fixture_locale();
        let mut hab = facility_stat("build_habitat", "Habitation");
        hab.role = Some("CrewCapacity".into());
        hab.role_magnitude = 100.0;
        let sirenix = Sirenix {
            facilities: vec![hab],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("**Habitat**"))
            .expect("Habitat row present");
        // Magnitude column carries the raw number; the role column shows
        // CrewCapacity → friendly "Crew" / similar. We accept either ordering
        // but the row must contain both the number and a recognisable label.
        assert!(row.contains("100"), "magnitude missing: {row}");
        assert!(
            row.to_lowercase().contains("crew"),
            "role label should mention crew: {row}"
        );
    }

    #[test]
    fn facilities_page_renders_terraforming_deltas_for_magnet_station() {
        let locale = facility_fixture_locale();
        let mut mag = facility_stat("build_terraform_magnet", "Other");
        mag.habitability_deltas = vec![
            ("Radiation".into(), -0.6),
            ("Magnetic field".into(), 0.6),
        ];
        let sirenix = Sirenix {
            facilities: vec![mag],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        // Header column present.
        assert!(
            page.contains("Terraforming"),
            "facilities page is missing a Terraforming column:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Magnetic Field Generator"))
            .expect("magnet station row present");
        assert!(
            row.contains("Radiation"),
            "row should mention Radiation: {row}"
        );
        // The sign matters — radiation drops, so the cell should carry a
        // minus sign (either ASCII `-` or the prettier `−` U+2212). Both are
        // acceptable.
        assert!(
            row.contains("−0.6") || row.contains("-0.6"),
            "row should carry signed delta: {row}"
        );
        assert!(
            row.contains("Magnetic field"),
            "row should mention Magnetic field: {row}"
        );
    }

    #[test]
    fn facilities_page_renders_atmosphere_required_for_magrails() {
        // build_launch_magrails carries a single Pressure 0.0001..2 build
        // constraint — the cell should surface a player-facing "Atmosphere
        // required" label, never the raw enum or numeric thresholds (the
        // 0.0001 lower bound is a "must not be vacuum" tell, not a number
        // the player should have to interpret).
        let locale = facility_fixture_locale();
        let mut mr = facility_stat("build_launch_magrails", "LaunchFacility");
        mr.habitat_constraints = vec![HabitatConstraintStat {
            parameter: "Pressure".into(),
            min: 0.0001,
            max: 2.0,
        }];
        let sirenix = Sirenix {
            facilities: vec![mr],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        assert!(
            page.contains("Habitat req."),
            "facilities page is missing the Habitat req. column header:\n{page}"
        );
        let row = page
            .lines()
            .find(|l| l.contains("Magnetic Launch Rails"))
            .expect("Magnetic Launch Rails row present");
        assert!(
            row.contains("Atmosphere required"),
            "row should label the Pressure 0.0001..2 constraint as 'Atmosphere required': {row}"
        );
        // Raw enum name must not leak.
        assert!(
            !row.contains("Pressure 0.0001"),
            "raw pressure range leaked instead of friendly label: {row}"
        );
    }

    #[test]
    fn facilities_page_renders_vacuum_only_for_pressure_zero() {
        // Vacuum-only launch facilities (mass driver / magnetic catapult)
        // carry a Pressure 0..0.0001 constraint — the cell should label
        // that as "Vacuum only".
        let locale = facility_fixture_locale();
        let mut md = facility_stat("build_launch_pad", "LaunchFacility");
        md.habitat_constraints = vec![HabitatConstraintStat {
            parameter: "Pressure".into(),
            min: 0.0,
            max: 0.0001,
        }];
        let sirenix = Sirenix {
            facilities: vec![md],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Launchpad"))
            .expect("Launchpad row present");
        assert!(
            row.contains("Vacuum only"),
            "row should label the Pressure 0..0.0001 constraint as 'Vacuum only': {row}"
        );
    }

    #[test]
    fn facilities_page_labels_object_type_asteroid_gate() {
        // Synthetic ObjectType:Asteroid constraint should render as
        // "Asteroid only" — never leaking the raw synthetic key.
        let mut locale = facility_fixture_locale();
        locale.facilities.push(Facility {
            id: "asteroid_engine_facility".into(),
            name: "Asteroid Engine".into(),
            description: "Pushes asteroids.".into(),
        });
        let mut ae = facility_stat("build_asteroid_engine_facility", "Other");
        ae.habitat_constraints = vec![HabitatConstraintStat {
            parameter: "ObjectType:Asteroid".into(),
            min: 0.0,
            max: 0.0,
        }];
        let sirenix = Sirenix {
            facilities: vec![ae],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Asteroid Engine"))
            .expect("Asteroid Engine row present");
        assert!(
            row.contains("Asteroid only"),
            "row should label the asteroid-only gate as 'Asteroid only': {row}"
        );
        // Synthetic raw key must NOT leak.
        assert!(
            !row.contains("ObjectType:"),
            "raw synthetic key leaked: {row}"
        );
    }

    #[test]
    fn facilities_page_renders_dash_for_no_habitat_constraints() {
        // A facility with an empty constraint list should render `—` in the
        // Habitat req. column.
        let locale = facility_fixture_locale();
        let f = facility_stat("build_lab", "Other");
        let sirenix = Sirenix {
            facilities: vec![f],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Research Lab"))
            .expect("Research Lab row present");
        // The Habitat req. column lives between Terraforming and Prereq —
        // it should carry the standard em-dash placeholder.
        assert!(
            row.contains(" — "),
            "row should carry an em-dash placeholder for missing habitat constraints: {row}"
        );
    }

    #[test]
    fn facilities_page_role_dash_when_absent() {
        // A facility without a role should NOT leak the raw enum "None" — it
        // should render the standard `—` placeholder.
        let locale = facility_fixture_locale();
        let f = facility_stat("build_lab", "Other");
        let sirenix = Sirenix {
            facilities: vec![f],
            ..Default::default()
        };
        let page = page_facilities(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("Research Lab"))
            .expect("Lab row present");
        // The string "None" appears nowhere on the row's facility-side data.
        // (The literal word "none" in description text is allowed; our fixture
        // description doesn't contain it, so any "None" leak would be from
        // role.)
        assert!(
            !row.contains("None"),
            "raw role enum leaked: {row}"
        );
    }

    // ---------- Launch-vehicles page: alternative-launch-methods table ----------

    /// Locale fixture used by the alternative-launch-methods tests.  Mirrors
    /// `facility_fixture_locale` but adds a `launch_massdriver` row plus a
    /// resource entry so `fmt_build_cost` can resolve a label.
    fn alt_launch_fixture_locale() -> Locale {
        let mut locale = facility_fixture_locale();
        // Override the shared fixture's "Launchpad" with the real-game display
        // name so the alt-launch tests can assert on it directly.
        for f in &mut locale.facilities {
            if f.id == "launch_pad" {
                f.name = "Launch Pad".into();
            }
        }
        locale.facilities.push(Facility {
            id: "launch_massdriver".into(),
            name: "Stationary Mass Driver".into(),
            description: "Set of superconducting electromagnetic accelerators.".into(),
        });
        locale.resources.push(ResourceEntry {
            id: "steel".into(),
            name: "Steel".into(),
        });
        locale
    }

    #[test]
    fn alternative_launch_methods_table_uses_facility_data() {
        let locale = alt_launch_fixture_locale();
        let mut pad = facility_stat("build_launch_pad", "LaunchFacility");
        pad.build_cost = vec![ResourceCost { resource_id: "steel".into(), amount: 1500.0 }];
        pad.build_time_days = 40.0;
        pad.bonus_data = Some(("LaunchCost".into(), 10.0));
        let mut md = facility_stat("build_launch_massdriver", "LaunchFacility");
        md.build_cost = vec![ResourceCost { resource_id: "steel".into(), amount: 75_000.0 }];
        md.build_time_days = 365.0;
        md.bonus_data = Some(("LaunchCost".into(), 90.0));
        let sirenix = Sirenix {
            facilities: vec![pad, md],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        let alt = page
            .split("## Alternative launch methods")
            .nth(1)
            .expect("Alternative launch methods section present");
        // The hand-coded prose ("Long ramp built atop suitable terrain") must
        // be gone — the section now comes from data.
        assert!(
            !alt.contains("Long ramp built atop suitable terrain"),
            "hand-coded prose still present: {alt}"
        );
        // Each fixture facility should have a row whose Build cost cell shows
        // the real fixture amount (1.5k / 75k), not "—".
        let pad_row = alt
            .lines()
            .find(|l| l.contains("[Launch Pad]"))
            .unwrap_or_else(|| panic!("Launch Pad row present:\n{alt}"));
        assert!(
            pad_row.contains("1.5k"),
            "Launch Pad row should show build cost 1.5k steel: {pad_row}"
        );
        let md_row = alt
            .lines()
            .find(|l| l.contains("[Stationary Mass Driver]"))
            .unwrap_or_else(|| panic!("Mass Driver row present:\n{alt}"));
        assert!(
            md_row.contains("75k"),
            "Mass Driver row should show build cost 75k steel: {md_row}"
        );
    }

    #[test]
    fn alternative_launch_methods_omits_non_launch_facilities() {
        let locale = alt_launch_fixture_locale();
        let pad = facility_stat("build_launch_pad", "LaunchFacility");
        let hab = facility_stat("build_habitat", "Habitation");
        let sirenix = Sirenix {
            facilities: vec![pad, hab],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        let alt = page
            .split("## Alternative launch methods")
            .nth(1)
            .expect("Alternative launch methods section present");
        assert!(
            !alt.contains("**Habitat**"),
            "non-LaunchFacility leaked into table: {alt}"
        );
    }

    #[test]
    fn alternative_launch_methods_links_to_facility_anchor() {
        let locale = alt_launch_fixture_locale();
        let pad = facility_stat("build_launch_pad", "LaunchFacility");
        let md = facility_stat("build_launch_massdriver", "LaunchFacility");
        let sirenix = Sirenix {
            facilities: vec![pad, md],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        let alt = page
            .split("## Alternative launch methods")
            .nth(1)
            .expect("Alternative launch methods section present");
        assert!(
            alt.contains("../facilities/#facility-launch-pad"),
            "missing cross-page link to facilities/#facility-launch-pad: {alt}"
        );
        assert!(
            alt.contains("../facilities/#facility-launch-massdriver"),
            "missing cross-page link to facilities/#facility-launch-massdriver: {alt}"
        );
    }

    // ---------- Launch-vehicles page: Max G column ----------

    /// Minimal LaunchVehicleStat builder for tests. Mirrors the `facility_stat`
    /// fixture: zero/empty defaults that callers customise per-test.
    fn launch_vehicle_stat(id: &str) -> LaunchVehicleStat {
        LaunchVehicleStat {
            id: id.into(),
            max_payload: 0.0,
            max_fuel_load: 0.0,
            exhaust_velocity: 0.0,
            reusability: 0.0,
            can_send_human: false,
            is_locked: false,
            build_cost: vec![],
            build_time_days: 0.0,
            launch_cost: 0.0,
            maintenance_cost_per_day: 0.0,
            fuel_type_on_start: None,
            gravity_gate: None,
        }
    }

    fn max_g_fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![],
            launch_vehicles: vec![
                NameDesc {
                    id: "id_Rocket_RocketType5".into(),
                    name: "Al-Ice Rocket".into(),
                    description: String::new(),
                },
                NameDesc {
                    id: "id_Rocket_RocketType1".into(),
                    name: "Sparrow".into(),
                    description: String::new(),
                },
            ],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    #[test]
    fn launch_vehicles_page_renders_max_g_column_header() {
        // The chemical-rockets table should expose a "Max G" header column with
        // the canonical tooltip — even when no rocket carries a gate.
        let locale = max_g_fixture_locale();
        let sparrow = launch_vehicle_stat("id_Rocket_RocketType1");
        let sirenix = Sirenix {
            launch_vehicles: vec![sparrow],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        assert!(
            page.contains("Max G"),
            "launch-vehicles page is missing a Max G column header:\n{page}"
        );
        assert!(
            page.contains("Maximum surface gravity this rocket can launch from"),
            "Max G column tooltip missing:\n{page}"
        );
    }

    #[test]
    fn launch_vehicles_page_renders_max_g_for_al_ice() {
        // Al-Ice rocket carries a Gravity 0..1.8 gate -> the cell should render
        // "≤ 1.8 G" (min == 0 collapses to a single ceiling). The Sparrow row
        // has no gate, so its Max G cell should render "Any".
        let locale = max_g_fixture_locale();
        let mut al_ice = launch_vehicle_stat("id_Rocket_RocketType5");
        al_ice.gravity_gate = Some(GravityGateStat { min_g: 0.0, max_g: 1.8 });
        let sparrow = launch_vehicle_stat("id_Rocket_RocketType1");
        let sirenix = Sirenix {
            launch_vehicles: vec![al_ice, sparrow],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        let al_ice_row = page
            .lines()
            .find(|l| l.contains("**Al-Ice Rocket**"))
            .expect("Al-Ice row present");
        assert!(
            al_ice_row.contains("≤ 1.8 G"),
            "Al-Ice row should render Max G as '≤ 1.8 G': {al_ice_row}"
        );
        let sparrow_row = page
            .lines()
            .find(|l| l.contains("**Sparrow**"))
            .expect("Sparrow row present");
        assert!(
            sparrow_row.contains("Any"),
            "Sparrow row (no gate) should render Max G as 'Any': {sparrow_row}"
        );
    }

    #[test]
    fn launch_vehicles_page_renders_max_g_range_when_min_nonzero() {
        // Defensive: a gate with min > 0 should render as "{min} – {max} G",
        // not collapsed to the ceiling.
        let locale = max_g_fixture_locale();
        let mut al_ice = launch_vehicle_stat("id_Rocket_RocketType5");
        al_ice.gravity_gate = Some(GravityGateStat { min_g: 0.5, max_g: 1.5 });
        let sirenix = Sirenix {
            launch_vehicles: vec![al_ice],
            ..Default::default()
        };
        let page = page_launch_vehicles(&locale, &sirenix);
        let row = page
            .lines()
            .find(|l| l.contains("**Al-Ice Rocket**"))
            .expect("Al-Ice row present");
        assert!(
            row.contains("0.5 – 1.5 G"),
            "row with min=0.5 max=1.5 should render '0.5 – 1.5 G': {row}"
        );
    }

    // ---------- Contract row ordering: chain-DFS, tutorials first ----------

    /// Return the 0-based index of `display`'s row inside the contracts table.
    /// Only data rows count — the header row carries `<span title=` (not a
    /// `**name**` cell) and the separator row is `| --- | …`, so both are
    /// filtered out by requiring a `**…**` bold cell.
    fn contract_row_index(page: &str, display: &str) -> usize {
        page.lines()
            .filter(|l| l.starts_with("| ") && l.contains("**"))
            .position(|l| l.contains(&format!("**{display}**")))
            .unwrap_or_else(|| panic!("contract row `{display}` not found in:\n{page}"))
    }

    #[test]
    fn contracts_render_in_chain_dfs_order() {
        // Two roots: a tutorial chain (First Orbit → Explore Luna → Lunar
        // Landing) and a non-tutorial chain (Asteroid Sample → Asteroid Mining).
        // The tutorial root must lead and its full chain must emit contiguously
        // before the non-tutorial root appears.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract("contract_tutorial_moonlanding", vec![], vec![]),
                make_contract(
                    "contract_asteroid_sample",
                    vec![],
                    vec!["contract_asteroid_mining".into()],
                ),
                make_contract("contract_asteroid_mining", vec![], vec![]),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let names = [
            "First Orbit",
            "Explore Luna",
            "Lunar Landing",
            "Asteroid Sample",
            "Asteroid Mining",
        ];
        let indices: Vec<usize> = names.iter().map(|n| contract_row_index(&page, n)).collect();
        assert_eq!(
            indices,
            (0..names.len()).collect::<Vec<_>>(),
            "expected chain-DFS order First Orbit, Explore Luna, Lunar Landing, Asteroid Sample, Asteroid Mining; got indices {indices:?}\npage:\n{page}"
        );
    }

    #[test]
    fn contracts_dfs_visits_branching_children_in_depth_then_alphabetical_order() {
        // Lunar Landing has two direct children at the same depth: Explore Mars
        // and Space Dock.  At equal depth, alphabetical wins — Explore Mars
        // (and its subtree) before Space Dock.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec![
                        "contract_tutorial_marsorbit".into(),
                        "contract_tutorial_spacedock".into(),
                    ],
                ),
                make_contract("contract_tutorial_marsorbit", vec![], vec![]),
                make_contract("contract_tutorial_spacedock", vec![], vec![]),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let mars_idx = contract_row_index(&page, "Explore Mars");
        let dock_idx = contract_row_index(&page, "Space Dock");
        assert!(
            mars_idx < dock_idx,
            "Explore Mars (idx {mars_idx}) should render before Space Dock (idx {dock_idx}); same-depth tie should be alphabetical\npage:\n{page}"
        );
    }

    #[test]
    fn contracts_dfs_does_not_double_visit() {
        // `contract_multi_parent` is reachable from BOTH non-tutorial roots
        // (Root Alpha, Root Beta).  It must render exactly once.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_root_a",
                    vec![],
                    vec!["contract_multi_parent".into()],
                ),
                make_contract(
                    "contract_root_b",
                    vec![],
                    vec!["contract_multi_parent".into()],
                ),
                make_contract("contract_multi_parent", vec![], vec![]),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let occurrences = page
            .lines()
            .filter(|l| l.starts_with("| ") && l.contains("**Multi Parent**"))
            .count();
        assert_eq!(
            occurrences, 1,
            "Multi Parent should render exactly once even with multiple parents; got {occurrences}\npage:\n{page}"
        );
    }

    // ---------- Layer-Asteroid depth bumping ----------

    /// Pull the Order-cell (first column) from a contract's row in the
    /// rendered page, returning it parsed as u32.  Used by the layer-bump
    /// tests below to assert on the *Order column* (not just row position).
    fn contract_order(page: &str, display: &str) -> u32 {
        let row = page
            .lines()
            .find(|l| {
                l.starts_with("| ")
                    && l.contains(&format!("**{display}**"))
            })
            .unwrap_or_else(|| panic!("contract row `{display}` not found in:\n{page}"));
        // Format: "| <order> | <contract> | ..."; first cell after the leading "| "
        // is the Order column.
        let parts: Vec<&str> = row.splitn(3, " | ").collect();
        let order_str = parts
            .first()
            .map(|s| s.trim_start_matches('|').trim())
            .unwrap_or("");
        order_str
            .parse::<u32>()
            .unwrap_or_else(|_| panic!("order cell `{order_str}` not a u32 in row: {row}"))
    }

    /// Build the asteroid-belt gate contract for these tests: it carries a
    /// `SelectLayer` objective on the Asteroid layer (the only contract in
    /// production with this objective is Probing Lutetia, and that's the cue
    /// the gen_pages depth-bump logic keys off).
    fn asteroid_gate_contract(id: &str, unlock_rewards: Vec<String>) -> ContractStat {
        let mut c = make_contract_with_layers(
            id,
            unlock_rewards,
            vec!["Asteroid".into()],
        );
        c.objectives = vec![obj("SelectLayer", 0.0, None)];
        c
    }

    #[test]
    fn layer_asteroid_contract_bumps_to_gate_depth() {
        // Chain: First Orbit (0) → Explore Luna (1) → Lunar Landing (2).
        // contract_asteroid_first has a SelectLayer:Asteroid objective and is
        // unlocked by Lunar Landing → its natural depth is 3.  That's the
        // asteroid gate.
        // contract_asteroid_base has layer:Asteroid but NO contract prereqs —
        // its natural depth is 0, but it must be bumped to ≥ 3 (gate depth)
        // because the player can't physically attempt it until they reach the
        // asteroid belt.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_asteroid_first".into()],
                ),
                asteroid_gate_contract("contract_asteroid_first", vec![]),
                make_contract_with_layers(
                    "contract_asteroid_base",
                    vec![],
                    vec!["Asteroid".into()],
                ),
            ],
            ..Default::default()
        };
        let mut locale = locale;
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_first".into(),
            name: "Probing Lutetia".into(),
            description: "Land on an asteroid.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_base".into(),
            name: "Asteroid Base".into(),
            description: "Build an asteroid base.".into(),
        });
        let page = page_contracts(&locale, &sirenix);
        let gate = contract_order(&page, "Probing Lutetia");
        let base = contract_order(&page, "Asteroid Base");
        assert!(
            base >= gate,
            "Asteroid Base order ({base}) must be ≥ Probing Lutetia gate order ({gate})\npage:\n{page}"
        );
        assert_eq!(gate, 3, "Probing Lutetia natural depth should be 3 (after Lunar Landing@2)\npage:\n{page}");
    }

    #[test]
    fn layer_asteroid_does_not_affect_non_asteroid_contracts() {
        // A contract WITHOUT layer:Asteroid keeps its natural topological depth
        // even when other contracts in the graph are being bumped.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_asteroid_first".into()],
                ),
                asteroid_gate_contract("contract_asteroid_first", vec![]),
                // Root Alpha — a non-tutorial, non-asteroid-layer root
                make_contract("contract_root_a", vec![], vec![]),
            ],
            ..Default::default()
        };
        let mut locale = locale;
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_first".into(),
            name: "Probing Lutetia".into(),
            description: "Land on an asteroid.".into(),
        });
        let page = page_contracts(&locale, &sirenix);
        let root_a_order = contract_order(&page, "Root Alpha");
        assert_eq!(
            root_a_order, 0,
            "Root Alpha has no prereqs and no asteroid layer; should stay at depth 0\npage:\n{page}"
        );
    }

    #[test]
    fn layer_bump_propagates_to_descendants() {
        // contract_asteroid_base has layer:Asteroid, no contract prereq → it
        // gets bumped to gate depth (3 here).  Its child via unlock_rewards
        // (contract_asteroid_mining, which ALSO has layer:Asteroid) must be
        // re-propagated: depth(child) = depth(asteroid_base) + 1 = 4.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_asteroid_first".into()],
                ),
                asteroid_gate_contract("contract_asteroid_first", vec![]),
                make_contract_with_layers(
                    "contract_asteroid_base",
                    vec!["contract_asteroid_mining".into()],
                    vec!["Asteroid".into()],
                ),
                make_contract_with_layers(
                    "contract_asteroid_mining",
                    vec![],
                    vec!["Asteroid".into()],
                ),
            ],
            ..Default::default()
        };
        let mut locale = locale;
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_first".into(),
            name: "Probing Lutetia".into(),
            description: "Land on an asteroid.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_base".into(),
            name: "Asteroid Base".into(),
            description: "Build an asteroid base.".into(),
        });
        let page = page_contracts(&locale, &sirenix);
        let base = contract_order(&page, "Asteroid Base");
        let mining = contract_order(&page, "Asteroid Mining");
        assert_eq!(
            base, 3,
            "Asteroid Base should be bumped to gate depth 3\npage:\n{page}"
        );
        assert!(
            mining > base,
            "Asteroid Mining (order {mining}) must propagate past its parent Asteroid Base (order {base})\npage:\n{page}"
        );
    }

    // ---------- Date-locked contracts use year as Order ----------

    /// Build a date-locked contract (isLocked + dateTimeStringStart).
    /// Used for the Exoplanet Search / interstellar chain ordering tests.
    fn date_locked_contract(
        id: &str,
        date_time_string_start: &str,
        unlock_rewards: Vec<String>,
    ) -> ContractStat {
        let mut c = make_contract(id, vec![], unlock_rewards);
        c.is_locked = true;
        c.date_time_string_start = Some(date_time_string_start.into());
        c
    }

    #[test]
    fn date_locked_contract_uses_year_as_order() {
        // Exoplanet Search has isLocked=true and dateTimeStringStart="2080-01-01 00:00:00".
        // Its Order should be 2080 (year extracted from dateTimeStringStart),
        // not 0 (its natural chain depth — it has no contract or research
        // prereqs).
        let locale = contracts_fixture_locale();
        let mut locale = locale;
        locale.contracts.push(NameDesc {
            id: "contract_general_exoplanetsearch".into(),
            name: "Exoplanet Search".into(),
            description: "Find an exoplanet.".into(),
        });
        let sirenix = Sirenix {
            contracts: vec![date_locked_contract(
                "contract_general_exoplanetsearch",
                "2080-01-01 00:00:00",
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let order = contract_order(&page, "Exoplanet Search");
        assert_eq!(
            order, 2080,
            "Exoplanet Search should be Order 2080 (year from dateTimeStringStart)\npage:\n{page}"
        );
    }

    #[test]
    fn date_locked_contract_surfaces_year_in_prereq_column() {
        // Hiding the Order column (CSS-driven) means the year requirement
        // disappears from the visible row.  The Prereq column should carry
        // a `Year ≥ YYYY` line so the time-gate is still visible to players.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_general_exoplanetsearch".into(),
            name: "Exoplanet Search".into(),
            description: "Find an exoplanet.".into(),
        });
        let sirenix = Sirenix {
            contracts: vec![date_locked_contract(
                "contract_general_exoplanetsearch",
                "2080-01-01 00:00:00",
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        // Locate the row by its name link and inspect the Prereq cell.
        let row = page
            .lines()
            .find(|l| l.contains("Exoplanet Search"))
            .expect("Exoplanet Search row should exist");
        let cells: Vec<&str> = row.split('|').collect();
        // Header: Order | Contract | Prereq | Requirements | Rewards | Premise
        // Pipe split produces leading + trailing empty cells.
        let prereq_cell = cells.get(3).copied().unwrap_or("").trim();
        assert!(
            prereq_cell.contains("Year"),
            "prereq cell should mention the year requirement; got: {prereq_cell}\nrow: {row}"
        );
        assert!(
            prereq_cell.contains("2080"),
            "prereq cell should mention 2080; got: {prereq_cell}\nrow: {row}"
        );
    }

    #[test]
    fn descendants_of_date_locked_get_incremental_years() {
        // Exoplanet Search (2080) → First Step to Interstellar (2081) →
        // Beyond the Solar System (2082).  Each downstream contract gets
        // +1 year past its parent's date-locked Order.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_general_exoplanetsearch".into(),
            name: "Exoplanet Search".into(),
            description: "Find an exoplanet.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_general_interstellar1".into(),
            name: "First Step to Interstellar".into(),
            description: "Build the interstellar vehicle.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_general_interstellar2".into(),
            name: "Beyond the Solar System".into(),
            description: "Travel beyond.".into(),
        });
        let sirenix = Sirenix {
            contracts: vec![
                date_locked_contract(
                    "contract_general_exoplanetsearch",
                    "2080-01-01 00:00:00",
                    vec!["contract_general_interstellar1".into()],
                ),
                make_contract(
                    "contract_general_interstellar1",
                    vec![],
                    vec!["contract_general_interstellar2".into()],
                ),
                make_contract(
                    "contract_general_interstellar2",
                    vec![],
                    vec![],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert_eq!(contract_order(&page, "Exoplanet Search"), 2080, "page:\n{page}");
        assert_eq!(
            contract_order(&page, "First Step to Interstellar"),
            2081,
            "page:\n{page}"
        );
        assert_eq!(
            contract_order(&page, "Beyond the Solar System"),
            2082,
            "page:\n{page}"
        );
    }

    // ---------- Objective-driven depth gating ----------

    #[test]
    fn make_research_objective_gates_contract_to_research_depth() {
        // A contract whose only objective is `MakeResearch: research_X` must
        // have its Order ≥ depth(research_X) + 1.  Here research_deep is at
        // depth 8 (7 prereqs chained), so the contract should be Order ≥ 9.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_gated_by_research".into(),
            name: "Gated By Research".into(),
            description: "Research gates.".into(),
        });
        // Add research entries to the locale.  Names/descriptions don't matter
        // for depth calc.
        for n in 0..=8 {
            locale.research.push(ResearchEntry {
                id: format!("research_chain_{n}"),
                category: "chem".into(),
                name: format!("Chain {n}"),
                description: String::new(),
            });
        }

        // Build a research chain: chain_0 has no prereqs, chain_N depends on
        // chain_(N-1).  depth(chain_8) = 8.
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..=8 {
            let mut r = make_research(&format!("research_chain_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_chain_{}", n - 1)];
            }
            research.push(r);
        }
        let sirenix = Sirenix {
            research,
            contracts: vec![make_contract(
                "contract_gated_by_research",
                vec![obj("MakeResearch", 1.0, Some("research_chain_8"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let order = contract_order(&page, "Gated By Research");
        assert!(
            order >= 9,
            "MakeResearch objective should gate contract depth to ≥ depth(research)+1 = 9; got {order}\npage:\n{page}"
        );
    }

    #[test]
    fn build_facility_objective_gates_contract_to_facility_research_depth() {
        // research_X at depth 5 unlocks facility F.  A contract whose objective
        // is `BuildFacility: F` must have its Order ≥ 6.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_gated_by_facility".into(),
            name: "Gated By Facility".into(),
            description: "Facility gates.".into(),
        });
        for n in 0..=5 {
            locale.research.push(ResearchEntry {
                id: format!("research_fac_{n}"),
                category: "chem".into(),
                name: format!("Fac {n}"),
                description: String::new(),
            });
        }
        // Research chain of length 5.
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..5 {
            let mut r = make_research(&format!("research_fac_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_fac_{}", n - 1)];
            }
            research.push(r);
        }
        // research_fac_5 is at depth 5 and unlocks build_special_facility.
        let mut r5 = make_research(
            "research_fac_5",
            "UnlockFacility",
            Some("build_special_facility"),
        );
        r5.prereqs = vec!["research_fac_4".into()];
        research.push(r5);

        let sirenix = Sirenix {
            research,
            facilities: vec![facility_stat("special_facility", "Production")],
            contracts: vec![make_contract(
                "contract_gated_by_facility",
                vec![obj("BuildFacility", 1.0, Some("build_special_facility"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let order = contract_order(&page, "Gated By Facility");
        assert!(
            order >= 6,
            "BuildFacility objective should gate contract depth to ≥ depth(research)+1 = 6; got {order}\npage:\n{page}"
        );
    }

    #[test]
    fn deliver_resource_objective_uses_first_producer_research() {
        // Resource R is produced by facility F.  F is unlocked by research_R0
        // at depth 4.  A contract with `Deliver: id_resource_R` must have
        // Order ≥ 5.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_deliver_R".into(),
            name: "Deliver R".into(),
            description: "Deliver resource R.".into(),
        });
        for n in 0..=4 {
            locale.research.push(ResearchEntry {
                id: format!("research_R{n}"),
                category: "chem".into(),
                name: format!("R{n}"),
                description: String::new(),
            });
        }
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..4 {
            let mut r = make_research(&format!("research_R{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_R{}", n - 1)];
            }
            research.push(r);
        }
        // research_R4 (depth 4) unlocks build_R_producer.
        let mut r4 = make_research(
            "research_R4",
            "UnlockFacility",
            Some("build_R_producer"),
        );
        r4.prereqs = vec!["research_R3".into()];
        research.push(r4);

        // Facility build_R_producer produces resource R.
        let mut facility = facility_stat("R_producer", "Production");
        facility.produces = vec![ResourceCost {
            resource_id: "R".into(),
            amount: 1.0,
        }];

        let sirenix = Sirenix {
            research,
            facilities: vec![facility],
            contracts: vec![make_contract(
                "contract_deliver_R",
                vec![obj("Deliver", 1.0, Some("id_resource_R"))],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let order = contract_order(&page, "Deliver R");
        assert!(
            order >= 5,
            "Deliver objective should gate contract depth to ≥ depth(producer-research)+1 = 5; got {order}\npage:\n{page}"
        );
    }

    #[test]
    fn generic_possession_with_no_product_uses_iris_research_depth_floor() {
        // Fleet Expansion-style: Possession objective with productItem=null
        // (e.g. "Possess 10 spacecraft" — no specific id).  Must be floored
        // at depth(research_sc_iris) + 1.  Here research_sc_iris is at depth
        // 2 (chained behind two prereqs), so the contract should be Order ≥ 3.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_generic_fleet".into(),
            name: "Generic Fleet".into(),
            description: "Possess spacecraft.".into(),
        });
        // research_sc_iris depth-2 chain.
        let mut research: Vec<ResearchStat> = Vec::new();
        let mut chem0 = make_research("research_chem_0", "None", None);
        chem0.prereqs = vec![];
        research.push(chem0);
        let mut chem1 = make_research("research_chem_1", "None", None);
        chem1.prereqs = vec!["research_chem_0".into()];
        research.push(chem1);
        let mut iris = make_research("research_sc_iris", "UnlockSpacecraftType", Some("spacecraft_chem_small"));
        iris.prereqs = vec!["research_chem_1".into()];
        research.push(iris);

        let sirenix = Sirenix {
            research,
            contracts: vec![make_contract(
                "contract_generic_fleet",
                // Possession with target=None.
                vec![obj("Possession", 10.0, None)],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let order = contract_order(&page, "Generic Fleet");
        assert!(
            order >= 3,
            "Generic Possession (no product) should be floored at depth(research_sc_iris)+1 = 3; got {order}\npage:\n{page}"
        );
    }

    // ---------- Re-ordered depth pipeline: asteroid-gate must run AFTER
    // objective-floor propagation so Path B's bumps reach the gate before
    // stranded asteroid contracts are floored against it. ----------

    #[test]
    fn asteroid_gate_reflects_path_b_bump_to_gate_parent() {
        // Setup mirrors production: the asteroid gate (`contract_asteroid_first`)
        // is reachable from `contract_mars_marslanding` via unlock_rewards, and
        // Mars Landing carries a `MakeResearch: research_sc_hermes` objective
        // whose research is at depth 9 (so Path B bumps Mars Landing to 10,
        // which propagates the gate to 11).  Stranded `contract_asteroid_base`
        // must end at ≥ 11, matching the final propagated gate depth — not the
        // gate's pre-Path-B value of ~3.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_first".into(),
            name: "Probing Lutetia".into(),
            description: "Land on an asteroid.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_base".into(),
            name: "Asteroid Base".into(),
            description: "Build an asteroid base.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_mars_marslanding".into(),
            name: "Humans on Mars".into(),
            description: "Land on Mars.".into(),
        });
        // research_sc_hermes chained to depth 9 (8 prereqs).
        for n in 0..=9 {
            locale.research.push(ResearchEntry {
                id: format!("research_hermes_chain_{n}"),
                category: "sc".into(),
                name: format!("Hermes Chain {n}"),
                description: String::new(),
            });
        }
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..9 {
            let mut r = make_research(&format!("research_hermes_chain_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_hermes_chain_{}", n - 1)];
            }
            research.push(r);
        }
        // Production uses `research_sc_hermes` as the gate research; fixture's
        // contracts_fixture_locale already declares it.  Wire it at depth 9.
        let mut hermes = make_research("research_sc_hermes", "None", None);
        hermes.prereqs = vec!["research_hermes_chain_8".into()];
        research.push(hermes);

        let sirenix = Sirenix {
            research,
            contracts: vec![
                // Tutorial chain leading to Mars Landing.
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_mars_marslanding".into()],
                ),
                // Mars Landing has the Hermes research objective (Path B bumps).
                make_contract(
                    "contract_mars_marslanding",
                    vec![obj("MakeResearch", 1.0, Some("research_sc_hermes"))],
                    vec!["contract_asteroid_first".into()],
                ),
                // Asteroid gate.
                asteroid_gate_contract("contract_asteroid_first", vec![]),
                // Stranded asteroid contract.
                make_contract_with_layers(
                    "contract_asteroid_base",
                    vec![],
                    vec!["Asteroid".into()],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let gate = contract_order(&page, "Probing Lutetia");
        let base = contract_order(&page, "Asteroid Base");
        assert!(
            gate >= 11,
            "Probing Lutetia should reach depth ≥ 11 once Mars Landing is bumped by Path B; got {gate}\npage:\n{page}"
        );
        assert!(
            base >= gate,
            "Asteroid Base ({base}) must be ≥ Probing Lutetia's final depth ({gate}) — asteroid-gate must run AFTER Path B propagation\npage:\n{page}"
        );
    }

    #[test]
    fn asteroid_layer_contract_with_research_parent_bumps_to_gate() {
        // Production case: contract_asteroid_pulling is unlocked by
        // research_launch_massengine (depth ~6) and is layer:Asteroid.  Its
        // natural depth via research is ~7, but it should still be bumped to
        // the gate depth (Probing Lutetia, ~11) since the player can't
        // physically attempt any asteroid contract before reaching the belt.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_first".into(),
            name: "Probing Lutetia".into(),
            description: "Land on an asteroid.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_asteroid_pulling".into(),
            name: "Asteroid Pulling".into(),
            description: "Pull an asteroid.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_mars_marslanding".into(),
            name: "Humans on Mars".into(),
            description: "Land on Mars.".into(),
        });
        // Push Mars Landing to depth 10 via Hermes (chain of 9).
        for n in 0..=9 {
            locale.research.push(ResearchEntry {
                id: format!("research_hermes_chain_{n}"),
                category: "sc".into(),
                name: format!("Hermes Chain {n}"),
                description: String::new(),
            });
        }
        locale.research.push(ResearchEntry {
            id: "research_launch_massengine".into(),
            category: "launch".into(),
            name: "Mass Engine".into(),
            description: String::new(),
        });
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..9 {
            let mut r = make_research(&format!("research_hermes_chain_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_hermes_chain_{}", n - 1)];
            }
            research.push(r);
        }
        let mut hermes = make_research("research_sc_hermes", "None", None);
        hermes.prereqs = vec!["research_hermes_chain_8".into()];
        research.push(hermes);
        // Shallow research that unlocks asteroid_pulling (depth ~6).
        let mut massengine = make_research("research_launch_massengine", "None", None);
        massengine.prereqs = vec!["research_hermes_chain_5".into()];
        massengine.contract_unlocks = vec!["contract_asteroid_pulling".into()];
        research.push(massengine);

        let sirenix = Sirenix {
            research,
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_mars_marslanding".into()],
                ),
                make_contract(
                    "contract_mars_marslanding",
                    vec![obj("MakeResearch", 1.0, Some("research_sc_hermes"))],
                    vec!["contract_asteroid_first".into()],
                ),
                asteroid_gate_contract("contract_asteroid_first", vec![]),
                make_contract_with_layers(
                    "contract_asteroid_pulling",
                    vec![],
                    vec!["Asteroid".into()],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let gate = contract_order(&page, "Probing Lutetia");
        let pulling = contract_order(&page, "Asteroid Pulling");
        assert!(
            pulling >= gate,
            "Asteroid Pulling ({pulling}) must reach gate depth ({gate}) even though it has a shallow research parent\npage:\n{page}"
        );
    }

    // ---------- Fix B: orphan general/spacestation contracts get a chain floor ----------

    #[test]
    fn orphan_general_contracts_floored_at_mars_landing_depth() {
        // contract_general_fleet has no unlock_rewards parent but a Possession
        // objective (Path B floors it at iris+1 ≈ 2).  In production this puts
        // it at Order 4-6, far earlier than realistic play.  Fix B floors it
        // at depth(contract_mars_marslanding) — the deepest tutorial-final
        // contract — so it sits with mid/late tutorial-chain progression.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_mars_marslanding".into(),
            name: "Humans on Mars".into(),
            description: "Land on Mars.".into(),
        });
        // research_sc_hermes chained to depth 9 so Mars Landing → Path B → 10.
        for n in 0..=9 {
            locale.research.push(ResearchEntry {
                id: format!("research_hermes_chain_{n}"),
                category: "sc".into(),
                name: format!("Hermes Chain {n}"),
                description: String::new(),
            });
        }
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..9 {
            let mut r = make_research(&format!("research_hermes_chain_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_hermes_chain_{}", n - 1)];
            }
            research.push(r);
        }
        let mut hermes = make_research("research_sc_hermes", "None", None);
        hermes.prereqs = vec!["research_hermes_chain_8".into()];
        research.push(hermes);

        let sirenix = Sirenix {
            research,
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_mars_marslanding".into()],
                ),
                make_contract(
                    "contract_mars_marslanding",
                    vec![obj("MakeResearch", 1.0, Some("research_sc_hermes"))],
                    vec![],
                ),
                // Orphan general — generic possession floor.
                make_contract(
                    "contract_general_fleet",
                    vec![obj("Possession", 10.0, None)],
                    vec![],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let mars = contract_order(&page, "Humans on Mars");
        let fleet = contract_order(&page, "Fleet Expansion");
        assert!(
            mars >= 10,
            "Humans on Mars should reach depth ≥ 10 once Path B bumps from Hermes research; got {mars}\npage:\n{page}"
        );
        assert!(
            fleet >= mars,
            "Fleet Expansion ({fleet}) must be floored at Humans on Mars depth ({mars}) — Fix B for orphan general/spacestation contracts\npage:\n{page}"
        );
    }

    #[test]
    fn orphan_spacestation_contracts_floored_at_mars_landing_depth() {
        // Same shape as the general-orphan test but for contract_spacestation_*.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_mars_marslanding".into(),
            name: "Humans on Mars".into(),
            description: "Land on Mars.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_spacestation_fuel1".into(),
            name: "Propellant Depot".into(),
            description: "Stock the orbital depot.".into(),
        });
        for n in 0..=9 {
            locale.research.push(ResearchEntry {
                id: format!("research_hermes_chain_{n}"),
                category: "sc".into(),
                name: format!("Hermes Chain {n}"),
                description: String::new(),
            });
        }
        let mut research: Vec<ResearchStat> = Vec::new();
        for n in 0..9 {
            let mut r = make_research(&format!("research_hermes_chain_{n}"), "None", None);
            if n > 0 {
                r.prereqs = vec![format!("research_hermes_chain_{}", n - 1)];
            }
            research.push(r);
        }
        let mut hermes = make_research("research_sc_hermes", "None", None);
        hermes.prereqs = vec!["research_hermes_chain_8".into()];
        research.push(hermes);

        let sirenix = Sirenix {
            research,
            contracts: vec![
                make_contract(
                    "contract_tutorial_firstorbit",
                    vec![],
                    vec!["contract_tutorial_moonorbit".into()],
                ),
                make_contract(
                    "contract_tutorial_moonorbit",
                    vec![],
                    vec!["contract_tutorial_moonlanding".into()],
                ),
                make_contract(
                    "contract_tutorial_moonlanding",
                    vec![],
                    vec!["contract_mars_marslanding".into()],
                ),
                make_contract(
                    "contract_mars_marslanding",
                    vec![obj("MakeResearch", 1.0, Some("research_sc_hermes"))],
                    vec![],
                ),
                // Orphan spacestation — generic possession floor.
                make_contract(
                    "contract_spacestation_fuel1",
                    vec![obj("Possession", 10.0, None)],
                    vec![],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        let mars = contract_order(&page, "Humans on Mars");
        let depot = contract_order(&page, "Propellant Depot");
        assert!(
            depot >= mars,
            "Propellant Depot ({depot}) must be floored at Humans on Mars depth ({mars}) — Fix B for orphan spacestation contracts\npage:\n{page}"
        );
    }

    #[test]
    fn fix_b_does_not_lower_date_locked_general_contracts() {
        // Date-locked contracts like Exoplanet Search are already at 2080+ via
        // Path A; Fix B must not pull them down to mars-landing depth.  The
        // `if floor > cur` guard inside Fix B handles this — verify it works.
        let mut locale = contracts_fixture_locale();
        locale.contracts.push(NameDesc {
            id: "contract_general_exoplanetsearch".into(),
            name: "Exoplanet Search".into(),
            description: "Find an exoplanet.".into(),
        });
        locale.contracts.push(NameDesc {
            id: "contract_mars_marslanding".into(),
            name: "Humans on Mars".into(),
            description: "Land on Mars.".into(),
        });
        let sirenix = Sirenix {
            contracts: vec![
                make_contract(
                    "contract_mars_marslanding",
                    vec![],
                    vec![],
                ),
                date_locked_contract(
                    "contract_general_exoplanetsearch",
                    "2080-01-01 00:00:00",
                    vec![],
                ),
            ],
            ..Default::default()
        };
        let page = page_contracts(&locale, &sirenix);
        assert_eq!(
            contract_order(&page, "Exoplanet Search"),
            2080,
            "Exoplanet Search must keep its 2080 year-Order; Fix B's `if floor > cur` guard must protect it\npage:\n{page}"
        );
    }

    #[test]
    fn missions_page_does_not_embed_contracts_table() {
        // The missions page is a planning primer, not a contracts list.
        // It must NOT carry the contracts table — that lives at
        // /contracts/. The previous embed left two giveaway signatures
        // in the rendered output: the no-sort wrapper around the table,
        // and at least one well-known contract name like "First Orbit".
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix {
            contracts: vec![make_contract(
                "contract_tutorial_firstorbit",
                vec![],
                vec![],
            )],
            ..Default::default()
        };
        let page = page_missions(&locale, &sirenix);
        assert!(
            !page.contains("<div class=\"no-sort\""),
            "missions page must NOT embed the no-sort-wrapped contracts table:\n{page}"
        );
        assert!(
            !page.contains("**First Orbit**"),
            "missions page must NOT contain contract names like 'First Orbit'; they belong on /contracts/:\n{page}"
        );
    }

    #[test]
    fn missions_page_links_to_contracts_page() {
        // Cross-link: the missions page sends the player to /contracts/
        // for the canonical contract list.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix::default();
        let page = page_missions(&locale, &sirenix);
        assert!(
            page.contains("(../contracts/)"),
            "missions page must link to ../contracts/:\n{page}"
        );
    }

    #[test]
    fn contracts_page_links_to_missions_page() {
        // Cross-link the other direction: the contracts page sends the
        // player to /missions/ for mission-planning mechanics.
        let locale = contracts_fixture_locale();
        let sirenix = Sirenix::default();
        let page = page_contracts(&locale, &sirenix);
        assert!(
            page.contains("(../missions/)"),
            "contracts page must link to ../missions/:\n{page}"
        );
    }

    fn asteroid_taxonomy_locale() -> Locale {
        let mut locale = nav_fixture_locale();
        locale.resources.push(ResourceEntry { id: "volatile".into(), name: "Carbon".into() });
        locale.resources.push(ResourceEntry { id: "metal".into(), name: "Metals".into() });
        locale.resources.push(ResourceEntry { id: "water".into(), name: "Water".into() });
        locale.resources.push(ResourceEntry { id: "silicon".into(), name: "Silicon".into() });
        locale
    }

    fn carbon_class_stat() -> AsteroidClassStat {
        AsteroidClassStat {
            name: "Carbon".into(),
            tiers: vec![
                AsteroidTierStat {
                    category: "High".into(),
                    rolls: vec![AsteroidRollStat { resource_id: "volatile".into(), probability: 1.0 }],
                },
                AsteroidTierStat {
                    category: "Low".into(),
                    rolls: vec![
                        AsteroidRollStat { resource_id: "metal".into(), probability: 0.45 },
                        AsteroidRollStat { resource_id: "water".into(), probability: 0.45 },
                        AsteroidRollStat { resource_id: "silicon".into(), probability: 0.10 },
                    ],
                },
            ],
        }
    }

    // ---------- Achievements page ----------

    #[test]
    fn humanize_strips_prefix_and_splits_camel_case() {
        assert_eq!(humanize_achievement_id("id_achievement_NotToday"), "Not Today");
        assert_eq!(humanize_achievement_id("id_achievement_FirstOrbit"), "First Orbit");
        assert_eq!(humanize_achievement_id("id_achievement_LunarLanding"), "Lunar Landing");
    }

    #[test]
    fn humanize_handles_three_word_camel_case() {
        assert_eq!(
            humanize_achievement_id("id_achievement_AsteroidColony"),
            "Asteroid Colony"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_GravitationalSlingshot"),
            "Gravitational Slingshot"
        );
    }

    #[test]
    fn humanize_splits_stuck_connectors_against_preceding_word() {
        // Real ids in the dump jam lowercase connectors ("of", "on", "a")
        // against the preceding capitalized word.  The humanize helper
        // recognizes a fixed connector set and splits them off.
        assert_eq!(
            humanize_achievement_id("id_achievement_HumansonMars"),
            "Humans on Mars"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_OnWindsofSunshine"),
            "On Winds of Sunshine"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_FancyWayofThrowingRocks"),
            "Fancy Way of Throwing Rocks"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_ThePowerofaStar"),
            "The Power of a Star"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_DoAstronautsDreamofElectricShip"),
            "Do Astronauts Dream of Electric Ship"
        );
    }

    #[test]
    fn humanize_returns_id_when_no_prefix() {
        // Defensive: any future id that doesn't start with the prefix should
        // round-trip unchanged rather than corrupt-rename to "".
        assert_eq!(humanize_achievement_id("unexpected_id"), "unexpected_id");
    }

    #[test]
    fn humanize_does_not_mangle_words_that_end_in_connector_letters() {
        // Earlier heuristics tried to split on stuck connectors via pattern
        // matching, which produced "Beg in Terraforming" and "Moonbase Alph a".
        // The current implementation uses an override table for the known
        // concatenated ids and a plain CamelCase split for everything else —
        // so words like "Begin" and "Alpha" must come through intact.
        assert_eq!(
            humanize_achievement_id("id_achievement_BeginTerraforming"),
            "Begin Terraforming"
        );
        assert_eq!(
            humanize_achievement_id("id_achievement_MoonbaseAlpha"),
            "Moonbase Alpha"
        );
    }

    fn achievements_fixture_sirenix() -> Sirenix {
        Sirenix {
            achievements: vec![
                AchievementStat {
                    id: "id_achievement_NotToday".into(),
                    name: String::new(),
                    source_type: "contract".into(),
                    source_id: "contract_asteroid_impact".into(),
                    description: String::new(),
                    ..Default::default()
                },
                AchievementStat {
                    id: "id_achievement_FirstOrbit".into(),
                    name: String::new(),
                    source_type: "contract".into(),
                    source_id: "contract_tutorial_firstorbit".into(),
                    description: String::new(),
                    ..Default::default()
                },
                AchievementStat {
                    id: "id_achievement_ThePowerofaStar".into(),
                    name: String::new(),
                    source_type: "spacecraft".into(),
                    source_id: "spacecraft_fusion_large".into(),
                    description: String::new(),
                    ..Default::default()
                },
                AchievementStat {
                    id: "id_achievement_HeavyLifter".into(),
                    name: String::new(),
                    source_type: "launch_vehicle".into(),
                    source_id: "lv_chem_seadragon".into(),
                    description: String::new(),
                    ..Default::default()
                },
            ],
            ..Sirenix::default()
        }
    }

    fn achievements_fixture_locale() -> Locale {
        Locale {
            celestial_bodies: vec![],
            spacecraft: vec![NameDesc {
                id: "spacecraft_fusion_large".into(),
                name: "Sirius".into(),
                description: String::new(),
            }],
            launch_vehicles: vec![NameDesc {
                id: "lv_chem_seadragon".into(),
                name: "Albatross".into(),
                description: String::new(),
            }],
            research: vec![],
            corporations: vec![],
            contracts: vec![
                NameDesc {
                    id: "contract_asteroid_impact".into(),
                    name: "Asteroid Impact".into(),
                    description: String::new(),
                },
                NameDesc {
                    id: "contract_tutorial_firstorbit".into(),
                    name: "First Orbit".into(),
                    description: String::new(),
                },
            ],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        }
    }

    /// Build a habitability stat with the supplied (temperature, pressure,
    /// gravity, water) plus zeros for everything else — handy for tests.
    fn habit_stat(
        body_id: i32,
        body_name: &str,
        t: f64,
        p: f64,
        g: f64,
        w: f64,
    ) -> ScenarioBodyHabitabilityStat {
        ScenarioBodyHabitabilityStat {
            body_id,
            body_name: body_name.into(),
            temperature: t,
            pressure: p,
            gravity: g,
            water: w,
            ..Default::default()
        }
    }

    #[test]
    fn humanize_planet_type_strips_prefix_and_capitalizes() {
        assert_eq!(humanize_planet_type("planet_rocky_volcanic"), "Rocky volcanic");
        assert_eq!(humanize_planet_type("planet_rocky_barren"), "Rocky barren");
        assert_eq!(humanize_planet_type("planet_gas_gasgiant"), "Gas giant");
        assert_eq!(humanize_planet_type("planet_gas_ice"), "Gas ice");
        assert_eq!(humanize_planet_type("planet_rocky_eyeballHot"), "Rocky eyeball hot");
    }

    fn trappist_fixture_system() -> ExoplanetSystemStat {
        ExoplanetSystemStat {
            name: "Trappist-1".into(),
            id: "PlanetarySystem_Trappist".into(),
            star_type: "M8".into(),
            second_star_type: None,
            system_age: "Mature".into(),
            bodies: vec![
                ExoplanetBodyStat {
                    name: "TRAPPIST-1b".into(),
                    planet_type: "planet_rocky_volcanic".into(),
                    semi_major_axis_au: 0.0115, eccentricity: 0.02, inclination_deg: 1.0,
                    mass_1e24_kg: 8.18164, radius_km: 7390.36,
                },
                ExoplanetBodyStat {
                    name: "TRAPPIST-1c".into(),
                    planet_type: "planet_rocky_barren".into(),
                    semi_major_axis_au: 0.0158, eccentricity: 0.01, inclination_deg: 0.85,
                    mass_1e24_kg: 7.811376, radius_km: 6988.987,
                },
            ],
        }
    }

    #[test]
    fn scenario_state_page_emits_per_body_section_with_scenario_rows() {
        // One body (Mars) appearing in three scenarios — the page should
        // render a section header, a column header per stat, and one row
        // per scenario carrying that scenario's values.
        let sirenix = Sirenix {
            scenario_starts: vec![
                ScenarioStartStat {
                    scenario_id: "StartGameEpoch_TheExpansion".into(),
                    corp_starts: vec![],
                    body_habitability: vec![habit_stat(59, "Mars", -63.3, 0.006, 3.71, 0.0)],
                },
                ScenarioStartStat {
                    scenario_id: "StartGameEpoch_Colonization".into(),
                    corp_starts: vec![],
                    body_habitability: vec![habit_stat(59, "Mars", -62.5, 0.007, 3.71, 0.02)],
                },
                ScenarioStartStat {
                    scenario_id: "StartGameEpoch_RaceBeyond".into(),
                    corp_starts: vec![],
                    body_habitability: vec![habit_stat(59, "Mars", -60.0, 0.020, 3.71, 0.15)],
                },
            ],
            ..Default::default()
        };
        let page = page_scenario_state(&sirenix);

        // Section header for Mars.
        assert!(
            page.contains("## Mars\n"),
            "expected `## Mars` section header:\n{page}"
        );

        // All four scenario labels must appear in the body's table, with the
        // Early Exploration row falling back to em-dashes since it isn't
        // present in the sirenix.scenario_starts list.
        for label in [
            "Early Exploration",
            "The Expansion",
            "Colonization Era",
            "Race Beyond",
        ] {
            assert!(
                page.contains(label),
                "expected scenario label `{label}`:\n{page}"
            );
        }

        // The Mars Colonization row must carry the actual values we passed
        // in. Take the line that's a table row (starts with `|`) and contains
        // the bolded Colonization label — the intro paragraph mentions the
        // label too but isn't a table row.
        let line = page
            .lines()
            .find(|l| l.starts_with("| ") && l.contains("**Colonization Era**"))
            .expect("Colonization Era table row");
        assert!(
            line.contains("-62.5"),
            "Mars temperature missing from Colonization row: {line}"
        );
        assert!(
            line.contains("0.007"),
            "Mars pressure missing from Colonization row: {line}"
        );
        assert!(
            line.contains("3.71"),
            "Mars gravity missing from Colonization row: {line}"
        );

        // No internal `StartGameEpoch_*` ids should appear as visible cells.
        for l in page.lines() {
            if l.starts_with("| ") && l.contains("StartGameEpoch_") {
                panic!("internal epoch id leaked into a player-facing cell: {l}");
            }
        }
    }

    #[test]
    fn asteroid_taxonomy_page_renders_carbon_high_volatiles_at_100_percent() {
        let locale = asteroid_taxonomy_locale();
        let sirenix = Sirenix {
            asteroid_classes: vec![carbon_class_stat()],
            ..Default::default()
        };
        let page = page_asteroid_taxonomy(&locale, &sirenix);
        assert!(page.contains("# Asteroid Taxonomy"), "page should start with the H1 heading:\n{page}");
        assert!(page.contains("## Carbon Asteroid"), "page should have a section heading per class:\n{page}");
        assert!(page.contains("| High | Carbon | 100% |"), "High tier row should show Carbon at 100%:\n{page}");
    }

    #[test]
    fn exoplanets_page_renders_trappist_with_humanized_types_and_eccentricity() {
        let sirenix = Sirenix {
            exoplanet_systems: vec![trappist_fixture_system()],
            ..Sirenix::default()
        };
        let page = page_exoplanets_systems(&sirenix);
        assert!(page.contains("# Exoplanet Systems"));
        assert!(page.contains("## Trappist-1"));
        assert!(page.contains("Rocky barren"));
        assert!(page.contains("Rocky volcanic"));
        assert!(page.contains("0.0100"));
        assert!(page.contains("**Host star:** M8"));
    }

    #[test]
    fn asteroid_taxonomy_page_renders_low_tier_with_three_rolls() {
        let locale = asteroid_taxonomy_locale();
        let sirenix = Sirenix {
            asteroid_classes: vec![carbon_class_stat()],
            ..Default::default()
        };
        let page = page_asteroid_taxonomy(&locale, &sirenix);
        // 0.45 → "45%", 0.10 → "10%".
        assert!(
            page.contains("| Low | Metals | 45% |"),
            "Low tier should list Metals at 45%:\n{page}"
        );
        assert!(
            page.contains("| Low | Water | 45% |"),
            "Low tier should list Water at 45%:\n{page}"
        );
        assert!(
            page.contains("| Low | Silicon | 10% |"),
            "Low tier should list Silicon at 10%:\n{page}"
        );
    }

    #[test]
    fn asteroid_taxonomy_page_skips_empty_tiers() {
        // Carbon class has no Mid tier — the page must not emit a row
        // for it (the parser already filters empty tiers; the renderer
        // just trusts the input list).
        let locale = asteroid_taxonomy_locale();
        let sirenix = Sirenix {
            asteroid_classes: vec![carbon_class_stat()],
            ..Default::default()
        };
        let page = page_asteroid_taxonomy(&locale, &sirenix);
        assert!(
            !page.contains("| Mid |"),
            "Carbon class has no Mid tier — page must not emit a Mid row:\n{page}"
        );
    }

    #[test]
    fn asteroid_taxonomy_page_lists_each_class_alphabetically() {
        // Multiple classes should each get their own H2 section.
        let locale = asteroid_taxonomy_locale();
        let dark = AsteroidClassStat {
            name: "Dark".into(),
            tiers: vec![AsteroidTierStat {
                category: "High".into(),
                rolls: vec![AsteroidRollStat {
                    resource_id: "water".into(),
                    probability: 1.0,
                }],
            }],
        };
        let sirenix = Sirenix {
            asteroid_classes: vec![carbon_class_stat(), dark],
            ..Default::default()
        };
        let page = page_asteroid_taxonomy(&locale, &sirenix);
        let carbon_idx = page.find("## Carbon Asteroid").expect("Carbon section");
        let dark_idx = page.find("## Dark Asteroid").expect("Dark section");
        assert!(
            carbon_idx < dark_idx,
            "classes should be listed alphabetically:\n{page}"
        );
    }

    #[test]
    fn scenario_state_page_filters_out_unresolved_numeric_ids() {
        // Asteroids and exotic bodies aren't in tabObjectInfoData and end up
        // with body_name == "<id>". They must NOT render as section headers —
        // a `## 223` heading is noise.
        let sirenix = Sirenix {
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_Colonization".into(),
                corp_starts: vec![],
                body_habitability: vec![
                    habit_stat(66, "Earth", 15.0, 1.0, 9.79, 0.7),
                    habit_stat(223, "223", -130.0, 0.0, 0.0, 0.0),
                ],
            }],
            ..Default::default()
        };
        let page = page_scenario_state(&sirenix);
        assert!(page.contains("## Earth\n"), "Earth section expected:\n{page}");
        assert!(
            !page.contains("## 223\n"),
            "numeric-id sections must be suppressed:\n{page}"
        );
    }

    #[test]
    fn asteroid_taxonomy_page_renames_helium3_display_label() {
        // "Helium3" → "Helium-3" for the H2 heading (matches the locale's
        // resource label).
        let locale = asteroid_taxonomy_locale();
        let h3 = AsteroidClassStat {
            name: "Helium3".into(),
            tiers: vec![AsteroidTierStat {
                category: "Low".into(),
                rolls: vec![AsteroidRollStat {
                    resource_id: "hel3".into(),
                    probability: 1.0,
                }],
            }],
        };
        let sirenix = Sirenix {
            asteroid_classes: vec![h3],
            ..Default::default()
        };
        let page = page_asteroid_taxonomy(&locale, &sirenix);
        assert!(
            page.contains("## Helium-3 Asteroid"),
            "Helium3 class should render as Helium-3 Asteroid:\n{page}"
        );
    }

    #[test]
    fn scenario_state_page_orders_bodies_by_planet_then_moons() {
        // Section order: planet → its moons → next planet → ... The natural
        // canonical order so the page reads from the inner solar system out.
        let sirenix = Sirenix {
            scenario_starts: vec![ScenarioStartStat {
                scenario_id: "StartGameEpoch_Colonization".into(),
                corp_starts: vec![],
                body_habitability: vec![
                    habit_stat(66, "Earth", 15.0, 1.0, 9.79, 0.7),
                    habit_stat(87, "Moon", 40.0, 0.0, 1.62, 0.0),
                    habit_stat(59, "Mars", -63.0, 0.006, 3.71, 0.0),
                    habit_stat(89, "Phobos", -40.0, 0.0, 0.005, 0.0),
                ],
            }],
            ..Default::default()
        };
        let page = page_scenario_state(&sirenix);
        let pos = |needle: &str| page.find(needle).unwrap_or(usize::MAX);
        let p_earth = pos("## Earth\n");
        let p_moon = pos("## Moon\n");
        let p_mars = pos("## Mars\n");
        let p_phobos = pos("## Phobos\n");
        assert!(p_earth < p_moon, "Earth should precede Moon ({p_earth} < {p_moon})");
        assert!(p_moon < p_mars, "Moon should precede Mars ({p_moon} < {p_mars})");
        assert!(
            p_mars < p_phobos,
            "Mars should precede Phobos ({p_mars} < {p_phobos})"
        );
    }

    #[test]
    fn exoplanets_page_with_no_data_falls_back_to_explanatory_stub() {
        let sirenix = Sirenix::default();
        let page = page_exoplanets_systems(&sirenix);
        assert!(page.starts_with("# Exoplanet Systems"));
        assert!(page.contains("not available"), "empty-data page must explain the situation:\n{page}");
    }
    #[test]
    fn achievements_page_has_one_section_per_source_type() {
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(page.starts_with("# Achievements"), "got:\n{page}");
        assert!(page.contains("## By contract"), "missing contract section:\n{page}");
        assert!(page.contains("## By spacecraft"), "missing spacecraft section:\n{page}");
        assert!(page.contains("## By launch vehicle"), "missing LV section:\n{page}");
    }

    #[test]
    fn achievements_page_resolves_contract_source_via_locale() {
        // The "How to earn" cell shows the player-facing contract name from
        // locale, NOT the raw `contract_*` id.
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("Asteroid Impact"), "missing contract name:\n{page}");
        assert!(page.contains("First Orbit"), "missing First Orbit:\n{page}");
        assert!(
            !page.contains("contract_asteroid_impact"),
            "leaked raw contract id:\n{page}"
        );
    }

    #[test]
    fn achievements_page_resolves_spacecraft_source_via_locale() {
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("Sirius"), "missing spacecraft name:\n{page}");
        assert!(
            !page.contains("spacecraft_fusion_large"),
            "leaked raw spacecraft id:\n{page}"
        );
    }

    #[test]
    fn achievements_page_resolves_launch_vehicle_source_via_locale() {
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("Albatross"), "missing LV name:\n{page}");
        assert!(
            !page.contains("lv_chem_seadragon"),
            "leaked raw LV id:\n{page}"
        );
    }

    #[test]
    fn achievements_page_uses_humanized_achievement_id_when_no_locale_name() {
        // With no locale name available, the Achievement column should show a
        // humanized form of the id (split camelCase), not the raw id.
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("Not Today"), "missing 'Not Today':\n{page}");
        assert!(page.contains("First Orbit"), "missing 'First Orbit':\n{page}");
        assert!(
            !page.contains("id_achievement_NotToday"),
            "leaked raw achievement id:\n{page}"
        );
    }

    #[test]
    fn achievements_page_omits_empty_sections() {
        // When no LV achievements exist (real-world state of the dump as of
        // writing), the LV section should not render at all.
        let locale = achievements_fixture_locale();
        let sirenix = Sirenix {
            achievements: vec![AchievementStat {
                id: "id_achievement_NotToday".into(),
                name: String::new(),
                source_type: "contract".into(),
                source_id: "contract_asteroid_impact".into(),
                description: String::new(),
                ..Default::default()
            }],
            ..Sirenix::default()
        };
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("## By contract"));
        assert!(!page.contains("## By spacecraft"), "should skip empty SC section:\n{page}");
        assert!(!page.contains("## By launch vehicle"), "should skip empty LV section:\n{page}");
    }

    #[test]
    fn achievements_page_preferred_uses_locale_name_over_humanized_id() {
        // If a future dump *does* surface a locale name for the achievement,
        // the page should prefer it over the humanized id.  We seed the
        // `name` field directly on the AchievementStat to simulate this.
        let locale = achievements_fixture_locale();
        let sirenix = Sirenix {
            achievements: vec![AchievementStat {
                id: "id_achievement_NotToday".into(),
                name: "Not Today!".into(), // synthetic locale name w/ punctuation
                source_type: "contract".into(),
                source_id: "contract_asteroid_impact".into(),
                description: String::new(),
                ..Default::default()
            }],
            ..Sirenix::default()
        };
        let page = page_achievements(&locale, &sirenix);
        assert!(page.contains("Not Today!"), "expected locale name:\n{page}");
    }

    // ---------- New assertions for the reworked Achievements page ----------
    //
    // 1. Source name in the "How to earn" / "Trigger" cell must be a
    //    markdown link to the row on the corresponding page
    //    (../contracts/#contract-<id> or ../spacecraft/#spacecraft-<id>).
    // 2. The empty "Description" column is replaced by a "Condition" column
    //    that renders year deadlines and required prior contracts when the
    //    binding carries `conditions[]`.  Spacecraft achievements have no
    //    conditions in the dump so their section omits the column entirely.
    // 3. The intro paragraph no longer claims LV is a source, since no LV
    //    actually populates an inner achievement.

    #[test]
    fn achievements_page_links_contract_row_to_contracts_page() {
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        // The "How to earn" cell for a contract-sourced achievement must be
        // a markdown link to the row's anchor on /contracts/.  Underscores
        // in the source id are slugged to dashes by `anchor_id`.
        assert!(
            page.contains(
                "[Asteroid Impact (contract)](../contracts/#contract-contract-asteroid-impact)"
            ),
            "expected contract link in How-to-earn cell:\n{page}"
        );
        assert!(
            page.contains(
                "[First Orbit (contract)](../contracts/#contract-contract-tutorial-firstorbit)"
            ),
            "expected First Orbit link:\n{page}"
        );
    }

    #[test]
    fn achievements_page_links_spacecraft_row_to_spacecraft_page() {
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(
            page.contains(
                "[Sirius (spacecraft)](../spacecraft/#spacecraft-spacecraft-fusion-large)"
            ),
            "expected spacecraft link in Trigger cell:\n{page}"
        );
    }

    #[test]
    fn achievements_page_drops_description_column_in_favor_of_condition() {
        // The old Description column was empty for every row.  The new
        // Condition column carries year deadlines and required-contract
        // dependencies parsed from the binding `conditions[]`.
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        assert!(
            !page.contains("| Description |") && !page.contains("Description |\n"),
            "Description column must be removed:\n{page}"
        );
        assert!(
            page.contains("| Condition |") || page.contains("Condition |"),
            "Condition column must be present in at least one section:\n{page}"
        );
    }

    #[test]
    fn achievements_page_renders_year_deadline_condition() {
        let locale = achievements_fixture_locale();
        let sirenix = Sirenix {
            achievements: vec![AchievementStat {
                id: "id_achievement_Wanderlust".into(),
                source_type: "contract".into(),
                source_id: "contract_asteroid_impact".into(),
                conditions: vec![AchievementConditionStat {
                    required_contract: String::new(),
                    before_year: 2400,
                }],
                ..Default::default()
            }],
            ..Sirenix::default()
        };
        let page = page_achievements(&locale, &sirenix);
        assert!(
            page.contains("by year 2400") || page.contains("By year 2400"),
            "year deadline must surface in Condition cell:\n{page}"
        );
    }

    #[test]
    fn achievements_page_renders_required_contract_condition_as_link() {
        // A condition that references another contract should render as a
        // markdown link to that contract's row on /contracts/.
        let locale = achievements_fixture_locale();
        let sirenix = Sirenix {
            achievements: vec![AchievementStat {
                id: "id_achievement_MarsTerraformed".into(),
                source_type: "contract".into(),
                source_id: "contract_asteroid_impact".into(),
                conditions: vec![AchievementConditionStat {
                    required_contract: "contract_tutorial_firstorbit".into(),
                    before_year: 0,
                }],
                ..Default::default()
            }],
            ..Sirenix::default()
        };
        let page = page_achievements(&locale, &sirenix);
        // Player-facing name from locale, linking to the same-id anchor.
        assert!(
            page.contains(
                "[First Orbit](../contracts/#contract-contract-tutorial-firstorbit)"
            ),
            "required contract should be a link to its /contracts/ row:\n{page}"
        );
        assert!(
            !page.contains("contract_tutorial_firstorbit (contract)"),
            "should not show raw id in the prereq cell:\n{page}"
        );
    }

    #[test]
    fn achievements_page_spacecraft_section_omits_condition_column() {
        // No spacecraft-sourced achievement in the dump carries conditions,
        // so the Spacecraft table should be a 2-column (Achievement +
        // Trigger spacecraft) table without an empty Condition column.
        let locale = achievements_fixture_locale();
        let sirenix = Sirenix {
            achievements: vec![AchievementStat {
                id: "id_achievement_ThePowerofaStar".into(),
                source_type: "spacecraft".into(),
                source_id: "spacecraft_fusion_large".into(),
                ..Default::default()
            }],
            ..Sirenix::default()
        };
        let page = page_achievements(&locale, &sirenix);
        // Find the "## By spacecraft" section and its header row.
        let sc_idx = page
            .find("## By spacecraft")
            .expect("spacecraft section must exist");
        let after = &page[sc_idx..];
        // The header row of the spacecraft table comes right after the heading.
        let header_line = after
            .lines()
            .find(|l| l.starts_with("| Achievement"))
            .expect("header row must exist");
        assert!(
            !header_line.contains("Condition"),
            "spacecraft section header must not include Condition:\n{header_line}"
        );
    }

    #[test]
    fn achievements_page_intro_lists_only_populated_source_types() {
        // The intro paragraph should not claim LaunchVehicleType is a source
        // because no LV actually populates an inner achievement in the dump.
        let locale = achievements_fixture_locale();
        let sirenix = achievements_fixture_sirenix();
        let page = page_achievements(&locale, &sirenix);
        // Grab just the intro (text before the first H2).
        let intro_end = page.find("## ").unwrap_or(page.len());
        let intro = &page[..intro_end];
        assert!(
            !intro.contains("LaunchVehicleType"),
            "intro must not name LaunchVehicleType as a source:\n{intro}"
        );
        assert!(
            intro.contains("ContractDefinition") && intro.contains("SpacecraftType"),
            "intro must name the two real source tables:\n{intro}"
        );
    }

    #[test]
    fn root_readme_links_to_achievements_page() {
        let root = page_root();
        assert!(
            root.contains("achievements/"),
            "root README must link to achievements page:\n{root}"
        );
    }

    #[test]
    fn scenario_state_page_handles_empty_data_gracefully() {
        // No scenario starts → page still renders without panicking and
        // emits a sentinel notice instead of an unstructured wall of
        // markdown.
        let sirenix = Sirenix::default();
        let page = page_scenario_state(&sirenix);
        assert!(
            page.contains("No scenario data available"),
            "expected sentinel:\n{page}"
        );
    }

    // ---------- Task 1: Asteroid class column ----------

    #[test]
    fn asteroid_table_renders_class_column_with_taxonomy_link() {
        // A body whose `asteroid_class` is "Metal" should appear in the
        // asteroid table with a Class column linking to the per-class
        // anchor on the asteroid-taxonomy page.
        let locale = Locale {
            celestial_bodies: vec![
                CelestialBody { id: "Psyche".into(), name: "16 Psyche".into() },
            ],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let mut psyche = fixture_body("16 Psyche");
        psyche.asteroid_class = Some("Metal".into());
        let stats = Stats { bodies: vec![psyche] };
        let ctx = WikiCtx::build(&locale, &stats);

        let table = asteroid_table(&ctx, &["Psyche"]);
        // Header is "Class" (between Asteroid and Radius).
        assert!(
            table.contains("Class"),
            "asteroid table must include a Class column header:\n{table}"
        );
        // The Metal class cell links to the metal-asteroid anchor on the
        // asteroid-taxonomy page (sibling section).
        assert!(
            table.contains("../asteroid-taxonomy/#metal-asteroid"),
            "Metal-class row must link to #metal-asteroid:\n{table}"
        );
        // And a tooltip on the Class cell mentions the roll-table classes.
        assert!(
            table.contains("Mining roll table"),
            "Class cell tooltip should reference the mining roll table:\n{table}"
        );
    }

    #[test]
    fn asteroid_table_helium3_class_links_to_helium_3_anchor() {
        // Game's internal `Helium3` should render as `Helium-3` and link to
        // the GitHub-auto-generated `#helium-3-asteroid` anchor.
        let locale = Locale {
            celestial_bodies: vec![
                CelestialBody { id: "Apophis".into(), name: "99942 Apophis".into() },
            ],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let mut apophis = fixture_body("99942 Apophis");
        apophis.asteroid_class = Some("Helium3".into());
        let stats = Stats { bodies: vec![apophis] };
        let ctx = WikiCtx::build(&locale, &stats);

        let table = asteroid_table(&ctx, &["Apophis"]);
        assert!(
            table.contains("../asteroid-taxonomy/#helium-3-asteroid"),
            "Helium3 row must link to #helium-3-asteroid:\n{table}"
        );
        assert!(
            table.contains("Helium-3"),
            "Helium3 should display as 'Helium-3':\n{table}"
        );
    }

    #[test]
    fn asteroid_table_dashes_class_when_unknown() {
        // Bodies without a known class should render an em-dash in the
        // Class column rather than failing the build.  Per-asteroid class
        // isn't linked anywhere in the current dump, so this is the
        // expected path for every shipped asteroid today.
        let locale = Locale {
            celestial_bodies: vec![
                CelestialBody { id: "Ceres".into(), name: "1 Ceres".into() },
            ],
            spacecraft: vec![],
            launch_vehicles: vec![],
            research: vec![],
            corporations: vec![],
            contracts: vec![],
            resources: vec![],
            facilities: vec![],
            habitability_scales: BTreeMap::new(),
            cargo: vec![],
        };
        let ceres = fixture_body("1 Ceres"); // asteroid_class defaults to None
        let stats = Stats { bodies: vec![ceres] };
        let ctx = WikiCtx::build(&locale, &stats);

        let table = asteroid_table(&ctx, &["Ceres"]);
        // The Ceres row should still parse with a Class cell — em-dash for
        // unknown class (don't fail the build).
        let row = table
            .lines()
            .find(|l| l.contains("1 Ceres"))
            .expect("Ceres row present");
        let cells: Vec<&str> = row.split('|').map(|c| c.trim()).collect();
        // Leading empty cell, then: Asteroid | Class | Radius | a | e | i | trailing.
        // Class is index 2.
        let class_cell = cells.get(2).expect("Class cell present");
        assert_eq!(*class_cell, "—", "unknown class should render as em-dash; row:\n{row}");
    }

    // ---------- Task 3: Root README "How to use this wiki" rewrite ----------

    #[test]
    fn root_readme_how_to_use_section_is_player_facing() {
        // The "How to use this wiki" section should help players navigate
        // the wiki (Bodies / Spacecraft / Calculator), not describe the
        // generator implementation (Jekyll, sortable-table snippet).
        let root = page_root();
        // Find the "How to use this wiki" section and the next H2.
        let start = root
            .find("## How to use this wiki")
            .expect("How to use this wiki section present");
        let after_start = &root[start..];
        let end_rel = after_start[3..].find("\n## ").map(|i| i + 3).unwrap_or(after_start.len());
        let section = &after_start[..end_rel];

        // Player-navigation cues that should appear.
        for needle in ["Bodies", "Spacecraft", "Calculator"] {
            assert!(
                section.contains(needle),
                "How-to-use section should mention {needle}; section:\n{section}"
            );
        }
        // Generator implementation talk that should NOT appear.
        for forbidden in ["Jekyll", "sortable-table snippet"] {
            assert!(
                !section.contains(forbidden),
                "How-to-use section should not mention {forbidden} (generator-talk); section:\n{section}"
            );
        }
    }

    // ---------- Task 4: Footer version detection ----------

    #[test]
    fn detect_game_version_parses_bundle_version_from_project_settings() {
        // Given a Unity ProjectSettings.asset containing a
        // `  bundleVersion: 0.26.5.15.14 BETA` line, the helper should
        // surface that string so the wiki footer renders v0.26.5.15.14 BETA
        // rather than v"unknown".
        let dir = std::env::temp_dir().join("solar_expanse_wiki_version_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mktemp");
        let path = dir.join("ProjectSettings.asset");
        std::fs::write(
            &path,
            "  serializedVersion: 26\n  bundleVersion: 0.26.5.15.14 BETA\n  AndroidBundleVersionCode: 0\n",
        )
        .expect("write fixture");

        let v = detect_game_version_from_project_settings(&path);
        assert_eq!(v.as_deref(), Some("0.26.5.15.14 BETA"));
    }

    #[test]
    fn detect_game_version_returns_none_when_file_missing() {
        let path = std::env::temp_dir().join("solar_expanse_wiki_does_not_exist_xyz.asset");
        let _ = std::fs::remove_file(&path);
        assert!(detect_game_version_from_project_settings(&path).is_none());
    }
}

/// Parse the Unity engine's `bundleVersion` line out of a
/// `ProjectSettings.asset` file.  Unity writes the line as e.g.:
///
/// ```text
///   bundleVersion: 0.26.5.15.14 BETA
/// ```
///
/// Returns `Some(version_string)` (with leading/trailing whitespace
/// stripped) when found; `None` when the file is missing, unreadable, or
/// has no `bundleVersion:` line.  The gen-pages binary calls this when no
/// explicit `[game-version]` CLI arg is passed, so an interactive
/// developer running `cargo run --bin gen-pages …` directly still sees a
/// real version in the footer instead of `vunknown`.
fn detect_game_version_from_project_settings(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        // Unity indents the field by two spaces; tolerate any leading
        // whitespace (some Unity versions reflow on save).
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("bundleVersion:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Look for a Unity `ProjectSettings.asset` near the cache layout that
/// `extract.sh` produces and return its `bundleVersion`.  The cache lives
/// at `extract/cache/project/ExportedProject/ProjectSettings/ProjectSettings.asset`
/// relative to the workspace root, and `gen-pages` is typically invoked
/// with stats.json at `extract/cache/stats.json` — so we walk up from the
/// stats path and check the expected sibling layout.
fn auto_detect_game_version(stats_path: &Path) -> Option<String> {
    let cache_dir = stats_path.parent()?;
    let candidate = cache_dir
        .join("project")
        .join("ExportedProject")
        .join("ProjectSettings")
        .join("ProjectSettings.asset");
    detect_game_version_from_project_settings(&candidate)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 || args.len() > 6 {
        return Err(anyhow!(
            "usage: gen-pages <locale.json> <stats.json> <sirenix.json> <wiki-root> [game-version]"
        ));
    }
    let locale_path = PathBuf::from(&args[1]);
    let stats_path = PathBuf::from(&args[2]);
    let sirenix_path = PathBuf::from(&args[3]);
    let wiki_root = PathBuf::from(&args[4]);
    // Priority: explicit CLI arg → sibling ProjectSettings.asset → "unknown".
    let game_version = args
        .get(5)
        .cloned()
        .or_else(|| auto_detect_game_version(&stats_path))
        .unwrap_or_else(|| "unknown".to_string());

    let locale: Locale = serde_json::from_str(&fs::read_to_string(&locale_path)?)
        .with_context(|| format!("parsing {}", locale_path.display()))?;
    let stats: Stats = serde_json::from_str(&fs::read_to_string(&stats_path)?)
        .with_context(|| format!("parsing {}", stats_path.display()))?;
    let sirenix: Sirenix = if sirenix_path.exists() {
        serde_json::from_str(&fs::read_to_string(&sirenix_path)?)
            .with_context(|| format!("parsing {}", sirenix_path.display()))?
    } else {
        eprintln!("warning: {} not found; spacecraft page will be empty", sirenix_path.display());
        Sirenix::default()
    };
    let ctx = WikiCtx::build(&locale, &stats);

    write_file(&wiki_root, "README.md", &page_root())?;
    write_file(&wiki_root, "celestial-bodies/README.md", &page_celestial_index())?;
    write_file(&wiki_root, "celestial-bodies/planets.md", &page_planets(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/moons.md", &page_moons(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/asteroids.md", &page_asteroids(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/comets.md", &page_comets(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/exoplanets.md", &page_exoplanets(&ctx, &sirenix))?;
    write_file(&wiki_root, "celestial-bodies/launch-windows.md", &page_launch_windows(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/scenario-state.md", &page_scenario_state(&sirenix))?;
    write_file(&wiki_root, "spacecraft/README.md", &page_spacecraft(&locale, &sirenix))?;
    write_file(&wiki_root, "launch-vehicles/README.md", &page_launch_vehicles(&locale, &sirenix))?;
    write_file(&wiki_root, "facilities/README.md", &page_facilities(&locale, &sirenix))?;
    write_file(&wiki_root, "corporations/README.md", &page_corporations(&locale, &sirenix))?;
    write_file(&wiki_root, "resources/README.md", &page_resources(&locale, &sirenix))?;
    write_file(&wiki_root, "terraforming/README.md", &page_terraforming(&locale, &sirenix))?;
    write_file(&wiki_root, "contracts/README.md", &page_contracts(&locale, &sirenix))?;
    write_file(&wiki_root, "achievements/README.md", &page_achievements(&locale, &sirenix))?;
    write_file(&wiki_root, "research/README.md", &page_research(&locale, &sirenix))?;
    write_file(&wiki_root, "missions/README.md", &page_missions(&locale, &sirenix))?;
    write_file(&wiki_root, "asteroid-taxonomy/README.md", &page_asteroid_taxonomy(&locale, &sirenix))?;
    write_file(&wiki_root, "exoplanets/README.md", &page_exoplanets_systems(&sirenix))?;

    // Site metadata for the footer (Jekyll auto-loads _data/*.yml).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let generated_at = format_unix_date(now);
    let wiki_yaml = format!(
        "# Auto-generated by extract/src/bin/gen_pages.rs; do not edit by hand.\n\
game_version: \"{}\"\n\
generated_at: \"{}\"\n",
        game_version.replace('"', "\\\""),
        generated_at,
    );
    write_file(&wiki_root, "_data/wiki.yml", &wiki_yaml)?;
    Ok(())
}

/// Format a Unix timestamp as `YYYY-MM-DD` (UTC).  Used for the footer's
/// "generated on" date — we don't pull in chrono just for this.
fn format_unix_date(secs: u64) -> String {
    // Days since 1970-01-01 (a Thursday).
    let days = secs / 86_400;
    let mut y: i64 = 1970;
    let mut d = days as i64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let yd = if leap { 366 } else { 365 };
        if d < yd { break; }
        d -= yd;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mlen = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m: i64 = 0;
    while m < 12 && d >= mlen[m as usize] {
        d -= mlen[m as usize];
        m += 1;
    }
    format!("{:04}-{:02}-{:02}", y, m + 1, d + 1)
}

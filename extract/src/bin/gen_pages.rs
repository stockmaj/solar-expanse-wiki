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
}

#[derive(Deserialize, Clone)]
struct ResourceStat {
    id: String,
    resource_type: String,
    market_price_base: f64,
    show_on_ui: bool,
    #[allow(dead_code)]
    can_be_left_on_object: bool,
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
}

#[derive(Deserialize, Clone)]
struct ContractObjectiveStat {
    kind: String,
    quantity: f64,
    target: Option<String>,
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
    #[allow(dead_code)]
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
            vec![
                format!("**{display}**"),
                fmt_radius(radius),
                fmt_au(a),
                fmt_opt(e, 4),
                fmt_opt(i, 2),
            ]
        })
        .collect();
    md_table(
        &["Asteroid", "Radius (km)", "Semi-major axis (AU)", "Eccentricity", "Inclination (°)"],
        &rows,
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
- [Comets](comets.md)\n\
- [Celestial Bodies overview](README.md)\n"
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
            vec![format!("**{display}**")]
        })
        .collect();
    md_table(&["Planet"], &rows)
}

fn page_exoplanets(ctx: &WikiCtx) -> String {
    let trappist = exoplanet_table(ctx, EXOPLANETS_TRAPPIST);
    let kepler = exoplanet_table(ctx, EXOPLANETS_KEPLER);
    format!(
        "# Exoplanet Systems\n\n\
Distant star systems reachable only through interstellar travel via a generation\n\
ship constructed in the late game.\n\n\
## TRAPPIST-1\n\n\
A red dwarf star with seven terrestrial planets, several within its habitable zone.\n\n\
{trappist}\n\
## Kepler-90\n\n\
A Sun-like star with at least eight known planets, the only confirmed system that\n\
rivals the Solar System in planet count.\n\n\
{kepler}\n\
## See also\n\n\
- [Planets](planets.md)\n\
- [Celestial Bodies overview](README.md)\n"
    )
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
            let synodic_label = if *syn < 2.0 {
                format!("{:.0} days (~{:.1} months)", synodic_days, syn * 12.0)
            } else {
                format!("{:.1} years", syn)
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
    let mut calc_bodies: Vec<String> = Vec::new();
    // Earth first
    calc_bodies.push(format!(
        "{{\"name\":\"Earth\",\"a\":{a},\"longitude\":{lng}}}",
        a = earth_a,
        lng = earth.longitude_deg.unwrap_or(0.0),
    ));
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
**Jump to:** [Body table](#body-table) · [Gravity-assist trajectory](#gravity-assist)\n\n\
> **Heads-up:** these numbers are computed by the wiki from the orbital\n\
> elements the game ships, *not* read from the game itself.  The in-game\n\
> Plan Mission window uses live n-body propagation including gravitational\n\
> perturbations and your spacecraft's specific Δv budget, so the dates and\n\
> intervals here are a **planning approximation** — the porkchop plot is\n\
> the source of truth at launch time.\n\n\
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
<div id=\"body-table\" markdown=\"1\">\n\
<div class=\"body-filters\">\n\
<label>Filter: <input id=\"body-filter\" type=\"search\"></label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Planet\" checked> Planets</label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Asteroid\"> Asteroids</label>\n\
<label><input type=\"checkbox\" class=\"body-type-filter\" value=\"Comet\"> Comets</label>\n\
</div>\n\n\
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
- Asteroid-belt bodies sit between Mars and Jupiter — windows ~14–16 months.\n\n\
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
### Suggested trajectories\n\n\
These are well-known flyby routes the calculator picks out as advantageous\n\
versus a direct transfer in the same launch window.  Computed on page load —\n\
expect a second or two for the table to populate.\n\n\
<button id=\"ga-suggest-btn\" type=\"button\">Calculate suggestions</button>\n\
<div id=\"ga-suggestions\"><em>Click the button to compute — this runs entirely in your browser and may take 10–20 seconds for outer-planet routes.</em></div>\n\n\
### Custom trajectory\n\n\
<div class=\"calc\">\n\
<label>From: <input id=\"ga-from\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Earth\"></label>\n\
<label>Flyby: <input id=\"ga-flyby\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Venus\"></label>\n\
<label>To: <input id=\"ga-to\" list=\"calc-bodies\" autocomplete=\"off\" placeholder=\"Body name…\" value=\"Ceres\"></label>\n\
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
All natural objects in Solar Expanse — from the Sun and the nine planets, through\n\
moons and asteroid belts, out to comets and the Trappist-1 and Kepler-90\n\
exoplanet systems reachable in the late game.\n\n\
{count_table}\n\
## Orbital data\n\n\
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
- [Launch Windows](launch-windows.md) — synodic periods for planning Earth → body missions\n"
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
        rows.push(vec![
            format!(
                "{anchor}**{name}**",
                anchor = anchor_tag("spacecraft", &s.id),
                name = escape_cell(display_name)
            ),
            fmt_amount(s.mass),
            fmt_amount(s.cargo_capacity),
            fmt_amount(s.fuel_capacity),
            thrust,
            exhaust,
            fmt_reusability(s.reusability).into(),
            if s.built_in_orbit { "Orbit" } else { "Surface" }.into(),
            fmt_build_cost(&s.build_cost, &resource_name),
            fmt_amount(s.build_time_days),
            escape_cell(desc),
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
- **Built at** is where the craft is assembled: *Orbit* means it's built in an orbital shipyard and never lands; *Surface* means it's built on a planet's surface (some surface craft are full SSTOs, some are upper stages or ride a Launch Vehicle — see the description column).\n\n\
## See also\n\n\
- [Launch Vehicles](../launch-vehicles/) — surface-to-orbit lifters\n\
- [Research](../research/) — propulsion tech tree\n",
    );
    out
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
- **Launch cost** is the cash fee paid every launch; **Maintenance** is the daily upkeep cost while idle on the pad.\n\n\
## Alternative launch methods\n\n\
The game also models several non-rocket launch systems unlocked through\n\
research and built as facilities at the launch site:\n\n\
| Method | Notes |\n\
| --- | --- |\n\
| **Launch Pad** | Organized launch infrastructure, reduces launch cost. |\n\
| **MagRails** | Long ramp built atop suitable terrain, outfitted with MagLev tracks. |\n\
| **Mass Driver** | Set of superconducting electromagnetic accelerators able to launch payloads directly into orbit. |\n\
| **Magnetic Catapult** | Larger mass driver capable of launching payloads on interplanetary trajectories by itself. |\n\
| **Spin Launcher** | Launches payloads via extremely high rotary acceleration. |\n\
| **Space Elevator** | Supermaterial cable from surface to geostationary orbit. |\n\n\
## See also\n\n\
- [Spacecraft](../spacecraft/)\n\
- [Research](../research/) — Launch Vehicles tech category\n"
    )
}

fn page_corporations(locale: &Locale) -> String {
    let mut out = String::from(
        "# Corporations\n\n\
The five playable starting factions in Solar Expanse. Each opens with a\n\
different research head start and corporate flavor.\n\n",
    );
    for c in &locale.corporations {
        out.push_str(&format!("## {}\n\n{}\n\n", c.name, c.description));
        let traits = c.traits.replace("\\n", "\n");
        let traits = traits.trim();
        if !traits.is_empty() {
            out.push_str("**Traits:**\n");
            out.push_str(traits);
            out.push_str("\n\n");
        }
    }
    out.push_str("## See also\n\n- [Research](../research/) — starting research differs by corporation\n");
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
    // Map facility id → human name to surface where each resource is produced.
    let fac_name: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.name.as_str()))
        .collect();
    let fac_desc: BTreeMap<&str, &str> = locale
        .facilities
        .iter()
        .map(|f| (f.id.as_str(), f.description.as_str()))
        .collect();

    // For each resource, walk facility tooltips to find any that mention the resource by name.
    let producers_for = |resource_display: &str| -> Vec<String> {
        let lower = resource_display.to_lowercase();
        // Avoid matching very short or ambiguous resource names ("ice", "gas") — substring noise.
        if lower.len() < 4 {
            return Vec::new();
        }
        let mut hits: Vec<String> = Vec::new();
        for (fid, fname) in &fac_name {
            let desc = fac_desc.get(fid).copied().unwrap_or("");
            if desc.to_lowercase().contains(&lower) {
                hits.push(smart_title_case(fname));
            }
        }
        hits.sort();
        hits.dedup();
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

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|r| {
            let display = res_name.get(r.id.as_str()).copied().unwrap_or(r.id.as_str());
            let price = if r.market_price_base > 0.0 {
                fmt_abbrev(r.market_price_base)
            } else {
                "—".to_string()
            };
            let producers = producers_for(display);
            let prod_cell = if producers.is_empty() {
                "—".to_string()
            } else {
                producers.join("<br>")
            };
            let desc = res_desc
                .get(r.id.as_str())
                .map(|s| s.as_str())
                .unwrap_or("");
            vec![
                format!("**{}**", escape_cell(display)),
                r.resource_type.clone(),
                price,
                prod_cell,
                escape_cell(desc),
            ]
        })
        .collect();
    let table = md_table_with_tips(
        &[
            "Resource",
            "Type",
            "Price",
            "Producers",
            "Description",
        ],
        &[
            None,
            Some("Normal (physical), Energy (real-time power), or Human (colonists)"),
            Some("Starting market clearing price; supply and demand move it from there"),
            None,
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
- **Base market price** is the starting clearing price on the marketplace; supply and demand move it from there.\n\
- **Produced by** is inferred from facility tooltip text — if a facility's description mentions the resource by name, it's listed. The actual produce-rate isn't extractable from the static descriptors (it lives on dynamic facility subclasses); the in-game tooltip is the source of truth for rate numbers.\n"
    )
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

    // Display order: by topological depth (roots first), then by display name.
    entries.sort_by(|a, b| {
        let da = depth.get(a.id.as_str()).copied().unwrap_or(0);
        let db = depth.get(b.id.as_str()).copied().unwrap_or(0);
        let name_a = contract_name.get(a.id.as_str()).copied().unwrap_or(a.id.as_str());
        let name_b = contract_name.get(b.id.as_str()).copied().unwrap_or(b.id.as_str());
        da.cmp(&db).then_with(|| name_a.cmp(name_b))
    });

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|c| {
            let display = contract_name.get(c.id.as_str()).copied().unwrap_or(c.id.as_str());
            let premise = contract_premise.get(c.id.as_str()).copied().unwrap_or("");
            let premise = if premise.len() > 240 {
                let cut = premise
                    .char_indices()
                    .nth(240)
                    .map(|(i, _)| i)
                    .unwrap_or(premise.len());
                format!("{}…", &premise[..cut])
            } else {
                premise.to_string()
            };

            // Objectives: dedupe identical lines (same kind + target + qty).
            let mut obj_bits: Vec<String> = Vec::new();
            let mut seen_obj: std::collections::BTreeSet<String> = Default::default();
            for o in &c.objectives {
                let pretty_target = o.target.as_deref().map(|t| {
                    if let Some(rest) = t.strip_prefix("build_") {
                        smart_title_case(fac_name.get(rest).copied().unwrap_or(rest))
                    } else if let Some(rest) = t.strip_prefix("id_resource_") {
                        resource_name.get(rest).copied().unwrap_or(rest).to_string()
                    } else if let Some(rest) = t.strip_prefix("module_") {
                        smart_title_case(&rest.replace('_', " "))
                    } else {
                        sc_name
                            .get(t)
                            .copied()
                            .or_else(|| lv_name.get(t).copied())
                            .unwrap_or(t)
                            .to_string()
                    }
                });
                let line = match (o.kind.as_str(), pretty_target.as_deref()) {
                    ("BuildFacility", Some(t)) => format!("Build {}× {}", fmt_amount(o.quantity), t),
                    ("Possession", Some(t)) => format!("Have {}× {}", fmt_amount(o.quantity), t),
                    ("Possession", None) => format!("Possess {}", fmt_amount(o.quantity)),
                    ("MarketsOffers", Some(t)) => format!("Market trade {}× {}", fmt_amount(o.quantity), t),
                    ("ChangeHabitabilityParameters", _) => "Adjust habitability parameter".into(),
                    ("ChangeDepositParameters", Some(t)) => format!("Survey {} deposit", t),
                    ("Exploration", _) => "Explore".into(),
                    (kind, Some(t)) => format!("{}: {}× {}", kind, fmt_amount(o.quantity), t),
                    (kind, None) => kind.into(),
                };
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
                if pretty != *u {
                    reward_bits.push(format!("Next: {}", pretty));
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
            // is offered.  Built from the reverse-rewards lookup.
            let prereq_cell = match unlocked_by.get(c.id.as_str()) {
                None => "—".to_string(),
                Some(srcs) => srcs
                    .iter()
                    .map(|src| {
                        let pretty = contract_name.get(src).copied().unwrap_or(src);
                        link_same_page("contract", src, &escape_cell(pretty))
                    })
                    .collect::<Vec<_>>()
                    .join("<br>"),
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
    format!(
        "# Contracts\n\n\
Contracts drive progression in Solar Expanse — they're the game's source of\n\
funding alongside resource sales. Each contract is a set of objectives — usually\n\
\"build facility X on body Y\" or \"deliver Z to a destination\" — that pay\n\
out cash, resources, or unlocks when complete. Many contracts also unlock\n\
the next link in a chain (Mars Phase 1 → Mars Phase 2 → …), a new spacecraft,\n\
or a new launch vehicle.\n\n\
{table}\n\
## Reading the table\n\n\
- Rows are sorted by **Order** — the contract's dependency depth in the unlock DAG. Order 0 contracts are roots with no prerequisite; Order 1 is unlocked by an Order 0 contract; and so on. Ties break alphabetically.\n\
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
        // Split by where the facility can be placed.  "Orbit" → orbital section;
        // "Surface" → ground section; "Surface, Orbit" appears in both.
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
            // Placement empty / unknown — show on ground as default rather than drop.
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
        let display = smart_title_case(raw_display);
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
            name = escape_cell(&display)
        );
        vec![
            name_cell,
            f.facility_type.clone(),
            fmt_build_cost(&f.build_cost, resource_name),
            workers,
            energy,
            maint,
            prereq,
            escape_cell(desc),
        ]
    };

    let header = [
        "Facility",
        "Type",
        "Build cost",
        "Workers",
        "Energy",
        "Maint",
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

    let header_tips: [Option<&str>; 8] = [
        None,
        Some("Facility category — Production / Mining / Power / Habitation / …"),
        Some("Resources required to construct"),
        Some("On-site population required for full output"),
        Some("Energy consumed per day"),
        Some("Daily maintenance cost"),
        Some("Research that unlocks this facility"),
        None,
    ];
    let ground_table = md_table_with_tips(&header, &header_tips, &ground_rows);
    let orbital_table = md_table_with_tips(&header, &header_tips, &orbital_rows);

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
{ground_table}\n\
## Orbital modules\n\n\
{orbital_table}\n\
## Reading the table\n\n\
- **Type** is the gameplay category — *Production*, *Mining*, *Storage*, *Power*, *Habitat*, etc. The Solar Expanse UI groups facilities by type when you open the build menu.\n\
- **Workers** is the on-site population the facility needs to operate at full output. Most facilities throttle when understaffed.\n\
- **Energy/day** is the running energy demand. Power facilities show this as `—`; everything else is a consumer.\n\
- **Maintenance** is the per-day cash upkeep while the facility is active.\n\
- **Research prereq** is the research that unlocks construction; `—` means it's available from the start (or the prereq lives outside the standard `lockByHelpNotUse` field, which a few specialist facilities use).\n\n\
What this page does *not* show: per-facility produces / consumes rates and special-effect bonuses. Those are stored on dynamically-typed subclasses of each facility and aren't in the static descriptor data — the in-game tooltip is the source of truth for now.\n"
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

fn fmt_research_unlock(
    r: &ResearchStat,
    facility_name: &BTreeMap<&str, &str>,
    spacecraft_name: &BTreeMap<&str, &str>,
    lv_name: &BTreeMap<&str, &str>,
) -> String {
    match r.action.as_str() {
        "UnlockFacility" => {
            let target = r.unlock_target.as_deref().unwrap_or("");
            // Facility unlock targets are full ids like "build_outpost"; locale ids are bare ("outpost").
            let key = target.strip_prefix("build_").unwrap_or(target);
            let pretty = smart_title_case(facility_name.get(key).copied().unwrap_or(key));
            let link = link_cross_page("facilities", "facility", key, &format!("**{pretty}**"));
            format!("Builds {link}")
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
                    format!(" on {}", r.bonus_components.join(", "))
                };
                format!("+{} {}{}", fmt_amount(r.bonus_amount), b, comps)
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

    // Every research node a player can interact with goes on the page.  `showInTree`
    // is the game's in-tree-header flag, not a "should-this-be-public" flag.
    let visible: Vec<&ResearchStat> = sirenix.research.iter().collect();

    // Bucket by branch then subbranch
    let mut by_branch: BTreeMap<&str, BTreeMap<&str, Vec<&ResearchStat>>> = BTreeMap::new();
    for r in &visible {
        by_branch
            .entry(r.branch.as_str())
            .or_default()
            .entry(r.subbranch.as_str())
            .or_default()
            .push(*r);
    }

    let mut out = String::from(
        "# Research\n\n\
The tech tree drives progression. Every research node has a work-hours cost,\n\
zero or more prerequisite research nodes, and unlocks something — a new\n\
facility, spacecraft, launch vehicle, or a numeric bonus on existing\n\
equipment. Research is grouped into three top-level branches (Engineering,\n\
Physics, Biotech), each subdivided into focused sub-branches.\n\n",
    );

    for (branch, subs) in &by_branch {
        out.push_str(&format!("## {}\n\n", pretty_branch(branch)));
        for (sub, items) in subs {
            if sub.is_empty() {
                continue;
            }
            out.push_str(&format!("### {}\n\n", sub));
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
                        prereqs,
                        fmt_research_unlock(r, &facility_name, &spacecraft_name, &lv_name),
                        escape_cell(&desc_for(&r.id)),
                    ]
                })
                .collect();
            out.push_str(&md_table_with_tips(
                &["Research", "Cost (h)", "Prereqs", "Unlocks", "Description"],
                &[
                    None,
                    Some("Cost in work-hours; divide by your labs' research output to get the actual research time in days"),
                    None,
                    None,
                    None,
                ],
                &rows,
            ));
            out.push('\n');
        }
    }

    out.push_str(
        "## Reading the table\n\n\
- **Cost** is in work hours and is divided by your laboratories' research output to get the actual research time in days.\n\
- **Prerequisites** must be completed before the node becomes available.\n\
- **Unlocks** — *Builds X* means the node makes a new facility constructable; *Spacecraft / Launch Vehicle* means the node unlocks a new ship or lifter; numeric bonuses apply to listed components.\n\n\
## See also\n\n\
- [Spacecraft](../spacecraft/) — propulsion research feeds directly into these\n\
- [Launch Vehicles](../launch-vehicles/)\n",
    );
    out
}

fn page_missions(locale: &Locale, sirenix: &Sirenix) -> String {
    // The Missions page combines the planning-flow primer with the in-game
    // contracts list (the game's word for what drives progression).
    let contracts_table = page_contracts(locale, sirenix);
    // Strip the "# Contracts\n\n…\n\n" preamble — we want just the table + reading notes.
    let table_only = contracts_table
        .find("\n\n|")
        .map(|i| &contracts_table[i..])
        .unwrap_or(&contracts_table)
        .trim_start();

    format!(
        "# Missions\n\n\
This page covers two related concepts, both of which the game calls\n\
\"missions\" depending on context.\n\n\
1. **Contracts** — the in-game *Contracts* tab, listed below.  A contract is\n\
   a fixed set of objectives that pays out cash, resources, or unlocks when\n\
   complete.  Contracts are the primary source of progression and funding\n\
   in single-player.\n\
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
## Funding missions (contracts)\n\n\
{table_only}\n\
## See also\n\n\
- [Spacecraft](../spacecraft/)\n\
- [Launch Vehicles](../launch-vehicles/)\n\
- [Launch Windows](../celestial-bodies/launch-windows.md)\n"
    )
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
| [Spacecraft](spacecraft/) | Interplanetary craft — Iris, Selene, Stratos, Hermes, Centaur, Athena, Prometheus, Hephaistos, Ariane, Cronos, Nike, Sirius, Zeus. |\n\
| [Launch Vehicles](launch-vehicles/) | Surface-to-orbit lifters — Albatross, Pelican, Magpie, Condor, Teratorn. |\n\
| [Facilities](facilities/) | Ground buildings and orbital modules — power, mining, refining, habitats, life support, etc. |\n\
| [Research](research/) | Tech tree — chemical, electric, nuclear, fusion propulsion, life support, materials, computing. |\n\
| [Missions](missions/) | Mission planning — landings, flybys, gravity assists, asteroid pulling, cyclical routes. |\n\
| [Contracts](contracts/) | Story and freelance contracts that drive progression. |\n\
| [Resources](resources/) | The 20+ resource types — water, metals, fissiles, He-3, supplies, exotic alloys. |\n\
| [Corporations](corporations/) | Playable starting factions — SoleX, NASA, ESA, CNSA, Roscosmos. |\n\n\
## How to use this wiki\n\n\
Every page is plain Markdown that renders directly on GitHub — no static-site\n\
generator, no theme, no JS. Browse by clicking section links above.\n\n\
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
    let game_version = args.get(5).cloned().unwrap_or_else(|| "unknown".to_string());

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
    write_file(&wiki_root, "celestial-bodies/exoplanets.md", &page_exoplanets(&ctx))?;
    write_file(&wiki_root, "celestial-bodies/launch-windows.md", &page_launch_windows(&ctx))?;
    write_file(&wiki_root, "spacecraft/README.md", &page_spacecraft(&locale, &sirenix))?;
    write_file(&wiki_root, "launch-vehicles/README.md", &page_launch_vehicles(&locale, &sirenix))?;
    write_file(&wiki_root, "facilities/README.md", &page_facilities(&locale, &sirenix))?;
    write_file(&wiki_root, "corporations/README.md", &page_corporations(&locale))?;
    write_file(&wiki_root, "resources/README.md", &page_resources(&locale, &sirenix))?;
    write_file(&wiki_root, "contracts/README.md", &page_contracts(&locale, &sirenix))?;
    write_file(&wiki_root, "research/README.md", &page_research(&locale, &sirenix))?;
    write_file(&wiki_root, "missions/README.md", &page_missions(&locale, &sirenix))?;

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

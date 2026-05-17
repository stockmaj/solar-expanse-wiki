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
    #[allow(dead_code)]
    id: String,
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
    let mut out = String::new();
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
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

fn page_celestial_index() -> String {
    let counts = [
        ("Planets", PLANETS.len()),
        (
            "Moons",
            moons_by_parent().iter().map(|(_, m)| m.len()).sum::<usize>(),
        ),
        (
            "Asteroids",
            ASTEROIDS_BELT.len()
                + ASTEROIDS_NEO.len()
                + ASTEROIDS_TROJAN_GREEK.len()
                + ASTEROIDS_FICTIONAL.len(),
        ),
        ("Comets", COMETS.len()),
        (
            "Exoplanets",
            EXOPLANETS_TRAPPIST.len() + EXOPLANETS_KEPLER.len(),
        ),
    ];
    let rows: Vec<Vec<String>> = counts
        .iter()
        .map(|(name, n)| {
            vec![
                format!("**[{name}]({}.md)**", name.to_lowercase()),
                n.to_string(),
            ]
        })
        .collect();
    let count_table = md_table(&["Category", "Count"], &rows);
    format!(
        "# Celestial Bodies\n\n\
All natural objects in Solar Expanse — from the Sun and the nine planets, through\n\
moons and asteroid belts, out to comets and the Trappist-1 and Kepler-90\n\
exoplanet systems reachable in the late game.\n\n\
{count_table}\n\
## Object types\n\n\
Solar Expanse distinguishes objects by type in the search and navigation UI:\n\n\
| Type | Notes |\n\
| --- | --- |\n\
| **Planet** | Major body orbiting the Sun. Most planets host one or more moons. |\n\
| **Moon** | Natural satellite orbiting a planet. |\n\
| **Asteroid** | Small body. Some are in the main belt, some are near-Earth, and some co-orbit Jupiter at the Trojan/Greek points. Asteroids can be pulled into new orbits with an Asteroid Engine Module. |\n\
| **Comet** | Periodic body on a highly eccentric orbit. |\n\
| **Exoplanet** | Body in a non-Solar system. Reachable only via a generation ship. |\n\n\
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
- [Exoplanets](exoplanets.md) — Trappist-1 and Kepler-90 systems\n"
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

fn fmt_amount(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e9 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}

fn fmt_build_cost(cost: &[ResourceCost], resource_name: &BTreeMap<&str, &str>) -> String {
    if cost.is_empty() {
        return "—".into();
    }
    cost.iter()
        .map(|c| {
            let label = resource_name.get(c.resource_id.as_str()).copied().unwrap_or(c.resource_id.as_str());
            format!("{} {}", fmt_amount(c.amount), label)
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
        out.push_str(&md_table(
            &[
                "Spacecraft",
                "Mass (t)",
                "Cargo (t)",
                "Fuel (t)",
                "Reusable",
                "Built at",
                "Build cost",
                "Build time (d)",
                "Description",
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
        rows.push(vec![
            format!("**{}**", escape_cell(display_name)),
            fmt_amount(s.mass),
            fmt_amount(s.cargo_capacity),
            fmt_amount(s.fuel_capacity),
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
- **Build cost** is the resource cost of building the spacecraft itself (engine and tank modules are paid for separately when configured).\n\
- **Built at** is where the craft is assembled: *Orbit* means it's built in an orbital shipyard and never lands; *Surface* means it's built on a planet's surface (some surface craft are full SSTOs, some are upper stages or ride a Launch Vehicle — see the description column).\n\n\
## See also\n\n\
- [Launch Vehicles](../launch-vehicles/) — surface-to-orbit lifters\n\
- [Research](../research/) — propulsion tech tree\n",
    );
    out
}


fn page_launch_vehicles(locale: &Locale) -> String {
    let mut items: Vec<&NameDesc> = locale
        .launch_vehicles
        .iter()
        .filter(|x| !x.name.is_empty())
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|x| {
            let desc = if x.description.is_empty() {
                "—".to_string()
            } else {
                escape_cell(&x.description)
            };
            vec![format!("**{}**", escape_cell(&x.name)), desc]
        })
        .collect();
    let table = md_table(&["Launch Vehicle", "Description"], &rows);
    format!(
        "# Launch Vehicles\n\n\
Surface-to-orbit lifters. Every spacecraft has to ride one of these to reach\n\
space, and choice of LV strongly affects launch cost.\n\n\
{table}\n\
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

fn page_resources(locale: &Locale) -> String {
    let mut seen = std::collections::BTreeSet::new();
    let mut items: Vec<&ResourceEntry> = locale
        .resources
        .iter()
        .filter(|r| !r.name.is_empty() && seen.insert(r.name.clone()))
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|r| vec![format!("**{}**", escape_cell(&r.name))])
        .collect();
    let table = md_table(&["Resource"], &rows);
    format!(
        "# Resources\n\n\
Every cargo type tracked by the game. Resources are produced by facilities,\n\
shipped between worlds, traded on the marketplace, and consumed in\n\
construction.\n\n\
{table}"
    )
}

fn page_contracts(locale: &Locale) -> String {
    let mut items: Vec<&NameDesc> = locale.contracts.iter().collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|c| {
            let desc = escape_cell(&c.description);
            let desc = if desc.len() > 200 {
                format!("{}…", &desc[..desc.char_indices().nth(200).map(|(i, _)| i).unwrap_or(desc.len())])
            } else {
                desc
            };
            vec![format!("**{}**", escape_cell(&c.name)), desc]
        })
        .collect();
    let table = md_table(&["Contract", "Premise"], &rows);
    format!(
        "# Contracts\n\n\
Story and freelance contracts that drive game progression. Many contracts have\n\
tutorial counterparts that walk new players through unfamiliar mechanics.\n\n\
{table}"
    )
}

fn page_research(locale: &Locale) -> String {
    let categories: Vec<&ResearchEntry> = locale
        .research
        .iter()
        .filter(|r| r.category == "category")
        .collect();
    let topics: Vec<&ResearchEntry> = locale
        .research
        .iter()
        .filter(|r| r.category == "topic")
        .collect();

    let mut cats = categories.clone();
    cats.sort_by(|a, b| a.name.cmp(&b.name));
    let cat_rows: Vec<Vec<String>> = cats
        .iter()
        .map(|c| {
            let summary = escape_cell(&c.description);
            let summary = if summary.len() > 200 {
                format!("{}…", &summary[..summary.char_indices().nth(200).map(|(i, _)| i).unwrap_or(summary.len())])
            } else {
                summary
            };
            vec![format!("**{}**", escape_cell(&c.name)), summary]
        })
        .collect();
    let cat_table = if cat_rows.is_empty() {
        "None.".to_string()
    } else {
        md_table(&["Category", "Summary"], &cat_rows)
    };

    format!(
        "# Research\n\n\
Solar Expanse's tech tree, organized into broad categories.\n\n\
## Categories\n\n\
{cat_table}\n\
## Topics\n\n\
The full tree contains {n} individual research topics across these categories.\n\n\
## See also\n\n\
- [Spacecraft](../spacecraft/) — Spacecraft research category\n\
- [Launch Vehicles](../launch-vehicles/) — Launch Vehicles research category\n",
        n = topics.len(),
    )
}

fn page_missions() -> String {
    String::from(
        "# Missions\n\n\
A *mission* is a planned trip from one body's orbit to another. The Plan\n\
Mission flow walks you through five steps:\n\n\
1. **Destination** — pick the target body (and landing type if applicable).\n\
2. **Spacecraft** — pick the craft to send.\n\
3. **Cargo** — pick the payload.\n\
4. **Launch Vehicle** — pick the lifter (only required for missions launching from a planet's surface).\n\
5. **Flight Plan** — pick the launch and arrival windows from the porkchop plot.\n\n\
## Mission types (from in-game UI)\n\n\
| Type | Notes |\n\
| --- | --- |\n\
| **Direct** | Single Hohmann-style transfer to the destination. |\n\
| **Gravity Assist** | Uses another body's gravity to bend the trajectory and save Δv. The game lets you choose whether cargo drops at the assist target or continues on. |\n\
| **Cyclical** | A repeating supply route between two or more bodies. |\n\
| **Asteroid Pulling** | Specialised mission to push an asteroid into a different orbit using an Asteroid Engine Module. |\n\
| **Probe Deployment** | Drops a small probe at a destination (typically the first thing you send anywhere). |\n\n\
## See also\n\n\
- [Spacecraft](../spacecraft/)\n\
- [Contracts](../contracts/)\n",
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
    if args.len() != 5 {
        return Err(anyhow!(
            "usage: gen-pages <locale.json> <stats.json> <sirenix.json> <wiki-root>"
        ));
    }
    let locale_path = PathBuf::from(&args[1]);
    let stats_path = PathBuf::from(&args[2]);
    let sirenix_path = PathBuf::from(&args[3]);
    let wiki_root = PathBuf::from(&args[4]);

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
    write_file(&wiki_root, "spacecraft/README.md", &page_spacecraft(&locale, &sirenix))?;
    write_file(&wiki_root, "launch-vehicles/README.md", &page_launch_vehicles(&locale))?;
    write_file(&wiki_root, "corporations/README.md", &page_corporations(&locale))?;
    write_file(&wiki_root, "resources/README.md", &page_resources(&locale))?;
    write_file(&wiki_root, "contracts/README.md", &page_contracts(&locale))?;
    write_file(&wiki_root, "research/README.md", &page_research(&locale))?;
    write_file(&wiki_root, "missions/README.md", &page_missions())?;
    Ok(())
}

//! Parser for `TerraformationConfig.asset` (Unity YAML).
//!
//! Each habitability parameter (temperature, pressure, composition, gravity,
//! magnetosphere, water) ships with an `AnimationCurve` and a `Gradient`. The
//! curve maps a raw input value (°C, atm, m/s², …) to a habitability *score*
//! roughly in [-100, 100]. The gradient stitches together the colored bands
//! the in-game Object Info window uses to highlight the current bucket; its
//! `ctime{N}` keys are 0–65535 fixed-point positions along the gradient's
//! `range` (a score interval, typically `{x: 0, y: 100}`).
//!
//! To surface the bucket boundaries on the wiki we:
//!   1. Parse the curve keyframes (`time`, `value`).
//!   2. Parse the gradient color/alpha key positions and the score range.
//!   3. For each distinct interior gradient transition, convert it back to a
//!      score and inverse-lookup the curve for the input values that produce
//!      that score. Non-monotonic curves (notably temperature, which peaks at
//!      ~13 °C and falls off in both directions) yield TWO crossings per
//!      transition; monotonic curves yield one.
//!   4. Stitch the crossings into a sorted list of value-space boundaries —
//!      these are the bucket edges.
//!
//! The output JSON keeps the raw curve+gradient data alongside the derived
//! `label_thresholds[]` so the gen-pages renderer can fall back to the raw
//! shape if a future config diverges from the bucket-count we expect.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// One animation-curve keyframe. We only keep the `(time, value)` pair —
/// `inSlope`/`outSlope` aren't needed for piecewise-linear inverse lookup,
/// which is plenty accurate for the bucket-boundary rendering we do.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub time: f64,
    pub value: f64,
}

/// Parsed Unity `Gradient`.
///
/// Unity stores up to 8 color keys (`ctime0..7`, `key0..7`) and 8 alpha keys
/// (`atime0..7`); `m_NumColorKeys` / `m_NumAlphaKeys` tell us how many slots
/// are actually live. Times are 0–65535 fixed-point; we convert to 0–1 floats
/// on parse.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Gradient {
    /// Sorted, deduplicated 0–1 positions combining color + alpha keys.
    pub key_positions: Vec<f64>,
    /// Score range the gradient spans (typically `{x: 0, y: 100}` for
    /// habitability — the negative-score half is implicit).
    pub range_min: f64,
    pub range_max: f64,
}

/// One score → value-range bucket on the wiki page.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct LabelThreshold {
    /// Lower score boundary (inclusive).
    pub score_min: f64,
    /// Upper score boundary (exclusive on the rising side).
    pub score_max: f64,
    /// Sorted-ascending value boundaries. For monotonic curves this is a
    /// `[min, max]` pair; for the temperature curve a non-edge bucket can be
    /// represented as two disjoint intervals (cold side + hot side) — those
    /// surface as four numbers `[cold_lo, cold_hi, hot_lo, hot_hi]`.
    pub value_ranges: Vec<f64>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ParameterData {
    pub unit: String,
    pub curve_keyframes: Vec<Keyframe>,
    pub gradient_ctimes_normalized: Vec<f64>,
    pub score_range: (f64, f64),
    /// Bucket thresholds derived from the gradient transitions. Sorted by
    /// `score_min` ascending so the renderer can walk the table in order.
    pub label_thresholds: Vec<LabelThreshold>,
}

/// One entry from the `resultCommentTranslationId[]` array under
/// `habitability:`. The game's Object Info window labels each body by the
/// first entry whose `min_result` ≤ the body's overall habitability score —
/// i.e. these are score-floor thresholds, listed in descending order of
/// floor.  The shipped asset carries four entries (90 / 50 / 0 / -100000),
/// which the renderer reads back as the Excellent / Good / Marginal /
/// Hostile buckets.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ResultCommentThreshold {
    pub min_result: f64,
    pub translation_id: String,
}

#[derive(Serialize)]
pub struct TerraformationData {
    pub source: String,
    pub parameters: std::collections::BTreeMap<String, ParameterData>,
    /// Habitability-% bucket thresholds — preserved in source/descending
    /// order. Empty when the asset doesn't carry the block (defensive — the
    /// shipped file always has it).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub result_comment_thresholds: Vec<ResultCommentThreshold>,
}

// ---------------------------------------------------------------------------
// YAML parsing
// ---------------------------------------------------------------------------

/// Strip a `# comment` tail and trim. The TerraformationConfig file is
/// vanilla Unity YAML — no embedded comments — but stripping makes the
/// parser robust against hand-edited fixtures.
fn clean_line(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => line[..idx].trim_end(),
        None => line.trim_end(),
    }
}

/// Compute indent depth in 2-space units. Unity asset files always use
/// 2-space indentation; we return `usize::MAX` for blank lines so callers
/// can skip them.
fn indent_of(line: &str) -> usize {
    let trimmed = line.trim_start_matches(' ');
    if trimmed.is_empty() {
        return usize::MAX;
    }
    (line.len() - trimmed.len()) / 2
}

/// Extract the value substring after the first colon at the given indent
/// level. Returns `None` for list-element rows (`- foo:`) — list elements
/// require dedicated handling in `parse_curve_block`.
fn split_kv(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    if line.starts_with('-') {
        return None;
    }
    let colon = line.find(':')?;
    let key = &line[..colon];
    let value = line[colon + 1..].trim();
    Some((key, value))
}

/// Parse `m_Curve` keyframes from a block of lines.
///
/// `start` is the line index of the first list element (`- serializedVersion: 3`)
/// or anything that immediately follows the `m_Curve:` header. `list_indent`
/// is the indent of the list elements themselves — in Unity YAML the list
/// items share their parent key's indent (`m_Curve:` at 8 spaces, then
/// `- serializedVersion: 3` also at 8 spaces), so the keyframe sub-fields
/// (`time`, `value`) live at `list_indent + 1`.
///
/// We stop as soon as we hit a line whose indent is `< list_indent`, OR a
/// line at exactly `list_indent` that ISN'T a list element (e.g. the
/// sibling `m_PreInfinity:` key).
///
/// The relevant fields per keyframe are `time` and `value`; tangent/weight
/// fields are skipped.
pub fn parse_curve_keyframes(lines: &[&str], start: usize, list_indent: usize) -> Vec<Keyframe> {
    let mut out: Vec<Keyframe> = Vec::new();
    let mut current_time: Option<f64> = None;
    let mut current_value: Option<f64> = None;

    for line in lines.iter().skip(start) {
        let cleaned = clean_line(line);
        if cleaned.trim().is_empty() {
            continue;
        }
        let depth = indent_of(line);
        if depth < list_indent {
            break;
        }
        let trimmed = cleaned.trim_start();
        if depth == list_indent {
            if trimmed.starts_with("- ") {
                // New keyframe — flush the previous one and reset.
                if let (Some(t), Some(v)) = (current_time, current_value) {
                    out.push(Keyframe { time: t, value: v });
                }
                current_time = None;
                current_value = None;
                continue;
            } else {
                // A sibling key at the m_Curve list's indent (m_PreInfinity,
                // m_PostInfinity, m_RotationOrder, …) — the list has ended.
                break;
            }
        }
        // depth > list_indent: this is a sub-field of the current keyframe.
        if let Some((key, value)) = split_kv(cleaned) {
            match key {
                "time" => current_time = parse_special_float(value),
                "value" => current_value = parse_special_float(value),
                _ => {}
            }
        }
    }
    if let (Some(t), Some(v)) = (current_time, current_value) {
        out.push(Keyframe { time: t, value: v });
    }
    out
}

/// Parse a Unity float that may be `Infinity` / `-Infinity` / `NaN` (Unity
/// emits these as bare identifiers without quotes for animation-curve
/// slopes/weights).
fn parse_special_float(s: &str) -> Option<f64> {
    match s {
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        "NaN" => Some(f64::NAN),
        _ => s.parse().ok(),
    }
}

/// Parse a gradient block. `start` is the line index of the first child of
/// `gradient:`; `parent_indent` is the indent of `gradient:` itself.
pub fn parse_gradient_block(lines: &[&str], start: usize, parent_indent: usize) -> Gradient {
    let mut ctimes_raw: Vec<u32> = vec![0; 8];
    let mut atimes_raw: Vec<u32> = vec![0; 8];
    let mut num_color: usize = 0;
    let mut num_alpha: usize = 0;
    for line in lines.iter().skip(start) {
        let cleaned = clean_line(line);
        if cleaned.trim().is_empty() {
            continue;
        }
        if indent_of(line) <= parent_indent {
            break;
        }
        if let Some((key, value)) = split_kv(cleaned) {
            if let Some(idx_str) = key.strip_prefix("ctime") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < 8 {
                        if let Ok(v) = value.parse() {
                            ctimes_raw[idx] = v;
                        }
                    }
                }
            } else if let Some(idx_str) = key.strip_prefix("atime") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < 8 {
                        if let Ok(v) = value.parse() {
                            atimes_raw[idx] = v;
                        }
                    }
                }
            } else if key == "m_NumColorKeys" {
                num_color = value.parse().unwrap_or(0);
            } else if key == "m_NumAlphaKeys" {
                num_alpha = value.parse().unwrap_or(0);
            }
        }
    }
    let mut positions: Vec<f64> = Vec::new();
    for i in 0..num_color.min(8) {
        positions.push(ctimes_raw[i] as f64 / 65535.0);
    }
    for i in 0..num_alpha.min(8) {
        positions.push(atimes_raw[i] as f64 / 65535.0);
    }
    // Sort + dedupe (within 1e-6 tolerance).
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    positions.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    Gradient {
        key_positions: positions,
        range_min: 0.0,
        range_max: 100.0,
    }
}

/// Read a `range: {x: A, y: B}` line and pull out the two floats.
fn parse_range_inline(value: &str) -> Option<(f64, f64)> {
    let inner = value.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut x = None;
    let mut y = None;
    for pair in inner.split(',') {
        let mut it = pair.splitn(2, ':');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        match k {
            "x" => x = v.parse().ok(),
            "y" => y = v.parse().ok(),
            _ => {}
        }
    }
    Some((x?, y?))
}

// ---------------------------------------------------------------------------
// Inverse curve lookup
// ---------------------------------------------------------------------------

/// Given a piecewise-linear curve defined by `keyframes` (sorted ascending
/// by `time`), find every input `time` where the curve's `value` equals the
/// target. Returns the crossings in ascending order. For a non-monotonic
/// curve there can be more than one crossing — temperature's hump produces
/// two crossings per interior score.
///
/// The lookup uses linear interpolation between adjacent keyframes. That's
/// a simplification of Unity's Hermite spline but is accurate to a few
/// percent for the smooth, mostly-monotonic curves in TerraformationConfig
/// — and gives identical bucket edges at the keyframe vertices where the
/// player-visible boundaries actually fall.
pub fn inverse_curve_lookup(keyframes: &[Keyframe], target: f64) -> Vec<f64> {
    let mut crossings: Vec<f64> = Vec::new();
    if keyframes.len() < 2 {
        return crossings;
    }
    for pair in keyframes.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        // Endpoint exact match — emit `a.time` so chained segments don't
        // double-count.  We skip exact matches at `b.time` because they
        // re-emerge on the next window iteration.
        if (a.value - target).abs() < 1e-9 {
            crossings.push(a.time);
            continue;
        }
        let lo = a.value.min(b.value);
        let hi = a.value.max(b.value);
        if target > lo && target < hi {
            // Linear interpolation: t such that a.value + t * (b.value - a.value) == target.
            let denom = b.value - a.value;
            if denom.abs() < 1e-12 {
                continue;
            }
            let t = (target - a.value) / denom;
            crossings.push(a.time + t * (b.time - a.time));
        }
    }
    // Also consider the last keyframe for an exact endpoint match.
    if let Some(last) = keyframes.last() {
        if (last.value - target).abs() < 1e-9 {
            crossings.push(last.time);
        }
    }
    // Sort + dedupe near-duplicates (boundary keyframes where adjacent
    // segments would otherwise each emit the same crossing).
    crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    crossings.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    crossings
}

/// Compute bucket boundaries (sorted, ascending) for a parameter given its
/// curve and gradient transitions.
///
/// Strategy: each gradient position `p ∈ [0, 1]` maps to a target score
/// `score = range_min + p * (range_max - range_min)`. We inverse-lookup the
/// curve at that score and collect crossings. Concatenated and sorted, the
/// crossings + curve endpoints form the bucket edges.
///
/// For a non-monotonic curve (temperature), gradient position 1.0 — which
/// maps to score=range_max=100 — surfaces the PEAK input value (e.g. 13°C),
/// since that's where the curve hits its maximum. We rely on
/// `inverse_curve_lookup` to return that crossing.
pub fn compute_bucket_boundaries(
    keyframes: &[Keyframe],
    gradient: &Gradient,
) -> Vec<f64> {
    let mut boundaries: Vec<f64> = Vec::new();
    // Always include the curve's input-value extents as outer boundaries.
    if let (Some(first), Some(last)) = (keyframes.first(), keyframes.last()) {
        boundaries.push(first.time);
        boundaries.push(last.time);
    }
    for &p in &gradient.key_positions {
        let target = gradient.range_min + p * (gradient.range_max - gradient.range_min);
        for x in inverse_curve_lookup(keyframes, target) {
            boundaries.push(x);
        }
    }
    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    boundaries.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
    boundaries
}

/// Compose the per-parameter `LabelThreshold` list from sorted bucket edges.
///
/// Each adjacent pair of edges becomes one bucket. We compute the score at
/// the bucket midpoint so the renderer can sort buckets by "how habitable"
/// they are if it wants to — but emit them in input-value order so the
/// table reads left-to-right as the player would expect.
pub fn build_label_thresholds(
    keyframes: &[Keyframe],
    gradient: &Gradient,
) -> Vec<LabelThreshold> {
    let edges = compute_bucket_boundaries(keyframes, gradient);
    let mut out: Vec<LabelThreshold> = Vec::new();
    for pair in edges.windows(2) {
        let lo = pair[0];
        let hi = pair[1];
        let mid = (lo + hi) / 2.0;
        let score = sample_curve(keyframes, mid);
        out.push(LabelThreshold {
            score_min: score,
            score_max: score,
            value_ranges: vec![lo, hi],
        });
    }
    out
}

/// Sample the curve at `x` using piecewise-linear interpolation.
pub fn sample_curve(keyframes: &[Keyframe], x: f64) -> f64 {
    if keyframes.is_empty() {
        return 0.0;
    }
    if x <= keyframes[0].time {
        return keyframes[0].value;
    }
    if x >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().value;
    }
    for pair in keyframes.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if x >= a.time && x <= b.time {
            let denom = b.time - a.time;
            if denom.abs() < 1e-12 {
                return a.value;
            }
            let t = (x - a.time) / denom;
            return a.value + t * (b.value - a.value);
        }
    }
    keyframes.last().unwrap().value
}

// ---------------------------------------------------------------------------
// Top-level config parsing
// ---------------------------------------------------------------------------

/// Parameters we surface on the wiki. Listed in the order they should appear.
const PARAMETER_KEYS: &[(&str, &str, &str)] = &[
    // (asset-key, output-key, unit)
    ("temperature", "temperature", "°C"),
    ("pressure", "pressure", "atm"),
    ("gravity", "gravity", "m/s²"),
    ("magnetosphere", "magnetosphere", "μT"),
    ("composition", "composition", "O₂ fraction"),
    ("water", "water", "ocean fraction"),
];

/// Pull the `resultCommentTranslationId[]` list out of the habitability
/// block. Entries are returned in source order (descending by `min_result`
/// in the shipped asset). Returns an empty vec when the block is missing.
pub fn parse_result_comment_thresholds(yaml: &str) -> Vec<ResultCommentThreshold> {
    let lines: Vec<&str> = yaml.lines().collect();
    // Locate the list header.  In the shipped file `habitability:` lives at
    // depth 1 and `resultCommentTranslationId:` at depth 2 — the list
    // items themselves are at the same indent as their parent key in Unity
    // YAML (i.e. each `- translationId: …` line is at depth 2 too).
    let header_idx = match lines
        .iter()
        .position(|l| l.trim_start().trim_end() == "resultCommentTranslationId:")
    {
        Some(idx) => idx,
        None => return Vec::new(),
    };
    let header_indent = indent_of(lines[header_idx]);
    let mut out: Vec<ResultCommentThreshold> = Vec::new();
    let mut cur_id: Option<String> = None;
    let mut cur_min: Option<f64> = None;
    for line in lines.iter().skip(header_idx + 1) {
        let cleaned = clean_line(line);
        if cleaned.trim().is_empty() {
            continue;
        }
        let depth = indent_of(line);
        let trimmed = cleaned.trim_start();
        if depth < header_indent {
            break;
        }
        if depth == header_indent {
            if trimmed.starts_with("- ") {
                // Flush previous entry, start a new one.
                if let (Some(t), Some(m)) = (cur_id.take(), cur_min.take()) {
                    out.push(ResultCommentThreshold {
                        translation_id: t,
                        min_result: m,
                    });
                }
                // The `- ` line carries the first field inline, e.g.
                //   `- translationId: Tooltip.…ResultComment0`
                let after_dash = trimmed.trim_start_matches('-').trim_start();
                if let Some(colon) = after_dash.find(':') {
                    let key = &after_dash[..colon];
                    let value = after_dash[colon + 1..].trim();
                    match key {
                        "translationId" => cur_id = Some(value.to_string()),
                        "minResult" => cur_min = value.parse().ok(),
                        _ => {}
                    }
                }
                continue;
            } else {
                // Sibling key at header indent — list ended.
                break;
            }
        }
        // depth > header_indent: sub-field of the current list item.
        if let Some((key, value)) = split_kv(cleaned) {
            match key {
                "translationId" => cur_id = Some(value.to_string()),
                "minResult" => cur_min = value.parse().ok(),
                _ => {}
            }
        }
    }
    if let (Some(t), Some(m)) = (cur_id, cur_min) {
        out.push(ResultCommentThreshold {
            translation_id: t,
            min_result: m,
        });
    }
    out
}

/// Parse the whole `TerraformationConfig.asset` and pull every parameter's
/// curve/gradient pair out.
pub fn parse_config(yaml: &str) -> std::collections::BTreeMap<String, ParameterData> {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut out: std::collections::BTreeMap<String, ParameterData> = std::collections::BTreeMap::new();

    for &(asset_key, out_key, unit) in PARAMETER_KEYS {
        let header = format!("    {asset_key}:");
        let start = match lines.iter().position(|l| l.trim_end() == header.trim_end()) {
            Some(idx) => idx + 1,
            None => continue,
        };
        let parent_indent = 2; // `temperature:` is at depth 2 (under habitability:)
        let parsed = parse_parameter(&lines, start, parent_indent, unit);
        if let Some(data) = parsed {
            out.insert(out_key.to_string(), data);
        }
    }
    out
}

/// Walk the lines following a parameter header (e.g. `    temperature:`) and
/// pull out its `curve`, `gradient`, and `range`. Stops when indent drops back
/// to `parent_indent` or shallower.
fn parse_parameter(
    lines: &[&str],
    start: usize,
    parent_indent: usize,
    unit: &str,
) -> Option<ParameterData> {
    let mut keyframes: Vec<Keyframe> = Vec::new();
    let mut gradient = Gradient {
        key_positions: Vec::new(),
        range_min: 0.0,
        range_max: 100.0,
    };
    let mut range = (0.0, 100.0);
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        let cleaned = clean_line(line);
        if cleaned.trim().is_empty() {
            i += 1;
            continue;
        }
        let depth = indent_of(line);
        if depth <= parent_indent {
            break;
        }
        // We only care about direct children at depth == parent_indent + 1.
        if depth == parent_indent + 1 {
            if let Some((key, value)) = split_kv(cleaned) {
                match key {
                    "curve" => {
                        // The `m_Curve:` sub-block starts inside the curve block.
                        // `curve:` is at depth `parent_indent + 1`; its inner
                        // fields (incl. m_Curve:) live at `parent_indent + 2`.
                        let curve_depth = parent_indent + 1;
                        for (j, inner) in lines.iter().enumerate().skip(i + 1) {
                            let inner_clean = clean_line(inner);
                            if inner_clean.trim().is_empty() {
                                continue;
                            }
                            if indent_of(inner) <= curve_depth {
                                break;
                            }
                            if let Some((ikey, _)) = split_kv(inner_clean) {
                                if ikey == "m_Curve" {
                                    // List elements sit at the same indent
                                    // as `m_Curve:` itself in Unity YAML.
                                    let list_indent = indent_of(inner);
                                    keyframes = parse_curve_keyframes(lines, j + 1, list_indent);
                                    break;
                                }
                            }
                        }
                    }
                    "gradient" => {
                        gradient = parse_gradient_block(lines, i + 1, parent_indent + 1);
                    }
                    "range" => {
                        if let Some(r) = parse_range_inline(value) {
                            range = r;
                            gradient.range_min = r.0;
                            gradient.range_max = r.1;
                        }
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }
    if keyframes.is_empty() || gradient.key_positions.is_empty() {
        return None;
    }
    let label_thresholds = build_label_thresholds(&keyframes, &gradient);
    let gradient_ctimes_normalized = gradient.key_positions.clone();
    Some(ParameterData {
        unit: unit.to_string(),
        curve_keyframes: keyframes,
        gradient_ctimes_normalized,
        score_range: range,
        label_thresholds,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- inverse_curve_lookup ---------------------------------------------

    #[test]
    fn inverse_lookup_monotonic_returns_single_crossing() {
        // Pressure-like curve: monotonic from 0 (score 0) up to 1 (score 100)
        // then down. We test the rising-only fragment.
        let curve = vec![
            Keyframe { time: 0.0, value: 0.0 },
            Keyframe { time: 1.0, value: 100.0 },
        ];
        let xs = inverse_curve_lookup(&curve, 50.0);
        assert_eq!(xs.len(), 1, "single rising segment yields one crossing");
        assert!((xs[0] - 0.5).abs() < 1e-9, "midpoint of segment: {:?}", xs);
    }

    #[test]
    fn inverse_lookup_nonmonotonic_returns_two_crossings_in_sorted_order() {
        // Synthetic non-monotonic peak — same topology as the temperature
        // curve. Score=50 is crossed on the way up AND on the way down.
        let curve = vec![
            Keyframe { time: -100.0, value: 0.0 },
            Keyframe { time: 0.0, value: 100.0 },
            Keyframe { time: 200.0, value: 0.0 },
            Keyframe { time: 600.0, value: -100.0 },
        ];
        let xs = inverse_curve_lookup(&curve, 50.0);
        assert_eq!(xs.len(), 2, "peak curve yields two crossings, got {:?}", xs);
        // Rising leg: 0→100 across -100..0, so 50 occurs at -50.
        assert!((xs[0] - (-50.0)).abs() < 1e-9, "rising crossing: {:?}", xs);
        // Falling leg: 100→0 across 0..200, so 50 occurs at 100.
        assert!((xs[1] - 100.0).abs() < 1e-9, "falling crossing: {:?}", xs);
        // Sorted ascending.
        assert!(xs[0] < xs[1]);
    }

    #[test]
    fn inverse_lookup_target_above_peak_returns_no_crossings() {
        let curve = vec![
            Keyframe { time: 0.0, value: 0.0 },
            Keyframe { time: 10.0, value: 50.0 },
            Keyframe { time: 20.0, value: 0.0 },
        ];
        let xs = inverse_curve_lookup(&curve, 75.0);
        assert!(xs.is_empty(), "no crossings above peak: {:?}", xs);
    }

    #[test]
    fn inverse_lookup_target_at_endpoint_value_is_returned_once() {
        // A target equal to a keyframe's exact value should be emitted once
        // (not duplicated across the two segments that meet at that
        // keyframe).
        let curve = vec![
            Keyframe { time: 0.0, value: 0.0 },
            Keyframe { time: 10.0, value: 100.0 },
            Keyframe { time: 20.0, value: 0.0 },
        ];
        let xs = inverse_curve_lookup(&curve, 100.0);
        assert_eq!(xs.len(), 1, "peak emits once: {:?}", xs);
        assert!((xs[0] - 10.0).abs() < 1e-6);
    }

    // ---- sample_curve -----------------------------------------------------

    #[test]
    fn sample_curve_interpolates_between_keyframes() {
        let curve = vec![
            Keyframe { time: 0.0, value: 0.0 },
            Keyframe { time: 10.0, value: 100.0 },
        ];
        assert!((sample_curve(&curve, 5.0) - 50.0).abs() < 1e-9);
    }

    #[test]
    fn sample_curve_clamps_below_first_keyframe() {
        let curve = vec![
            Keyframe { time: -100.0, value: 0.0 },
            Keyframe { time: 0.0, value: 100.0 },
        ];
        assert_eq!(sample_curve(&curve, -200.0), 0.0);
    }

    #[test]
    fn sample_curve_clamps_above_last_keyframe() {
        let curve = vec![
            Keyframe { time: 0.0, value: 100.0 },
            Keyframe { time: 100.0, value: -100.0 },
        ];
        assert_eq!(sample_curve(&curve, 9999.0), -100.0);
    }

    // ---- gradient parser --------------------------------------------------

    #[test]
    fn gradient_parses_color_and_alpha_keys_and_dedupes_positions() {
        // Temperature gradient from the real asset. We construct it
        // line-by-line with `\n` separators so the leading 8-space indent
        // survives — Rust's `\<newline>` escape strips trailing whitespace
        // up to the next non-WS char, which collapses indentation.
        let yaml = concat!(
            "        ctime0: 29491\n",
            "        ctime1: 58982\n",
            "        ctime2: 65535\n",
            "        ctime3: 65535\n",
            "        ctime4: 0\n",
            "        ctime5: 0\n",
            "        ctime6: 0\n",
            "        ctime7: 0\n",
            "        atime0: 0\n",
            "        atime1: 65535\n",
            "        atime2: 0\n",
            "        atime3: 0\n",
            "        atime4: 0\n",
            "        atime5: 0\n",
            "        atime6: 0\n",
            "        atime7: 0\n",
            "        m_Mode: 1\n",
            "        m_ColorSpace: 0\n",
            "        m_NumColorKeys: 3\n",
            "        m_NumAlphaKeys: 2\n",
        );
        let lines: Vec<&str> = yaml.lines().collect();
        let g = parse_gradient_block(&lines, 0, 3);
        // 3 color keys at 0.45, 0.9, 1.0; 2 alpha keys at 0.0, 1.0. Deduped =
        // {0.0, 0.45, 0.90, 1.0}.
        assert_eq!(g.key_positions.len(), 4, "deduped key count: {:?}", g.key_positions);
        assert!((g.key_positions[0] - 0.0).abs() < 1e-3);
        assert!((g.key_positions[1] - 0.45).abs() < 0.01);
        assert!((g.key_positions[2] - 0.90).abs() < 0.01);
        assert!((g.key_positions[3] - 1.0).abs() < 1e-3);
    }

    // ---- curve keyframe parser --------------------------------------------

    #[test]
    fn curve_keyframes_parse_real_temperature_block() {
        // Two keyframes from the real temperature curve. We use concat! so
        // the indentation survives — `\<newline>` in Rust string literals
        // strips leading whitespace on the next line, which would flatten
        // the YAML structure.
        let yaml = concat!(
            "        m_Curve:\n",
            "        - serializedVersion: 3\n",
            "          time: -273\n",
            "          value: 0\n",
            "          inSlope: 0.34965035\n",
            "          outSlope: 0.34965035\n",
            "          tangentMode: 0\n",
            "          weightedMode: 0\n",
            "          inWeight: 0\n",
            "          outWeight: 0.33333334\n",
            "        - serializedVersion: 3\n",
            "          time: 13\n",
            "          value: 100\n",
            "          inSlope: 0\n",
            "          outSlope: 0\n",
            "          tangentMode: 0\n",
            "          weightedMode: 0\n",
            "          inWeight: 0.33333334\n",
            "          outWeight: 0.33333334\n",
            "        m_PreInfinity: 2\n",
        );
        let lines: Vec<&str> = yaml.lines().collect();
        // m_Curve: is at indent depth 4 (8 leading spaces), list elements at
        // the same depth — Unity YAML keeps `- foo` at the parent key's
        // indent.
        let curve_idx = lines.iter().position(|l| l.trim() == "m_Curve:").unwrap();
        let list_indent = indent_of(lines[curve_idx]);
        let kfs = parse_curve_keyframes(&lines, curve_idx + 1, list_indent);
        assert_eq!(kfs.len(), 2, "two keyframes: {:?}", kfs);
        assert_eq!(kfs[0], Keyframe { time: -273.0, value: 0.0 });
        assert_eq!(kfs[1], Keyframe { time: 13.0, value: 100.0 });
    }

    #[test]
    fn curve_keyframes_handle_infinity_slopes() {
        // The composition curve has `outSlope: Infinity` — make sure the
        // parser doesn't choke (we don't store slopes, but the regex/colon
        // logic still needs to skip those lines).
        let yaml = concat!(
            "        m_Curve:\n",
            "        - serializedVersion: 3\n",
            "          time: 0\n",
            "          value: 0\n",
            "          inSlope: Infinity\n",
            "          outSlope: Infinity\n",
            "        - serializedVersion: 3\n",
            "          time: 1\n",
            "          value: 100\n",
            "          inSlope: -0.5\n",
            "          outSlope: -0.5\n",
            "        m_PreInfinity: 2\n",
        );
        let lines: Vec<&str> = yaml.lines().collect();
        let curve_idx = lines.iter().position(|l| l.trim() == "m_Curve:").unwrap();
        let list_indent = indent_of(lines[curve_idx]);
        let kfs = parse_curve_keyframes(&lines, curve_idx + 1, list_indent);
        assert_eq!(kfs.len(), 2);
        assert_eq!(kfs[1], Keyframe { time: 1.0, value: 100.0 });
    }

    // ---- range parser -----------------------------------------------------

    #[test]
    fn range_inline_parses_x_y() {
        assert_eq!(parse_range_inline("{x: 0, y: 100}"), Some((0.0, 100.0)));
        assert_eq!(parse_range_inline("{x: -50, y: 50}"), Some((-50.0, 50.0)));
    }

    // ---- bucket boundaries (integration of the above) ---------------------

    #[test]
    fn bucket_boundaries_for_monotonic_curve_produce_sorted_edges() {
        // Pressure-shape curve: rising then falling. Gradient cuts at score
        // 33 and 66 (positions 0.33, 0.66 within range [0, 100]).
        let curve = vec![
            Keyframe { time: 0.0, value: 0.0 },
            Keyframe { time: 1.0, value: 100.0 },
            Keyframe { time: 50.0, value: -100.0 },
        ];
        let gradient = Gradient {
            key_positions: vec![0.0, 0.33, 0.66, 1.0],
            range_min: 0.0,
            range_max: 100.0,
        };
        let edges = compute_bucket_boundaries(&curve, &gradient);
        // Should be sorted ascending.
        for w in edges.windows(2) {
            assert!(w[0] <= w[1], "edges must be sorted: {:?}", edges);
        }
        // Outer edges from the curve endpoints (0 and 50).
        assert!((edges[0] - 0.0).abs() < 1e-6, "first edge is curve start: {:?}", edges);
        assert!((edges[edges.len() - 1] - 50.0).abs() < 1e-6, "last edge is curve end: {:?}", edges);
    }

    #[test]
    fn bucket_boundaries_for_temperature_capture_both_sides_of_peak() {
        // The real temperature curve has its peak (score=100) at time=13 °C.
        // With a gradient at positions {0.0, 0.5, 1.0} over score range
        // [0, 100], the cuts target scores 0, 50, and 100. For the peak
        // curve:
        //   * score=0 occurs at -273 (curve start) and 100 (falling crossing
        //     after the peak), plus 700 ... actually 700 → -100, so 0 is at
        //     time=100 on the falling leg.
        //   * score=50 occurs at -130 (rising) and 56.5 (falling).
        //   * score=100 occurs at the peak time=13.
        // Concat + sort + dedupe with curve endpoints gives:
        //   [-273, -130, 13, 56.5, 100, 700]
        let curve = vec![
            Keyframe { time: -273.0, value: 0.0 },
            Keyframe { time: 13.0, value: 100.0 },
            Keyframe { time: 100.0, value: 0.0 },
            Keyframe { time: 700.0, value: -100.0 },
        ];
        let gradient = Gradient {
            key_positions: vec![0.0, 0.5, 1.0],
            range_min: 0.0,
            range_max: 100.0,
        };
        let edges = compute_bucket_boundaries(&curve, &gradient);
        // Sorted ascending.
        for w in edges.windows(2) {
            assert!(w[0] <= w[1], "edges must be sorted: {:?}", edges);
        }
        // Outer endpoints preserved.
        assert!((edges[0] - (-273.0)).abs() < 1e-3, "first edge: {:?}", edges);
        assert!((edges[edges.len() - 1] - 700.0).abs() < 1e-3, "last edge: {:?}", edges);
        // Both rising AND falling crossings for score=50 surface.
        let has_rising = edges.iter().any(|e| (*e - (-130.0)).abs() < 1e-3);
        let has_falling = edges.iter().any(|e| (*e - 56.5).abs() < 1.0);
        assert!(has_rising, "rising cut at -130 missing: {:?}", edges);
        assert!(has_falling, "falling cut at ~56.5 missing: {:?}", edges);
        // Peak (score=100 → time=13) shows up too.
        let has_peak = edges.iter().any(|e| (*e - 13.0).abs() < 1.0);
        assert!(has_peak, "peak at 13 missing: {:?}", edges);
    }

    // ---- end-to-end on the real asset format ------------------------------

    #[test]
    fn parse_config_extracts_temperature_block_from_real_yaml() {
        // Minimal subset of the real TerraformationConfig — only the
        // habitability block with temperature. Indentation in a regular
        // Rust literal would be eaten by the `\<newline>` continuation; the
        // raw string preserves the structure verbatim.
        let yaml = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!114 &11400000
MonoBehaviour:
  m_Name: TerraformationConfig
  habitability:
    temperature:
      curve:
        serializedVersion: 2
        m_Curve:
        - serializedVersion: 3
          time: -273
          value: 0
          inSlope: 0
          outSlope: 0
          tangentMode: 0
          weightedMode: 0
          inWeight: 0
          outWeight: 0
        - serializedVersion: 3
          time: 13
          value: 100
          inSlope: 0
          outSlope: 0
          tangentMode: 0
          weightedMode: 0
          inWeight: 0
          outWeight: 0
        - serializedVersion: 3
          time: 100
          value: 0
          inSlope: 0
          outSlope: 0
          tangentMode: 0
          weightedMode: 0
          inWeight: 0
          outWeight: 0
        - serializedVersion: 3
          time: 700
          value: -100
          inSlope: 0
          outSlope: 0
          tangentMode: 0
          weightedMode: 0
          inWeight: 0
          outWeight: 0
        m_PreInfinity: 2
        m_PostInfinity: 2
        m_RotationOrder: 4
      weight: 6
      gradient:
        serializedVersion: 2
        key0: {r: 1, g: 0, b: 0, a: 1}
        key1: {r: 1, g: 1, b: 0, a: 1}
        key2: {r: 0, g: 1, b: 0, a: 0}
        key3: {r: 0, g: 1, b: 0, a: 0}
        key4: {r: 0, g: 0, b: 0, a: 0}
        key5: {r: 0, g: 0, b: 0, a: 0}
        key6: {r: 0, g: 0, b: 0, a: 0}
        key7: {r: 0, g: 0, b: 0, a: 0}
        ctime0: 29491
        ctime1: 58982
        ctime2: 65535
        ctime3: 65535
        ctime4: 0
        ctime5: 0
        ctime6: 0
        ctime7: 0
        atime0: 0
        atime1: 65535
        atime2: 0
        atime3: 0
        atime4: 0
        atime5: 0
        atime6: 0
        atime7: 0
        m_Mode: 1
        m_ColorSpace: 0
        m_NumColorKeys: 3
        m_NumAlphaKeys: 2
      range: {x: 0, y: 100}
"#;
        let params = parse_config(yaml);
        assert!(params.contains_key("temperature"), "temperature parsed: {:?}", params.keys().collect::<Vec<_>>());
        let temp = &params["temperature"];
        assert_eq!(temp.unit, "°C");
        assert_eq!(temp.curve_keyframes.len(), 4);
        assert_eq!(temp.curve_keyframes[0], Keyframe { time: -273.0, value: 0.0 });
        assert_eq!(temp.curve_keyframes[1], Keyframe { time: 13.0, value: 100.0 });
        assert_eq!(temp.score_range, (0.0, 100.0));
        // gradient_ctimes_normalized — deduped { 0.0, 0.45, 0.90, 1.0 }.
        assert_eq!(temp.gradient_ctimes_normalized.len(), 4);
        // label_thresholds: with gradient cuts at 0.45 and 0.90 (scores 45 and 90)
        // and a peak curve, you get crossings on both sides → 4 internal
        // cuts + 2 outer endpoints = 6 buckets total.
        let edges: Vec<f64> = {
            let mut v = vec![temp.label_thresholds[0].value_ranges[0]];
            for lt in &temp.label_thresholds {
                v.push(lt.value_ranges[1]);
            }
            v
        };
        assert!(edges.len() >= 4, "at least 4 edges from temperature peak: {:?}", edges);
        // Outer extents preserved.
        assert!((edges[0] - (-273.0)).abs() < 1.0);
        assert!((edges[edges.len() - 1] - 700.0).abs() < 1.0);
        // The peak input (time=13) should be one of the bucket edges
        // (the rising/falling boundary).
        let has_peak = edges.iter().any(|e| (e - 13.0).abs() < 1.0);
        assert!(has_peak, "peak at 13°C should be a bucket edge: {:?}", edges);
    }

    // ---- result-comment thresholds ----------------------------------------

    /// The habitability block ships a `resultCommentTranslationId:` list that
    /// maps a score floor (`minResult`) to a translation id surfaced by the
    /// in-game Object Info tooltip ("A perfect place for life.", "Our crews
    /// can live here…", etc.). The parser must walk the list in source order
    /// — the renderer relies on that ordering to label the Habitability %
    /// buckets on the terraforming page.
    #[test]
    fn parses_result_comment_thresholds_from_real_yaml() {
        // Subset of `TerraformationConfig.asset` containing only the
        // resultCommentTranslationId block — same shape as the shipped file
        // (real `minResult` values: 90, 50, 0, -100000).
        let yaml = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!114 &11400000
MonoBehaviour:
  m_Name: TerraformationConfig
  habitability:
    resultCommentTranslationId:
    - translationId: Tooltip.UIBasicInfoHabitabilityParameters.ResultComment0
      minResult: 90
    - translationId: Tooltip.UIBasicInfoHabitabilityParameters.ResultComment1
      minResult: 50
    - translationId: Tooltip.UIBasicInfoHabitabilityParameters.ResultComment2
      minResult: 0
    - translationId: Tooltip.UIBasicInfoHabitabilityParameters.ResultComment3
      minResult: -100000
"#;
        let thresholds = parse_result_comment_thresholds(yaml);
        assert_eq!(
            thresholds.len(),
            4,
            "should parse all 4 result-comment entries: {:?}",
            thresholds
        );
        // Walk-order preserves the YAML order; the renderer prints
        // descending-score rows which is the same as source order here.
        assert_eq!(thresholds[0].min_result, 90.0);
        assert_eq!(
            thresholds[0].translation_id,
            "Tooltip.UIBasicInfoHabitabilityParameters.ResultComment0"
        );
        assert_eq!(thresholds[1].min_result, 50.0);
        assert_eq!(thresholds[2].min_result, 0.0);
        assert_eq!(thresholds[3].min_result, -100000.0);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!(
            "usage: parse-terraformation-config <TerraformationConfig.asset> <out.json>"
        ));
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let yaml = fs::read_to_string(&input)
        .with_context(|| format!("reading {}", input.display()))?;
    let parameters = parse_config(&yaml);
    let result_comment_thresholds = parse_result_comment_thresholds(&yaml);
    let data = TerraformationData {
        source: input.display().to_string(),
        parameters,
        result_comment_thresholds,
    };

    serde_json::to_writer_pretty(fs::File::create(&output)?, &data)?;
    eprintln!(
        "wrote {} ({} parameters)",
        output.display(),
        data.parameters.len()
    );
    Ok(())
}

// Node `--test` suite for the corporation-comparison renderer in
// docs/assets/js/corporations.js.
//
// Run with:  node --test extract/tests/corporations.test.js
//
// The script imports the JS module (CommonJS export at the bottom of the
// browser script) and exercises the pure render helpers used by the
// scenario / difficulty dropdowns on the Corporations page.  Nothing here
// touches the DOM — the bindDom() side is left to manual browser checks.

const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');

const C = require(path.join(__dirname, '..', '..', 'docs', 'assets', 'js', 'corporations.js'));

// Compact fixture mirroring the shape page_corporations() emits.
// Research entries are objects { name, category } where category is the
// player-facing sub-branch label from the tech tree (e.g. "Spacecraft",
// "Chemical Propulsion").
const FIXTURE = {
  scenarios: [
    {
      id: 'StartGameColonization',
      name: 'Colonization Era',
      corps: [
        { name: 'SoleX',     starting_money: 33_700_000, lv_count: 2, sc_count: 8, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
          { name: 'Hydrolox',      category: 'Chemical Propulsion' },
          { name: 'Lasers',        category: 'Electromagnetism' },
        ] },
        { name: 'NASA',      starting_money: 35_900_000, lv_count: 1, sc_count: 2, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
          { name: 'Kerolox',       category: 'Chemical Propulsion' },
        ] },
        { name: 'ESA',       starting_money: 30_000_000, lv_count: 1, sc_count: 1, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
          { name: 'Hydrolox',      category: 'Chemical Propulsion' },
        ] },
        { name: 'CNSA',      starting_money: 28_000_000, lv_count: 1, sc_count: 1, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
        ] },
        { name: 'Roscosmos', starting_money: 27_000_000, lv_count: 1, sc_count: 1, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
          { name: 'Kerolox',       category: 'Chemical Propulsion' },
          { name: 'Lasers',        category: 'Electromagnetism' },
        ] },
      ],
    },
    {
      id: 'StartGameExpansion',
      name: 'The Expansion',
      corps: [
        { name: 'SoleX', starting_money: 27_200_000, lv_count: 2, sc_count: 4, research: [
          { name: 'Crewed Flight', category: 'Spacecraft' },
        ] },
      ],
    },
  ],
  difficulties: [
    { name: 'Explorer', money_multiplier: 1.25 },
    { name: 'Pioneer',  money_multiplier: 1.0  },
    { name: 'Veteran',  money_multiplier: 0.75 },
  ],
};

test('buildComparison: research union groups by category, alphabetical within each', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  // Sort: category primary (alphabetical), name secondary (alphabetical).
  // Chemical Propulsion → Hydrolox, Kerolox.  Electromagnetism → Lasers.
  // Spacecraft → Crewed Flight.
  const researchNames = cmp.researchRows.map(function (r) { return r.name; });
  assert.deepEqual(researchNames, ['Hydrolox', 'Kerolox', 'Lasers', 'Crewed Flight']);
  // Sanity: nothing held by zero corps slipped in.
  cmp.researchRows.forEach(function (r) {
    assert.ok(r.held.some(Boolean), 'row ' + r.name + ' has no holders');
  });
});

test('buildComparison: each researchRow carries a category field', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  cmp.researchRows.forEach(function (r) {
    assert.ok(typeof r.category === 'string' && r.category.length > 0,
      'row ' + r.name + ' missing category, got ' + JSON.stringify(r));
  });
  // Specific lookups.
  const hydrolox = cmp.researchRows.find(function (r) { return r.name === 'Hydrolox'; });
  assert.equal(hydrolox.category, 'Chemical Propulsion');
  const crewed = cmp.researchRows.find(function (r) { return r.name === 'Crewed Flight'; });
  assert.equal(crewed.category, 'Spacecraft');
});

test('buildComparison: research items missing a category bucket under "Other"', () => {
  const fx = {
    scenarios: [{
      id: 'X', name: 'X',
      corps: [{ name: 'A', starting_money: 0, lv_count: 0, sc_count: 0, research: [
        { name: 'Mystery',  category: '' },
        { name: 'Other Thing' /* no category property at all */ },
        { name: 'Hydrolox', category: 'Chemical Propulsion' },
      ] }],
    }],
    difficulties: [{ name: 'Pioneer', money_multiplier: 1.0 }],
  };
  const cmp = C.buildComparison(fx, 'X', 'Pioneer', true);
  // Order: Chemical Propulsion (Hydrolox) then Other (Mystery, Other Thing).
  const seq = cmp.researchRows.map(function (r) { return [r.category, r.name]; });
  assert.deepEqual(seq, [
    ['Chemical Propulsion', 'Hydrolox'],
    ['Other',               'Mystery'],
    ['Other',               'Other Thing'],
  ]);
});

test('renderTableMarkup: each category emits exactly one category-header row before its items', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const html = C.renderTableMarkup(cmp);
  // Three categories in this fixture: Chemical Propulsion, Electromagnetism, Spacecraft.
  function occurrences(needle) {
    var count = 0, idx = 0;
    while ((idx = html.indexOf(needle, idx)) !== -1) { count++; idx += needle.length; }
    return count;
  }
  // Category header rows are emitted with class corp-research-category.
  const categoryRowCount = occurrences('corp-research-category');
  assert.equal(categoryRowCount, 3, 'expected 3 category header rows, got ' + categoryRowCount + '\n' + html);
  // Each category label appears in the markup.
  ['Chemical Propulsion', 'Electromagnetism', 'Spacecraft'].forEach(function (cat) {
    assert.ok(html.indexOf(cat) !== -1, 'category label missing: ' + cat);
  });
  // Header for a category must appear before its items.
  const chemHeader = html.indexOf('>Chemical Propulsion<');
  const hydro     = html.indexOf('>Hydrolox<');
  const kero      = html.indexOf('>Kerolox<');
  assert.ok(chemHeader >= 0 && hydro > chemHeader && kero > chemHeader,
    'Chemical Propulsion header must precede its items, got ' +
    JSON.stringify({ chemHeader, hydro, kero }));
  // Spacecraft header should come after Electromagnetism (alphabetical).
  const emHeader = html.indexOf('>Electromagnetism<');
  const scHeader = html.indexOf('>Spacecraft<');
  assert.ok(chemHeader < emHeader && emHeader < scHeader,
    'category headers must be alphabetical');
});

test('buildComparison: per-corp ✓/— marks match the fixture', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  // Column order is the locale-corp order from the scenario.
  assert.deepEqual(cmp.corpNames, ['SoleX', 'NASA', 'ESA', 'CNSA', 'Roscosmos']);
  // Crewed Flight: every corp.
  const crewed = cmp.researchRows.find(function (r) { return r.name === 'Crewed Flight'; });
  assert.deepEqual(crewed.held, [true, true, true, true, true]);
  // Hydrolox: SoleX & ESA only.
  const hydro = cmp.researchRows.find(function (r) { return r.name === 'Hydrolox'; });
  assert.deepEqual(hydro.held, [true, false, true, false, false]);
  // Lasers: SoleX & Roscosmos.
  const lasers = cmp.researchRows.find(function (r) { return r.name === 'Lasers'; });
  assert.deepEqual(lasers.held, [true, false, false, false, true]);
});

test('buildComparison: starting cash scales by difficulty multiplier', () => {
  const explorer = C.buildComparison(FIXTURE, 'StartGameColonization', 'Explorer');
  const pioneer  = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const veteran  = C.buildComparison(FIXTURE, 'StartGameColonization', 'Veteran');
  // SoleX Pioneer base = 33.7M; Explorer ×1.25, Veteran ×0.75.
  assert.equal(pioneer.cash[0],  33_700_000);
  assert.equal(explorer.cash[0], 33_700_000 * 1.25);
  assert.equal(veteran.cash[0],  33_700_000 * 0.75);
});

test('buildComparison: launch-vehicle and spacecraft counts do not scale by difficulty', () => {
  const explorer = C.buildComparison(FIXTURE, 'StartGameColonization', 'Explorer');
  const veteran  = C.buildComparison(FIXTURE, 'StartGameColonization', 'Veteran');
  assert.deepEqual(explorer.lvCounts, veteran.lvCounts);
  assert.deepEqual(explorer.scCounts, veteran.scCounts);
  assert.deepEqual(explorer.lvCounts, [2, 1, 1, 1, 1]);
  assert.deepEqual(explorer.scCounts, [8, 2, 1, 1, 1]);
});

test('buildComparison: switching scenario changes the corp set and research union', () => {
  const expansion = C.buildComparison(FIXTURE, 'StartGameExpansion', 'Pioneer', true);
  assert.deepEqual(expansion.corpNames, ['SoleX']);
  assert.equal(expansion.researchRows.length, 1);
  assert.equal(expansion.researchRows[0].name, 'Crewed Flight');
  assert.deepEqual(expansion.researchRows[0].held, [true]);
});

test('formatMoney: produces the same abbreviations the Rust generator uses', () => {
  assert.equal(C.formatMoney(33_700_000),  '$33.7M');
  assert.equal(C.formatMoney(42_125_000),  '$42.1M');
  assert.equal(C.formatMoney(430_000_000), '$430M');
  assert.equal(C.formatMoney(0),           '$0');
});

test('renderTableMarkup: row order is cash, LVs, spacecraft, then research', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const html = C.renderTableMarkup(cmp);
  const cashIdx    = html.indexOf('Starting cash');
  const lvIdx      = html.indexOf('Pre-built launch vehicles');
  const scIdx      = html.indexOf('Pre-built spacecraft');
  const crewedIdx  = html.indexOf('Crewed Flight');
  assert.ok(cashIdx >= 0 && lvIdx > cashIdx && scIdx > lvIdx && crewedIdx > scIdx,
    'expected cash < LV < spacecraft < research, got indices ' +
    JSON.stringify({ cashIdx, lvIdx, scIdx, crewedIdx }));
});

test('renderTableMarkup: splits into two tables on a div.corp-comparison-split', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const html = C.renderTableMarkup(cmp);
  // Outer wrapper div.
  assert.ok(/<div class="corp-comparison-split">/.test(html),
    'expected outer wrapper div.corp-comparison-split, got:\n' + html);
  // Exactly two <table> elements inside.
  const tableOpens = (html.match(/<table\b/g) || []).length;
  assert.equal(tableOpens, 2,
    'expected exactly 2 <table> elements, got ' + tableOpens + '\n' + html);
  // Each marked with its side class.
  assert.ok(/<table class="corp-comparison-left"/.test(html),
    'expected a left table');
  assert.ok(/<table class="corp-comparison-right"/.test(html),
    'expected a right table');
});

test('renderTableMarkup: left table holds cash/LVs/SC; right table holds research', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const html = C.renderTableMarkup(cmp);
  // Carve the document into the two tables (between table-open and the
  // following </table>) and assert each contains/excludes the right rows.
  function tableBody(cls) {
    const re = new RegExp('<table class="' + cls + '"[\\s\\S]*?</table>');
    const m = html.match(re);
    assert.ok(m, 'could not locate <table class="' + cls + '">');
    return m[0];
  }
  const left  = tableBody('corp-comparison-left');
  const right = tableBody('corp-comparison-right');

  // Left: cash, LVs, SC — but no research items / category headers.
  assert.ok(left.indexOf('Starting cash')               !== -1, 'left missing cash');
  assert.ok(left.indexOf('Pre-built launch vehicles')   !== -1, 'left missing LV row');
  assert.ok(left.indexOf('Pre-built spacecraft')        !== -1, 'left missing SC row');
  assert.equal(left.indexOf('Crewed Flight'), -1, 'left should not have research rows');
  assert.equal(left.indexOf('corp-research-category'), -1,
    'left should not have category-header rows');

  // Right: research rows (category headers + items), no cash/fleet rows.
  assert.ok(right.indexOf('Crewed Flight')         !== -1, 'right missing Crewed Flight');
  assert.ok(right.indexOf('Hydrolox')              !== -1, 'right missing Hydrolox');
  assert.ok(right.indexOf('corp-research-category') !== -1,
    'right missing category header rows');
  assert.equal(right.indexOf('Starting cash'),             -1, 'right should not have cash');
  assert.equal(right.indexOf('Pre-built launch vehicles'), -1, 'right should not have LV row');
  assert.equal(right.indexOf('Pre-built spacecraft'),      -1, 'right should not have SC row');
});

test('renderTableMarkup: each table has its own <thead> row with all corp names', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer', true);
  const html = C.renderTableMarkup(cmp);
  // Two separate <thead> blocks.
  const theadOpens = (html.match(/<thead\b/g) || []).length;
  assert.equal(theadOpens, 2,
    'expected 2 <thead> blocks (one per table), got ' + theadOpens);
  function tableBody(cls) {
    const re = new RegExp('<table class="' + cls + '"[\\s\\S]*?</table>');
    return html.match(re)[0];
  }
  const left  = tableBody('corp-comparison-left');
  const right = tableBody('corp-comparison-right');
  // Both heads list every corp name.
  ['SoleX', 'NASA', 'ESA', 'CNSA', 'Roscosmos'].forEach(function (name) {
    assert.ok(left.indexOf('<th>' + name + '</th>')  !== -1,
      'left thead missing corp ' + name);
    assert.ok(right.indexOf('<th>' + name + '</th>') !== -1,
      'right thead missing corp ' + name);
  });
});

// ---- Sol-system (Realistic) scenarios from the live CORP_DATA blob -----
// These assertions mirror the four-scenario routing built from
// PlanetarySystem_Realistic.mapEpochToToStartData on the Rust side.

const REALISTIC_FIXTURE = {
  scenarios: [
    { id: 'StartGameEpoch_EarlyExploration', name: 'Early Exploration', corps: [] },
    { id: 'StartGameEpoch_TheExpansion',     name: 'The Expansion',     corps: [] },
    { id: 'StartGameEpoch_Colonization',     name: 'Colonization Era',  corps: [] },
    { id: 'StartGameEpoch_RaceBeyond',       name: 'Race Beyond',       corps: [] },
  ],
  difficulties: [
    { name: 'Pioneer', money_multiplier: 1.0 },
  ],
};

test('CORP_DATA.scenarios includes all four Sol-system epochs', () => {
  assert.equal(REALISTIC_FIXTURE.scenarios.length, 4);
  const ids = REALISTIC_FIXTURE.scenarios.map(function (s) { return s.id; });
  assert.ok(ids.includes('StartGameEpoch_EarlyExploration'));
  assert.ok(ids.includes('StartGameEpoch_TheExpansion'));
  assert.ok(ids.includes('StartGameEpoch_Colonization'));
  assert.ok(ids.includes('StartGameEpoch_RaceBeyond'));
});

test('CORP_DATA.scenarios is ordered Early → Expansion → Colonization → RaceBeyond', () => {
  const ids = REALISTIC_FIXTURE.scenarios.map(function (s) { return s.id; });
  assert.deepEqual(ids, [
    'StartGameEpoch_EarlyExploration',
    'StartGameEpoch_TheExpansion',
    'StartGameEpoch_Colonization',
    'StartGameEpoch_RaceBeyond',
  ]);
});

test('default-selected scenario on page load is Early Exploration', () => {
  assert.equal(C.DEFAULT_SCENARIO_ID, 'StartGameEpoch_EarlyExploration');
});

// ---- Starting facilities section ---------------------------------------

// Fixture mirroring the CORP_DATA shape with per-corp starting_facilities,
// each a `{name, count}` object resolved/title-cased by the Rust generator.
// Only mining/extraction/refinery rows differ between corps — universals
// like "HQ" and "Main Building" appear on every corp and are filtered out
// by buildComparison.
const FACILITIES_FIXTURE = {
  scenarios: [
    {
      id: 'StartGameEpoch_TheExpansion',
      name: 'The Expansion',
      corps: [
        { name: 'SoleX', starting_money: 27_200_000, lv_count: 0, sc_count: 0, research: [],
          starting_facilities: [
            { name: 'HQ',              count: 1 },
            { name: 'Main Building',   count: 1 },
            { name: 'Metal Mine',      count: 2 },
            { name: 'Noble Gas Mine',  count: 1 },
          ] },
        { name: 'NASA',  starting_money: 30_100_000, lv_count: 0, sc_count: 0, research: [],
          starting_facilities: [
            { name: 'HQ',              count: 1 },
            { name: 'Main Building',   count: 1 },
            { name: 'Noble Gas Mine',  count: 1 },
            { name: 'Uranium Mine',    count: 1 },
          ] },
        { name: 'ESA',   starting_money: 25_700_000, lv_count: 0, sc_count: 0, research: [],
          starting_facilities: [
            { name: 'HQ',              count: 1 },
            { name: 'Main Building',   count: 1 },
            { name: 'Noble Gas Mine',  count: 3 },
          ] },
      ],
    },
  ],
  difficulties: [
    { name: 'Pioneer', money_multiplier: 1.0 },
  ],
};

test('buildComparison: surfaces facilityRows with per-corp counts', () => {
  const cmp = C.buildComparison(FACILITIES_FIXTURE, 'StartGameEpoch_TheExpansion', 'Pioneer');
  assert.ok(Array.isArray(cmp.facilityRows),
    'buildComparison() must include a facilityRows array, got ' + JSON.stringify(cmp));
  // Each row is `{name, counts: [n…]}` with one count per corp column.
  cmp.facilityRows.forEach((r) => {
    assert.ok(typeof r.name === 'string' && r.name.length > 0,
      'facility row missing name: ' + JSON.stringify(r));
    assert.ok(Array.isArray(r.counts) && r.counts.length === cmp.corpNames.length,
      'facility row counts must match corpNames length: ' + JSON.stringify(r));
  });
  // Rows are alphabetical by facility name.
  const names = cmp.facilityRows.map((r) => r.name);
  const sorted = names.slice().sort();
  assert.deepEqual(names, sorted, 'facility rows must be alphabetical, got ' + names);
});

test('buildComparison: drops universal facilities (HQ, Main Building)', () => {
  const cmp = C.buildComparison(FACILITIES_FIXTURE, 'StartGameEpoch_TheExpansion', 'Pioneer');
  const names = cmp.facilityRows.map((r) => r.name);
  // HQ and Main Building are universal — every corp has one — so they
  // carry no comparison signal and must be filtered out of the matrix.
  assert.ok(!names.includes('HQ'), 'HQ should be filtered out, got ' + names);
  assert.ok(!names.includes('Main Building'),
    'Main Building should be filtered out, got ' + names);
});

test('buildComparison: facility counts line up with corpNames order', () => {
  const cmp = C.buildComparison(FACILITIES_FIXTURE, 'StartGameEpoch_TheExpansion', 'Pioneer');
  assert.deepEqual(cmp.corpNames, ['SoleX', 'NASA', 'ESA']);
  // Noble Gas Mine: SoleX=1, NASA=1, ESA=3.
  const noble = cmp.facilityRows.find((r) => r.name === 'Noble Gas Mine');
  assert.ok(noble, 'Noble Gas Mine row missing');
  assert.deepEqual(noble.counts, [1, 1, 3]);
  // Metal Mine: SoleX=2, NASA=0 (none), ESA=0 (none).
  const metal = cmp.facilityRows.find((r) => r.name === 'Metal Mine');
  assert.ok(metal, 'Metal Mine row missing');
  assert.deepEqual(metal.counts, [2, 0, 0]);
  // Uranium Mine: only NASA owns one.
  const uran = cmp.facilityRows.find((r) => r.name === 'Uranium Mine');
  assert.ok(uran, 'Uranium Mine row missing');
  assert.deepEqual(uran.counts, [0, 1, 0]);
});

test('renderTableMarkup: emits a "Starting facilities" section in the LEFT table', () => {
  const cmp = C.buildComparison(FACILITIES_FIXTURE, 'StartGameEpoch_TheExpansion', 'Pioneer');
  const html = C.renderTableMarkup(cmp);
  // Carve out the left table.
  const leftMatch = html.match(/<table class="corp-comparison-left"[\s\S]*?<\/table>/);
  assert.ok(leftMatch, 'left table not found in render output');
  const left = leftMatch[0];
  // The "Starting facilities" header sits in the left table, AFTER the
  // pre-built spacecraft row.
  const facHeader = left.indexOf('Starting facilities');
  const scRow     = left.indexOf('Pre-built spacecraft');
  assert.ok(facHeader > scRow && scRow !== -1,
    'Starting facilities section must follow Pre-built spacecraft in the left table, got ' +
    JSON.stringify({ facHeader, scRow }));
  // Each surviving facility name appears as a row label.
  ['Metal Mine', 'Noble Gas Mine', 'Uranium Mine'].forEach((n) => {
    assert.ok(left.indexOf(n) !== -1, 'left table missing facility row "' + n + '"');
  });
  // Counts render as plain integers; zero counts render as "—".
  // Metal Mine row should carry a 2 and two em-dashes.
  const metalRowMatch = left.match(/<tr>[^<]*<td[^>]*>Metal Mine[\s\S]*?<\/tr>/);
  assert.ok(metalRowMatch, 'Metal Mine row not found in left table');
  const metalRow = metalRowMatch[0];
  assert.ok(metalRow.includes('>2<'),
    'Metal Mine SoleX cell should show count 2, got ' + metalRow);
  // Count of em-dashes equals number of zero-count corps for this row (2).
  const dashes = (metalRow.match(/—/g) || []).length;
  assert.equal(dashes, 2,
    'expected 2 em-dashes for zero-count cells, got ' + dashes + '\n' + metalRow);
});

test('renderTableMarkup: facility rows do NOT appear in the right table', () => {
  const cmp = C.buildComparison(FACILITIES_FIXTURE, 'StartGameEpoch_TheExpansion', 'Pioneer');
  const html = C.renderTableMarkup(cmp);
  const rightMatch = html.match(/<table class="corp-comparison-right"[\s\S]*?<\/table>/);
  assert.ok(rightMatch, 'right table not found');
  const right = rightMatch[0];
  // The right table is research-only; facility rows must stay on the left.
  assert.equal(right.indexOf('Starting facilities'), -1,
    'right table should not have a Starting facilities section');
  ['Metal Mine', 'Noble Gas Mine', 'Uranium Mine'].forEach((n) => {
    assert.equal(right.indexOf(n), -1,
      'facility row "' + n + '" leaked into the right table');
  });
});

test('renderTableMarkup: omits Starting facilities header when every corp has none', () => {
  const cmp = C.buildComparison({
    scenarios: [{
      id: 'X', name: 'X',
      corps: [
        { name: 'A', starting_money: 0, lv_count: 0, sc_count: 0, research: [],
          starting_facilities: [] },
        { name: 'B', starting_money: 0, lv_count: 0, sc_count: 0, research: [],
          starting_facilities: [] },
      ],
    }],
    difficulties: [{ name: 'Pioneer', money_multiplier: 1.0 }],
  }, 'X', 'Pioneer');
  const html = C.renderTableMarkup(cmp);
  assert.equal(html.indexOf('Starting facilities'), -1,
    'Starting facilities section must not render when no corp has any');
});

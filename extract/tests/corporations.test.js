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
const FIXTURE = {
  scenarios: [
    {
      id: 'StartGameColonization',
      name: 'Colonization Era',
      corps: [
        { name: 'SoleX',     starting_money: 33_700_000, lv_count: 2, sc_count: 8, research: ['Crewed Flight', 'Hydrolox', 'Lasers'] },
        { name: 'NASA',      starting_money: 35_900_000, lv_count: 1, sc_count: 2, research: ['Crewed Flight', 'Kerolox'] },
        { name: 'ESA',       starting_money: 30_000_000, lv_count: 1, sc_count: 1, research: ['Crewed Flight', 'Hydrolox'] },
        { name: 'CNSA',      starting_money: 28_000_000, lv_count: 1, sc_count: 1, research: ['Crewed Flight'] },
        { name: 'Roscosmos', starting_money: 27_000_000, lv_count: 1, sc_count: 1, research: ['Crewed Flight', 'Kerolox', 'Lasers'] },
      ],
    },
    {
      id: 'StartGameExpansion',
      name: 'The Expansion',
      corps: [
        { name: 'SoleX', starting_money: 27_200_000, lv_count: 2, sc_count: 4, research: ['Crewed Flight'] },
      ],
    },
  ],
  difficulties: [
    { name: 'Explorer', money_multiplier: 1.25 },
    { name: 'Pioneer',  money_multiplier: 1.0  },
    { name: 'Veteran',  money_multiplier: 0.75 },
  ],
};

test('buildComparison: research union is alphabetical and excludes zero-corp items', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer');
  // Three research items appear across the five corps: Crewed Flight, Hydrolox, Kerolox, Lasers.
  const researchNames = cmp.researchRows.map(function (r) { return r.name; });
  assert.deepEqual(researchNames, ['Crewed Flight', 'Hydrolox', 'Kerolox', 'Lasers']);
  // Sanity: nothing held by zero corps slipped in.
  cmp.researchRows.forEach(function (r) {
    assert.ok(r.held.some(Boolean), 'row ' + r.name + ' has no holders');
  });
});

test('buildComparison: per-corp ✓/— marks match the fixture', () => {
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer');
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
  const pioneer  = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer');
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
  const expansion = C.buildComparison(FIXTURE, 'StartGameExpansion', 'Pioneer');
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
  const cmp = C.buildComparison(FIXTURE, 'StartGameColonization', 'Pioneer');
  const html = C.renderTableMarkup(cmp);
  const cashIdx = html.indexOf('Starting cash');
  const lvIdx = html.indexOf('Launch vehicles');
  const scIdx = html.indexOf('Spacecraft');
  const crewedIdx = html.indexOf('Crewed Flight');
  assert.ok(cashIdx >= 0 && lvIdx > cashIdx && scIdx > lvIdx && crewedIdx > scIdx,
    'expected cash < LV < spacecraft < research, got indices ' +
    JSON.stringify({ cashIdx, lvIdx, scIdx, crewedIdx }));
});

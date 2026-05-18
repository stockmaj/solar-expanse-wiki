// Node-only unit tests for calculator.js — additive-stacking math.
// Run with `node docs/assets/js/calculator.test.js` from anywhere.

const { applyReductions, workerTotal, powerNetTotal, addSaved, removeSaved, iconFile, fmtAbbrev, crewTransportMass, buildDayTotal, encodeShareState, decodeShareState } = require('./calculator.js');

let passed = 0;
let failed = 0;

function eq(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed++;
    console.log('  ok  - ' + label);
  } else {
    failed++;
    console.log('  FAIL - ' + label);
    console.log('    expected: ' + e);
    console.log('    actual:   ' + a);
  }
}

const facility = {
  id: 'build_outpost',
  name: 'Outpost',
  category: 'Habitation',
  build_cost: [
    { resource: 'metal', amount: 100 },
    { resource: 'electronics', amount: 50 },
  ],
};

// 1. Zero reductions: cost unchanged.
eq(
  applyReductions(facility, []),
  { metal: 100, electronics: 50 },
  'no reductions leave cost unchanged'
);

// 2. One 35% reduction targeting the facility.
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 35, affects: ['build_outpost'], affects_all: false },
  ]),
  { metal: 65, electronics: 33 },
  'single 35% reduction multiplies cost by 0.65'
);

// 3. Two reductions stacking additively (35% + 40% = 75% -> ×0.25).
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 35, affects: ['build_outpost'], affects_all: false },
    { id: 'r2', kind: 'BuildCost', percent: 40, affects: ['build_outpost'], affects_all: false },
  ]),
  { metal: 25, electronics: 13 },
  'two reductions stack additively, not multiplicatively'
);

// 4. affects_all=true applies even when affects: [].
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 20, affects: [], affects_all: true },
  ]),
  { metal: 80, electronics: 40 },
  'affects_all=true applies regardless of affects list'
);

// 5. affects_all=false and facility NOT in affects -> no-op.
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 50, affects: ['build_habitat'], affects_all: false },
  ]),
  { metal: 100, electronics: 50 },
  'reduction not targeting this facility is ignored'
);

// 6. Sum >= 100% clamps to multiplier 0.
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 60, affects: ['build_outpost'], affects_all: false },
    { id: 'r2', kind: 'BuildCost', percent: 50, affects: [], affects_all: true },
  ]),
  { metal: 0, electronics: 0 },
  'sum >= 100% clamps multiplier to 0'
);

// 7. Non-BuildCost kinds are ignored (PowerProduction / ReduceCrewRequirements).
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'PowerProduction', percent: 50, affects: ['build_outpost'], affects_all: false },
    { id: 'r2', kind: 'ReduceCrewRequirements', percent: 50, affects: [], affects_all: true },
  ]),
  { metal: 100, electronics: 50 },
  'non-BuildCost reductions are ignored for resource math'
);

// 8. Mixed: one BuildCost applies, one PowerProduction ignored.
eq(
  applyReductions(facility, [
    { id: 'r1', kind: 'BuildCost', percent: 25, affects: ['build_outpost'], affects_all: false },
    { id: 'r2', kind: 'PowerProduction', percent: 99, affects: [], affects_all: true },
  ]),
  { metal: 75, electronics: 38 },
  'mixed reduction kinds — only BuildCost contributes'
);

// 9. Rounding: 50 × 0.65 = 32.5 -> 33 (round to nearest).
eq(
  applyReductions(
    { id: 'x', name: 'X', category: 'Y', build_cost: [{ resource: 'r', amount: 50 }] },
    [{ id: 'a', kind: 'BuildCost', percent: 35, affects: [], affects_all: true }]
  ),
  { r: 33 },
  'half-up rounding (32.5 -> 33)'
);

// ----- Worker total (ReduceCrewRequirements) ------------------------------

const habitat = {
  id: 'build_habitat',
  workers_required: 100,
  energy_consumption: 1,
  power_production: 0,
};
const powerPlant = {
  id: 'build_power_chemical',
  workers_required: 100,
  energy_consumption: 0,
  power_production: 400,
};
const mine = {
  id: 'build_alloymine',
  workers_required: 5,
  energy_consumption: 0.5,
  power_production: 0,
};

eq(
  workerTotal([{ facility: habitat, count: 1 }, { facility: mine, count: 2 }], []),
  110,
  'worker total: no reductions sums workers × count'
);

eq(
  workerTotal(
    [{ facility: habitat, count: 1 }],
    [{ id: 'r', kind: 'ReduceCrewRequirements', percent: 10, affects: [], affects_all: true }]
  ),
  90,
  'worker total: 10% crew reduction applies to all'
);

eq(
  workerTotal(
    [{ facility: habitat, count: 1 }],
    [
      { id: 'r1', kind: 'ReduceCrewRequirements', percent: 60, affects: [], affects_all: true },
      { id: 'r2', kind: 'ReduceCrewRequirements', percent: 50, affects: [], affects_all: true },
    ]
  ),
  0,
  'worker total: sum >= 100% clamps to 0'
);

eq(
  workerTotal(
    [{ facility: habitat, count: 1 }, { facility: mine, count: 1 }],
    [{ id: 'r', kind: 'ReduceCrewRequirements', percent: 20, affects: ['build_habitat'], affects_all: false }]
  ),
  85,
  'worker total: targeted reduction only affects matching facility'
);

eq(
  workerTotal(
    [{ facility: habitat, count: 1 }],
    [{ id: 'r', kind: 'BuildCost', percent: 99, affects: [], affects_all: true }]
  ),
  100,
  'worker total: non-crew reductions are ignored'
);

// ----- Power net total (PowerProduction) ----------------------------------

eq(
  powerNetTotal(
    [{ facility: powerPlant, count: 1 }, { facility: mine, count: 4 }],
    []
  ),
  -398,
  'power net: no reductions → consumption (2) minus production (400) = -398 (surplus)'
);

eq(
  powerNetTotal([{ facility: mine, count: 4 }], []),
  2,
  'power net: positive value means deficit (need to import power)'
);

eq(
  powerNetTotal(
    [{ facility: powerPlant, count: 1 }],
    [{ id: 'r', kind: 'PowerProduction', percent: 50, affects: [], affects_all: true }]
  ),
  -600,
  'power net: PowerProduction bonus scales production up (400 × 1.5 = 600)'
);

eq(
  powerNetTotal(
    [{ facility: powerPlant, count: 1 }],
    [{ id: 'r', kind: 'PowerProduction', percent: 25, affects: ['build_other'], affects_all: false }]
  ),
  -400,
  'power net: targeted production bonus skips non-matching facility'
);

eq(
  powerNetTotal(
    [{ facility: powerPlant, count: 1 }],
    [{ id: 'r', kind: 'BuildCost', percent: 99, affects: [], affects_all: true }]
  ),
  -400,
  'power net: non-power reductions are ignored'
);

// ----- Saved lists --------------------------------------------------------

eq(
  addSaved([], 'Mars Colony', { build_habitat: 2 }),
  [{ name: 'Mars Colony', placed: { build_habitat: 2 } }],
  'addSaved: appends a new entry'
);

eq(
  addSaved(
    [{ name: 'Mars Colony', placed: { build_habitat: 1 } }],
    'Mars Colony',
    { build_habitat: 5, build_alloymine: 1 }
  ),
  [{ name: 'Mars Colony', placed: { build_habitat: 5, build_alloymine: 1 } }],
  'addSaved: overwrites entry with same name'
);

eq(
  addSaved(
    [{ name: 'A', placed: { x: 1 } }],
    'B',
    { y: 2 }
  ),
  [{ name: 'A', placed: { x: 1 } }, { name: 'B', placed: { y: 2 } }],
  'addSaved: preserves existing entries when adding new'
);

eq(
  addSaved([], '   ', { x: 1 }),
  [],
  'addSaved: blank name is rejected (no entry added)'
);

eq(
  addSaved([], '  Mars  ', { x: 1 }),
  [{ name: 'Mars', placed: { x: 1 } }],
  'addSaved: trims surrounding whitespace from name'
);

eq(
  removeSaved(
    [
      { name: 'A', placed: { x: 1 } },
      { name: 'B', placed: { y: 2 } },
    ],
    'A'
  ),
  [{ name: 'B', placed: { y: 2 } }],
  'removeSaved: drops the named entry'
);

eq(
  removeSaved([{ name: 'A', placed: {} }], 'Z'),
  [{ name: 'A', placed: {} }],
  'removeSaved: no-op when name not present'
);

// Mutation guards — both functions should return a new array, not mutate input.
{
  const original = [{ name: 'A', placed: { x: 1 } }];
  const frozen = JSON.stringify(original);
  addSaved(original, 'B', { y: 1 });
  removeSaved(original, 'A');
  eq(JSON.stringify(original), frozen, 'addSaved/removeSaved do not mutate input array');
}

// ----- Icon file mapping --------------------------------------------------

eq(iconFile('metal'), 'metal.png', 'iconFile: default = id + .png');
eq(iconFile('hel3'), 'HEL3.png', 'iconFile: hel3 is an all-caps override');
eq(iconFile('human'), 'human.png', 'iconFile: human maps directly');
eq(iconFile('energy'), 'energy.png', 'iconFile: energy maps directly');

// ----- fmtAbbrev ----------------------------------------------------------

eq(fmtAbbrev(0), '0', 'fmtAbbrev: zero');
eq(fmtAbbrev(125), '125', 'fmtAbbrev: below 1k stays as integer');
eq(fmtAbbrev(999), '999', 'fmtAbbrev: 999 stays as integer (< 1k threshold)');
eq(fmtAbbrev(1000), '1k', 'fmtAbbrev: exactly 1000 → 1k');
eq(fmtAbbrev(1200), '1.2k', 'fmtAbbrev: 1200 → 1.2k');
eq(fmtAbbrev(60000), '60k', 'fmtAbbrev: 60000 → 60k');
eq(fmtAbbrev(200000), '200k', 'fmtAbbrev: 200000 → 200k');
eq(fmtAbbrev(1500000), '1.5M', 'fmtAbbrev: 1.5M');
eq(fmtAbbrev(3000000000), '3B', 'fmtAbbrev: billions');

// ----- Crew transport -----------------------------------------------------

const compartment = { id: 'module_crew_compartment', capacity: 5, mass: 5 };
const medium      = { id: 'module_crew_medium', capacity: 20, mass: 15 };
const large       = { id: 'module_crew_large', capacity: 100, mass: 60 };

eq(crewTransportMass(0, compartment),  { capsules: 0, mass: 0 }, 'crew: 0 humans → 0 capsules');
eq(crewTransportMass(5, compartment),  { capsules: 1, mass: 10 }, 'crew: 5 humans fill 1 compartment = 5T + 5T = 10');
eq(crewTransportMass(35, compartment), { capsules: 7, mass: 70 }, 'crew: 35 humans = 7 compartments = 35T + 35T = 70');
eq(crewTransportMass(12, compartment), { capsules: 3, mass: 27 }, 'crew: 12 humans → 3 caps (15T) + 12 (12T) = 27 (partial last)');
eq(crewTransportMass(35, medium),      { capsules: 2, mass: 65 }, 'crew: 35 humans in mediums → 2 caps (30T) + 35T = 65');
eq(crewTransportMass(150, large),      { capsules: 2, mass: 270 }, 'crew: 150 humans in larges → 2 (120T) + 150T = 270');
eq(crewTransportMass(5, null),         { capsules: 0, mass: 0 }, 'crew: no transport selected → 0');

// ----- Build days ---------------------------------------------------------

eq(buildDayTotal([]), 0, 'build days: empty placed → 0');
eq(
  buildDayTotal([
    { facility: { build_time_days: 150 }, count: 2 },
    { facility: { build_time_days: 100 }, count: 1 },
    { facility: {}, count: 3 },
  ]),
  400,
  'build days: serial sum of build_time × count, missing field treated as 0'
);

// ----- Share-by-URL round-trip --------------------------------------------

// Node lacks atob/btoa by default in old versions; the JS expects globals.
// Confirm they're around (Node 16+ provides them).
if (typeof atob === 'undefined') { global.atob = require('buffer').Buffer.from(arguments[0], 'base64').toString('binary'); }

const fullState = {
  placed: { build_habitat: 3, build_alloymine: 1, module_crew_compartment: 2 },
  checked: { research_lifesup_10: true },
  spacecraft: 'spacecraft_chem_large',
  onSite: { metal: 100, human: 5 },
};
eq(decodeShareState(encodeShareState(fullState)), fullState, 'share: round-trip preserves full state');

eq(
  decodeShareState(encodeShareState({
    placed: {}, checked: {}, spacecraft: null, onSite: {},
  })),
  { placed: {}, checked: {}, spacecraft: null, onSite: {} },
  'share: round-trip preserves empty state'
);

eq(decodeShareState('not-base64!!!'), null, 'share: bad input → null');

// URL safety — no characters that need encoding in a query string.
const encoded = encodeShareState(fullState);
eq(/^[A-Za-z0-9_-]+$/.test(encoded), true, 'share: encoded form is URL-safe (no +/=)');

console.log('\n' + passed + ' passed, ' + failed + ' failed');
process.exit(failed === 0 ? 0 : 1);

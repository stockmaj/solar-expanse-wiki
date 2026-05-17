// Node-only unit tests for calculator.js — additive-stacking math.
// Run with `node docs/assets/js/calculator.test.js` from anywhere.

const { applyReductions } = require('./calculator.js');

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

console.log('\n' + passed + ' passed, ' + failed + ' failed');
process.exit(failed === 0 ? 0 : 1);

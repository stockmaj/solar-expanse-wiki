// Node `--test` suite for the launch-window math in docs/assets/js/launch-windows.js
//
// Run with:   node --test extract/tests/launch-windows.test.js
//
// The script imports the JS module (CommonJS export at the bottom of the
// browser script).  Most tests use the *game's* orbital data — the longitudes
// the game ships, anchored at our 1959-01-01 baseline — so the "known" dates
// here are the dates the calculator should produce given that data, not the
// real-world J2000 windows.  A couple of sanity tests check that the synodic
// spacing matches real-world (Earth-Mars ~780 days, Earth-Venus ~584 days)
// because that depends only on semi-major axes, not on the longitude anchor.

const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');

const lw = require(path.join(__dirname, '..', '..', 'docs', 'assets', 'js', 'launch-windows.js'));

// J2000 epoch as a Unix-ms timestamp (2000-01-01 12:00:00 UTC) — the
// canonical anchor for the real-world orbital elements used in these tests.
const J2000_MS = Date.UTC(2000, 0, 1, 12, 0, 0);

// Real-world J2000 orbital elements, used as a sanity layer for the math.
// The game ships slightly different mean longitudes (anchored at a different
// epoch) — those tests are below in `gameDataSanity`.
const REAL_EARTH = { a: 1.000001, longitude: 100.46 };  // J2000 mean longitude
const REAL_MARS  = { a: 1.523679, longitude: 355.45 };
const REAL_VENUS = { a: 0.723332, longitude: 181.98 };

const DAY = 86400000;
function daysBetween(a, b) { return Math.abs(a.getTime() - b.getTime()) / DAY; }

test('Hohmann transfer time matches the Earth-Mars textbook value (~258 days)', () => {
  const years = lw.hohmannTransferYears(1, 1.524);
  const days = years * 365.25;
  assert.ok(Math.abs(days - 258) < 5, `expected ~258 days, got ${days.toFixed(1)}`);
});

test('Hohmann transfer Earth → Ceres (~470 days)', () => {
  const years = lw.hohmannTransferYears(1, 2.77);
  const days = years * 365.25;
  assert.ok(Math.abs(days - 470) < 15, `expected ~470 days, got ${days.toFixed(1)}`);
});

test('Earth-Mars synodic period is ~780 days', () => {
  // Synodic = 1 / |1/T_e - 1/T_m| (years).  We test via two consecutive windows.
  const w1 = lw.nextWindow(REAL_EARTH, REAL_MARS, new Date('2020-01-01T00:00:00Z'), J2000_MS);
  const w2 = lw.nextWindow(REAL_EARTH, REAL_MARS, new Date(w1.getTime() + DAY), J2000_MS);
  const gap = daysBetween(w1, w2);
  assert.ok(Math.abs(gap - 779.94) < 1, `expected 779.94 days between Mars windows, got ${gap.toFixed(2)}`);
});

test('Earth-Venus synodic period is ~584 days', () => {
  const w1 = lw.nextWindow(REAL_EARTH, REAL_VENUS, new Date('2020-01-01T00:00:00Z'), J2000_MS);
  const w2 = lw.nextWindow(REAL_EARTH, REAL_VENUS, new Date(w1.getTime() + DAY), J2000_MS);
  const gap = daysBetween(w1, w2);
  assert.ok(Math.abs(gap - 583.92) < 1, `expected 583.92 days between Venus windows, got ${gap.toFixed(2)}`);
});

test('Earth-Mars 2020 window (real-world Perseverance: 2020-07-22)', () => {
  // Using J2000 mean longitudes anchored at J2000, the next Earth-Mars
  // Hohmann window after 2020-04-01 should land within a month of the
  // real-world Perseverance launch (2020-07-22).
  const w = lw.nextWindow(REAL_EARTH, REAL_MARS, new Date('2020-04-01T00:00:00Z'), J2000_MS);
  const real = new Date('2020-07-22T00:00:00Z');
  const days = daysBetween(w, real);
  assert.ok(days < 35, `expected within 35 days of 2020-07-22, got ${w.toISOString().slice(0,10)} (delta ${days.toFixed(0)} days)`);
});

test('Earth-Venus 2020 window (real-world ~2020-04)', () => {
  const w = lw.nextWindow(REAL_EARTH, REAL_VENUS, new Date('2020-01-01T00:00:00Z'), J2000_MS);
  const real = new Date('2020-04-04T00:00:00Z');
  const days = daysBetween(w, real);
  assert.ok(days < 35, `expected within 35 days of 2020-04-04, got ${w.toISOString().slice(0,10)} (delta ${days.toFixed(0)} days)`);
});

test('nextNWindows returns N distinct dates spaced by synodic period', () => {
  const ws = lw.nextNWindows(REAL_EARTH, REAL_MARS, new Date('2020-01-01T00:00:00Z'), 5, J2000_MS);
  assert.equal(ws.length, 5);
  for (let i = 1; i < ws.length; i++) {
    const gap = daysBetween(ws[i - 1], ws[i]);
    assert.ok(Math.abs(gap - 779.94) < 1, `gap[${i}] = ${gap.toFixed(2)} not ~779.94`);
  }
});

test('Identical-orbit pair returns null (no transfer)', () => {
  const w = lw.nextWindow(REAL_EARTH, REAL_EARTH, new Date('2020-01-01T00:00:00Z'));
  assert.equal(w, null);
});

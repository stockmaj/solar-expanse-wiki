// Node `--test` suite for the gravity-assist math in
// docs/assets/js/gravity-assist.js.
//
// Run with:  node --test extract/tests/gravity-assist.test.js
//
// All tests use real-world J2000 mean longitudes anchored at J2000 so that
// quantities like "v_inf at Earth for a Hohmann to Mars" come out at the
// textbook ~2.94 km/s.

const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');

const ga = require(path.join(__dirname, '..', '..', 'docs', 'assets', 'js', 'gravity-assist.js'));

const J2000_MS = Date.UTC(2000, 0, 1, 12, 0, 0);
const DAY = 86400000;
const YEAR_DAYS = 365.25;
const MU = 4 * Math.PI * Math.PI;                // AU^3 / yr^2
const AU_PER_YR_TO_KM_S = 4.74047;

const EARTH = { a: 1.000001, longitude: 100.46, name: 'Earth' };
const MARS  = { a: 1.523679, longitude: 355.45, name: 'Mars' };
const VENUS = { a: 0.723332, longitude: 181.98, name: 'Venus' };
const CERES = { a: 2.766,    longitude: 95.99,  name: 'Ceres' };

function vmag(v) { return Math.sqrt(v[0]*v[0] + v[1]*v[1]); }

test('Lambert: ~180° Earth→Mars Hohmann recovers vis-viva velocity at Earth', () => {
  // Place Earth and Mars on (very nearly) opposite sides of the Sun and
  // ask for a transfer time equal to half the Hohmann period.  The
  // resulting v1 should match the vis-viva speed at r = 1 AU on an
  // ellipse with a = (1 + 1.524)/2, i.e. v = sqrt(μ (2/1 - 1/a_t)).
  // We use 179.5° because exact 180° is a Lambert singularity (the
  // transfer plane is undefined when r1, r2 are collinear).
  var aT = (1.0 + 1.524) / 2;                    // semi-major of transfer
  var tof = 0.5 * Math.pow(aT, 1.5);             // years
  var ang = 179.5 * Math.PI / 180;
  var r1 = [1.0, 0.0];
  var r2 = [1.524 * Math.cos(ang), 1.524 * Math.sin(ang)];
  var sol = ga.lambert(r1, r2, tof, true);
  assert.ok(sol, 'Lambert returned a solution');
  var visViva = Math.sqrt(MU * (2/1.0 - 1/aT));  // AU/yr at Earth's r
  var got = vmag(sol.v1);
  var ratio = got / visViva;
  assert.ok(Math.abs(ratio - 1) < 0.05,
    `expected v1 ≈ vis-viva (${visViva.toFixed(4)} AU/yr), got ${got.toFixed(4)} (ratio ${ratio.toFixed(3)})`);
});

test('Lambert: Hohmann v_inf at Earth for Mars transfer is ~2.94 km/s', () => {
  // The classic textbook result for an Earth-Mars Hohmann transfer.
  var aT = (1.0 + 1.524) / 2;
  var tof = 0.5 * Math.pow(aT, 1.5);
  var ang = 179.5 * Math.PI / 180;
  var sol = ga.lambert([1.0, 0.0],
                       [1.524 * Math.cos(ang), 1.524 * Math.sin(ang)],
                       tof, true);
  assert.ok(sol);
  // Earth's circular orbital speed
  var vEarth = Math.sqrt(MU / 1.0);              // AU/yr
  // Near 180° the transfer velocity at perihelion is essentially tangential.
  var vT = vmag(sol.v1);
  var vInf = Math.abs(vT - vEarth) * AU_PER_YR_TO_KM_S;
  assert.ok(Math.abs(vInf - 2.94) < 0.5,
    `expected v_inf ≈ 2.94 km/s, got ${vInf.toFixed(2)}`);
});

test('positionAt: Earth at J2000 is near (cos 100.46°, sin 100.46°)', () => {
  var p = ga.positionAt(EARTH, J2000_MS, J2000_MS);
  var L = 100.46 * Math.PI / 180;
  assert.ok(Math.abs(p.r[0] - Math.cos(L)) < 1e-6);
  assert.ok(Math.abs(p.r[1] - Math.sin(L)) < 1e-6);
  // Circular velocity magnitude = sqrt(μ/a) ≈ 2π for a = 1.
  assert.ok(Math.abs(vmag(p.v) - 2 * Math.PI) < 1e-4);
});

test('bestTrajectory: Earth→Venus→Ceres beats Earth→Ceres direct in a 5-yr window', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 5 * YEAR_DAYS * DAY;
  var withFlyby = ga.bestTrajectory({
    earth: EARTH, flybyBody: VENUS, target: CERES,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  var direct = ga.bestDirect({
    earth: EARTH, target: CERES,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(withFlyby, 'flyby trajectory found');
  assert.ok(direct, 'direct trajectory found');
  // With a free-rotation flyby the gravity-assist path should be at worst
  // comparable to direct (the swing-by adds energy if used right).  Allow
  // a small slack to account for coarse-grid noise.
  assert.ok(withFlyby.totalDvKms <= direct.totalDvKms + 0.5,
    `flyby (${withFlyby.totalDvKms.toFixed(2)} km/s) should be ≤ direct (${direct.totalDvKms.toFixed(2)} km/s)`);
});

test('bestTrajectory: returns dates in correct order and within window', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 5 * YEAR_DAYS * DAY;
  var traj = ga.bestTrajectory({
    earth: EARTH, flybyBody: VENUS, target: CERES,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(traj);
  assert.ok(traj.launchMs >= startMs && traj.launchMs <= endMs);
  assert.ok(traj.flybyMs > traj.launchMs);
  assert.ok(traj.arriveMs > traj.flybyMs);
});

// --- findBestFlybys -------------------------------------------------------
// Real-world (J2000) elements for the test pool.  Sun has a=0 so the solver
// must special-case it (perihelion-Oberth model) rather than try to compute
// an orbit at zero radius.
const MERCURY = { a: 0.387098, longitude: 252.25, name: 'Mercury' };
const JUPITER = { a: 5.2026,   longitude: 34.40,  name: 'Jupiter' };
const SATURN  = { a: 9.5549,   longitude: 50.08,  name: 'Saturn' };
const URANUS  = { a: 19.2184,  longitude: 314.06, name: 'Uranus' };
const NEPTUNE = { a: 30.1104,  longitude: 304.35, name: 'Neptune' };
const PLUTO   = { a: 39.482,   longitude: 238.93, name: 'Pluto' };
const SUN     = { a: 0,        longitude: 0,      name: 'Sun' };

const POOL = [MERCURY, VENUS, EARTH, MARS, CERES, JUPITER, SATURN, URANUS, NEPTUNE, PLUTO, SUN];

test('findBestFlybys: always returns a direct result even if nothing helps', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 2 * YEAR_DAYS * DAY;
  var res = ga.findBestFlybys({
    from: EARTH, to: MARS, bodies: POOL,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(res, 'returned a result object');
  assert.ok(res.direct, 'direct trajectory present');
  assert.ok(typeof res.direct.totalDvKms === 'number');
  assert.ok(Array.isArray(res.flybys));
});

test('findBestFlybys: Earth → Jupiter (5 yr window) finds a flyby beating direct', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 5 * YEAR_DAYS * DAY;
  var res = ga.findBestFlybys({
    from: EARTH, to: JUPITER, bodies: POOL,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(res && res.direct);
  // Top flyby (if present) should save Δv vs direct.
  if (res.flybys.length > 0) {
    assert.ok(res.flybys[0].savedDv > 0,
      `top flyby (${res.flybys[0].flybyName}) should save Δv, got ${res.flybys[0].savedDv.toFixed(2)} km/s`);
  } else {
    // If we found nothing, that's still a valid outcome — but for Earth →
    // Jupiter on a 5-year window we expect *something* helpful.
    assert.fail('expected at least one viable flyby for Earth → Jupiter');
  }
});

test('findBestFlybys: Earth → Pluto (10 yr window) top-5 includes Jupiter', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 10 * YEAR_DAYS * DAY;
  var res = ga.findBestFlybys({
    from: EARTH, to: PLUTO, bodies: POOL,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(res && res.direct);
  var names = res.flybys.map(function (f) { return f.flybyName; });
  assert.ok(names.indexOf('Jupiter') >= 0,
    `expected Jupiter in top flyby list, got: ${names.join(', ')}`);
});

test('findBestFlybys: Earth → Pluto via Sun returns a non-null result', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 10 * YEAR_DAYS * DAY;
  var res = ga.findBestFlybys({
    from: EARTH, to: PLUTO, bodies: POOL,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(res);
  var sun = res.flybys.find(function (f) { return f.flybyName === 'Sun'; });
  // The Sun flyby may or may not be in the top-5, but bestTrajectory called
  // directly with Sun should produce numerics, not crash.  Test that path:
  var traj = ga.bestTrajectory({
    earth: EARTH, flybyBody: SUN, target: PLUTO,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(traj, 'Sun flyby trajectory should produce a result');
  assert.ok(isFinite(traj.totalDvKms), 'Sun flyby Δv should be finite');
  // If sun happens to be in the top-5 keep us honest:
  if (sun) {
    assert.ok(isFinite(sun.totalDvKms), 'Sun-flyby total Δv finite in ranked list');
  }
});

test('findBestFlybys: Mercury → Venus short window has no helpful flyby', () => {
  var startMs = Date.UTC(2020, 0, 1);
  var endMs = startMs + 1 * YEAR_DAYS * DAY;
  var res = ga.findBestFlybys({
    from: MERCURY, to: VENUS, bodies: POOL,
    fromDateMs: startMs, toDateMs: endMs, epochMs: J2000_MS,
  });
  assert.ok(res && res.direct);
  // Either no flybys, or every flyby has negative savedDv (i.e. worse than direct).
  var helpful = res.flybys.filter(function (f) { return f.savedDv > 0; });
  assert.equal(helpful.length, 0,
    `expected no helpful flyby for Mercury→Venus; got: ${helpful.map(f => f.flybyName + ' (+' + f.savedDv.toFixed(2) + ')').join(', ')}`);
});

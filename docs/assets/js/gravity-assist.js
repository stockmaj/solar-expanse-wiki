// Gravity-assist trajectory calculator — single-flyby, patched-conic,
// circular-coplanar approximation.  Companion to launch-windows.js.
//
// Units: AU for distance, years for time.  Sun's gravitational parameter
//   μ_sun = 4π² (AU³/yr²)   because Kepler's third law gives T² = a³ for AU/yr.
// Heliocentric velocities therefore come out in AU/yr; convert to km/s by
//   1 AU/yr ≈ 4.7404 km/s.
//
// Pure functions on globalThis.GravityAssist; CommonJS export too.

(function (root) {
  var MU = 4 * Math.PI * Math.PI;             // AU^3 / yr^2
  var DAY_MS = 86400000;
  var YEAR_DAYS = 365.25;
  var YEAR_MS = YEAR_DAYS * DAY_MS;
  var TWO_PI = Math.PI * 2;
  var DEG = Math.PI / 180;
  var AU_PER_YR_TO_KM_S = 4.74047;            // 1 AU/yr in km/s

  // --- tiny 2D vector helpers ---------------------------------------------
  function vsub(a, b) { return [a[0] - b[0], a[1] - b[1]]; }
  function vmag(a)    { return Math.sqrt(a[0]*a[0] + a[1]*a[1]); }
  function vdot(a, b) { return a[0]*b[0] + a[1]*b[1]; }
  function vscale(a, s) { return [a[0]*s, a[1]*s]; }
  function vadd(a, b) { return [a[0] + b[0], a[1] + b[1]]; }

  // --- position at date (circular coplanar, same convention as launch-windows.js)
  // body = {a, longitude}  where longitude is degrees at epochMs.
  // Returns {r: [x, y] in AU, v: [vx, vy] in AU/yr} for a circular orbit.
  function positionAt(body, dateMs, epochMs) {
    var daysSinceEpoch = (dateMs - epochMs) / DAY_MS;
    var n = TWO_PI / Math.pow(body.a, 1.5);       // mean motion (rad/yr)
    var theta = body.longitude * DEG + n * (daysSinceEpoch / YEAR_DAYS);
    theta = ((theta % TWO_PI) + TWO_PI) % TWO_PI;
    var r = [body.a * Math.cos(theta), body.a * Math.sin(theta)];
    // For a circular orbit, v = sqrt(μ/a) tangent to the orbit (prograde = +θ̂).
    var vmag_circ = Math.sqrt(MU / body.a);
    var v = [-vmag_circ * Math.sin(theta), vmag_circ * Math.cos(theta)];
    return { r: r, v: v, theta: theta };
  }

  // --- Lambert solver (universal-variable formulation, Bate/Mueller/White) -
  // Given r1, r2 (position vectors), tof (time-of-flight, years), and
  // prograde (boolean), returns {v1, v2} — the velocity vectors at r1 and r2
  // for the transfer arc connecting them.
  //
  // Idea: parameterize the transfer ellipse by `z = (Δν/2)^2 * sign`.  For
  // each guess at z we compute the implied tof via Lagrange's f/g/F/G with
  // Stumpff functions C(z), S(z); Newton-iterate on z until tof matches.
  function stumpffC(z) {
    if (z > 1e-6)  return (1 - Math.cos(Math.sqrt(z))) / z;
    if (z < -1e-6) return (1 - Math.cosh(Math.sqrt(-z))) / z;
    return 0.5 - z/24 + z*z/720;                 // series for z near 0
  }
  function stumpffS(z) {
    if (z > 1e-6) {
      var sz = Math.sqrt(z);
      return (sz - Math.sin(sz)) / (sz*sz*sz);
    }
    if (z < -1e-6) {
      var smz = Math.sqrt(-z);
      return (Math.sinh(smz) - smz) / (smz*smz*smz);
    }
    return 1/6 - z/120 + z*z/5040;
  }

  function lambert(r1, r2, tof, prograde) {
    var r1m = vmag(r1), r2m = vmag(r2);
    var cosDnu = vdot(r1, r2) / (r1m * r2m);
    // 2D cross product (z-component) tells us the direction of motion.
    var crossZ = r1[0]*r2[1] - r1[1]*r2[0];
    var dnu;                                      // transfer angle Δν
    if (prograde) {
      dnu = (crossZ >= 0) ? Math.acos(Math.max(-1, Math.min(1, cosDnu)))
                          : TWO_PI - Math.acos(Math.max(-1, Math.min(1, cosDnu)));
    } else {
      dnu = (crossZ < 0)  ? Math.acos(Math.max(-1, Math.min(1, cosDnu)))
                          : TWO_PI - Math.acos(Math.max(-1, Math.min(1, cosDnu)));
    }
    var sinDnu = Math.sin(dnu);
    if (Math.abs(sinDnu) < 1e-10) return null;    // collinear — plane undefined
    var A = Math.sin(dnu) * Math.sqrt(r1m * r2m / (1 - cosDnu));

    // Newton iteration on z.
    var z = 0;                                    // start near parabolic
    var zLow = -4 * Math.PI * Math.PI;
    var zHigh = 4 * Math.PI * Math.PI;
    var t = 0, y = 0;
    for (var i = 0; i < 30; i++) {
      var C = stumpffC(z);
      var S = stumpffS(z);
      y = r1m + r2m + A * (z * S - 1) / Math.sqrt(C);
      if (A > 0 && y < 0) {
        // Adjust z upward until y > 0 (handles short-way edge cases).
        while (y < 0) { z += 0.1; C = stumpffC(z); S = stumpffS(z); y = r1m + r2m + A * (z*S - 1) / Math.sqrt(C); }
      }
      var x = Math.sqrt(y / C);
      t = (x*x*x * S + A * Math.sqrt(y)) / Math.sqrt(MU);
      if (Math.abs(t - tof) < 1e-8) break;
      // Bisection fallback to keep us bounded.
      if (t < tof) zLow = z; else zHigh = z;
      // Newton step on dt/dz.
      var dtdz;
      if (Math.abs(z) > 1e-6) {
        dtdz = (x*x*x * (S - 1.5*S/z + 0.5*C/z) - 0.375*A*x/Math.sqrt(C) + 0.125*A*(3*S*Math.sqrt(y) + A*Math.sqrt(C/y))) / Math.sqrt(MU);
      } else {
        // Series approximation at z = 0.
        var y0 = r1m + r2m - A * Math.sqrt(2);
        dtdz = (Math.sqrt(2)/40) * Math.pow(y0, 1.5) + (A/8) * (Math.sqrt(y0) + A*Math.sqrt(1/(2*y0)));
        dtdz = dtdz / Math.sqrt(MU);
      }
      var zNext = z - (t - tof) / dtdz;
      // Clamp to bracket.
      if (zNext < zLow || zNext > zHigh || !isFinite(zNext)) {
        zNext = 0.5 * (zLow + zHigh);
      }
      z = zNext;
    }
    if (!isFinite(z) || !isFinite(y) || y <= 0) return null;

    var Cf = stumpffC(z);
    var f = 1 - y / r1m;
    var g = A * Math.sqrt(y / MU);
    var gdot = 1 - y / r2m;
    if (Math.abs(g) < 1e-12) return null;
    // v1 = (r2 - f*r1) / g ;  v2 = (gdot*r2 - r1) / g
    var v1 = vscale(vsub(r2, vscale(r1, f)), 1/g);
    var v2 = vscale(vsub(vscale(r2, gdot), r1), 1/g);
    return { v1: v1, v2: v2 };
  }

  // --- Single-flyby trajectory optimizer ----------------------------------
  // Patched-conic, free-rotation flyby model: the gravity assist can bend
  // v_inf by any angle (no mismatch penalty), so the user pays only
  // |v_at_launch - v_earth| + |v_at_arrival - v_target|.  This is the
  // "best-case" cost — useful for ranking flyby options.
  function bestTrajectory(args) {
    var earth = args.earth;
    var flyby = args.flybyBody;
    var target = args.target;
    var fromMs = args.fromDateMs;
    var toMs = args.toDateMs;
    var epoch = args.epochMs;

    // Coarse grid: launch dates every 15 days, flyby dates spanning a
    // reasonable range.  Total grid stays small (<3500 evaluations).
    var LAUNCH_STEP_DAYS = 15;
    var FLYBY_STEP_DAYS = 15;
    // First-leg time: bracket the Hohmann time between Earth and flyby body
    // by [0.4×, 1.8×] to allow shorter/longer-than-Hohmann arcs.
    var hohmann1 = 0.5 * Math.pow((earth.a + flyby.a) / 2, 1.5) * YEAR_DAYS;
    var leg1Min = Math.max(40, hohmann1 * 0.4);
    var leg1Max = hohmann1 * 1.8;
    var hohmann2 = 0.5 * Math.pow((flyby.a + target.a) / 2, 1.5) * YEAR_DAYS;
    var leg2Min = Math.max(60, hohmann2 * 0.4);
    var leg2Max = hohmann2 * 1.8;

    var best = null;
    for (var lMs = fromMs; lMs <= toMs; lMs += LAUNCH_STEP_DAYS * DAY_MS) {
      var earthPos = positionAt(earth, lMs, epoch);
      for (var leg1 = leg1Min; leg1 <= leg1Max; leg1 += FLYBY_STEP_DAYS) {
        var fMs = lMs + leg1 * DAY_MS;
        var flybyPos = positionAt(flyby, fMs, epoch);
        var lam1 = lambert(earthPos.r, flybyPos.r, leg1 / YEAR_DAYS, true);
        if (!lam1) continue;
        var vInfLaunch = vmag(vsub(lam1.v1, earthPos.v));
        for (var leg2 = leg2Min; leg2 <= leg2Max; leg2 += FLYBY_STEP_DAYS) {
          var aMs = fMs + leg2 * DAY_MS;
          var targetPos = positionAt(target, aMs, epoch);
          var lam2 = lambert(flybyPos.r, targetPos.r, leg2 / YEAR_DAYS, true);
          if (!lam2) continue;
          var vInfArrive = vmag(vsub(lam2.v2, targetPos.v));
          // Free-rotation flyby: no mismatch term.  Cost = launch C3 +
          // arrival v_inf (in AU/yr; converted for the report).
          var cost = vInfLaunch + vInfArrive;
          if (!best || cost < best.cost) {
            best = {
              cost: cost,
              launchMs: lMs,
              flybyMs: fMs,
              arriveMs: aMs,
              vInfLaunch: vInfLaunch,
              vInfArrive: vInfArrive,
              vInfLaunchKms: vInfLaunch * AU_PER_YR_TO_KM_S,
              vInfArriveKms: vInfArrive * AU_PER_YR_TO_KM_S,
              totalDvKms: cost * AU_PER_YR_TO_KM_S,
            };
          }
        }
      }
    }
    return best;
  }

  // --- Direct (no-flyby) cost, for sanity comparisons ---------------------
  function bestDirect(args) {
    var earth = args.earth, target = args.target;
    var fromMs = args.fromDateMs, toMs = args.toDateMs, epoch = args.epochMs;
    var LAUNCH_STEP_DAYS = 15, TOF_STEP_DAYS = 15;
    var hohmann = 0.5 * Math.pow((earth.a + target.a) / 2, 1.5) * YEAR_DAYS;
    var tofMin = Math.max(60, hohmann * 0.5), tofMax = hohmann * 1.6;
    var best = null;
    for (var lMs = fromMs; lMs <= toMs; lMs += LAUNCH_STEP_DAYS * DAY_MS) {
      var ep = positionAt(earth, lMs, epoch);
      for (var tof = tofMin; tof <= tofMax; tof += TOF_STEP_DAYS) {
        var aMs = lMs + tof * DAY_MS;
        var tp = positionAt(target, aMs, epoch);
        var lam = lambert(ep.r, tp.r, tof / YEAR_DAYS, true);
        if (!lam) continue;
        var c = vmag(vsub(lam.v1, ep.v)) + vmag(vsub(lam.v2, tp.v));
        if (!best || c < best.cost) {
          best = { cost: c, launchMs: lMs, arriveMs: aMs,
                   totalDvKms: c * AU_PER_YR_TO_KM_S };
        }
      }
    }
    return best;
  }

  // --- DOM binding --------------------------------------------------------
  function fmtDate(ms) { return new Date(ms).toISOString().slice(0, 10); }

  function bindDom() {
    var bodies = root.LAUNCH_WINDOW_ALL_BODIES;
    if (!bodies) return;
    var fromInput = document.getElementById('ga-from');
    var flybyInput = document.getElementById('ga-flyby');
    var toInput = document.getElementById('ga-to');
    var dateInput = document.getElementById('ga-date');
    var submitBtn = document.getElementById('ga-submit');
    var resultBox = document.getElementById('ga-result');
    if (!fromInput || !flybyInput || !toInput || !dateInput || !submitBtn || !resultBox) return;

    // Use the same shared datalist as the launch-window calculator
    // (`calc-bodies`).  If the page only has the GA calculator, build a
    // private alphabetical datalist as a fallback.
    if (!document.getElementById('calc-bodies')) {
      var dl = document.createElement('datalist');
      dl.id = 'calc-bodies';
      bodies.slice().sort(function (a, b) {
        return a.name.localeCompare(b.name);
      }).forEach(function (b) {
        var o = document.createElement('option');
        o.value = b.name;
        dl.appendChild(o);
      });
      document.body.appendChild(dl);
    }
    [fromInput, flybyInput, toInput].forEach(function (inp) {
      inp.setAttribute('list', 'calc-bodies');
    });

    function findBody(name) {
      if (!name) return null;
      var needle = name.trim().toLowerCase();
      for (var i = 0; i < bodies.length; i++) {
        if (bodies[i].name.toLowerCase() === needle) return bodies[i];
      }
      return null;
    }

    function update() {
      var v = dateInput.value;
      if (!v) return;
      var startMs = new Date(v + 'T00:00:00Z').getTime();
      if (isNaN(startMs)) return;
      var from = findBody(fromInput.value);
      var flyby = findBody(flybyInput.value);
      var target = findBody(toInput.value);
      if (!from || !flyby || !target) {
        resultBox.innerHTML = '<em>Pick valid From, Flyby, and To bodies from the suggestions.</em>';
        return;
      }
      var epoch = Date.UTC(1959, 0, 1);
      var endMs = startMs + 5 * YEAR_MS;

      resultBox.innerHTML = '<em>Searching…</em>';
      // Defer to next tick so the "searching" message paints first.
      setTimeout(function () {
        var ga = bestTrajectory({
          earth: from, flybyBody: flyby, target: target,
          fromDateMs: startMs, toDateMs: endMs, epochMs: epoch,
        });
        var direct = bestDirect({
          earth: from, target: target,
          fromDateMs: startMs, toDateMs: endMs, epochMs: epoch,
        });
        if (!ga) {
          resultBox.innerHTML = '<em>No trajectory found in window.</em>';
          return;
        }
        var html = '<p><strong>' + from.name + ' → ' + flyby.name +
          ' → ' + target.name + '</strong></p>' +
          '<table><tbody>' +
          '<tr><td>Launch</td><td>' + fmtDate(ga.launchMs) + '</td></tr>' +
          '<tr><td>Flyby (' + flyby.name + ')</td><td>' + fmtDate(ga.flybyMs) + '</td></tr>' +
          '<tr><td>Arrival</td><td>' + fmtDate(ga.arriveMs) + '</td></tr>' +
          '<tr><td>v∞ at launch</td><td>' + ga.vInfLaunchKms.toFixed(2) + ' km/s</td></tr>' +
          '<tr><td>v∞ at arrival</td><td>' + ga.vInfArriveKms.toFixed(2) + ' km/s</td></tr>' +
          '<tr><td><strong>Total Δv proxy</strong></td><td><strong>' +
            ga.totalDvKms.toFixed(2) + ' km/s</strong></td></tr>' +
          '</tbody></table>';
        if (direct) {
          var saved = direct.totalDvKms - ga.totalDvKms;
          html += '<p>Direct ' + from.name + ' → ' + target.name +
            ' Δv proxy: <strong>' + direct.totalDvKms.toFixed(2) +
            ' km/s</strong> (' + (saved >= 0 ? 'saves ' : 'costs extra ') +
            Math.abs(saved).toFixed(2) + ' km/s vs. flyby).</p>';
        }
        resultBox.innerHTML = html;
      }, 0);
    }

    // Grid search is ~200 ms, so don't auto-fire on every keystroke.
    // Submit button only.  Enter inside any field also triggers it.
    submitBtn.addEventListener('click', update);
    [fromInput, flybyInput, toInput, dateInput].forEach(function (inp) {
      inp.addEventListener('keydown', function (e) {
        if (e.key === 'Enter') { e.preventDefault(); update(); }
      });
    });
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', bindDom);
  }

  root.GravityAssist = {
    lambert: lambert,
    positionAt: positionAt,
    bestTrajectory: bestTrajectory,
    bestDirect: bestDirect,
    MU: MU,
    AU_PER_YR_TO_KM_S: AU_PER_YR_TO_KM_S,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.GravityAssist;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

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
  // Special case: a body with a = 0 (e.g. the Sun itself) sits at the origin
  // with zero velocity — the bestTrajectory path special-cases this so the
  // Lambert legs use a near-Sun perihelion point instead of the origin.
  function positionAt(body, dateMs, epochMs) {
    if (!body.a || body.a <= 0) {
      return { r: [0, 0], v: [0, 0], theta: 0 };
    }
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
    var isSunFlyby = !flyby.a || flyby.a <= 0;

    // Coarse grid: launch dates every 15 days, flyby dates spanning a
    // reasonable range.  Step sizes scale up with leg length so outer-planet
    // routes don't blow up — Earth → Jupiter → Neptune would otherwise have
    // a ~1260-step leg-2 dimension and ~28 M total grid points.  We cap the
    // grid to ~200 steps per dimension.
    var LAUNCH_STEP_DAYS = 15;
    var MAX_STEPS_PER_DIM = 200;
    // For a Sun flyby (a=0) we model the spacecraft dipping to a small
    // perihelion ~0.2 AU between launch and target — close enough for an
    // Oberth boost but far enough that the Lambert solver doesn't fight a
    // near-singular geometry.
    var SUN_PERIHELION_AU = 0.2;
    // First-leg time: bracket the Hohmann time between Earth and flyby body
    // by [0.4×, 1.8×] to allow shorter/longer-than-Hohmann arcs.
    var flybyA = isSunFlyby ? SUN_PERIHELION_AU : flyby.a;
    var hohmann1 = 0.5 * Math.pow((earth.a + flybyA) / 2, 1.5) * YEAR_DAYS;
    var leg1Min = Math.max(40, hohmann1 * 0.4);
    var leg1Max = hohmann1 * 1.8;
    var leg1Step = Math.max(15, Math.ceil((leg1Max - leg1Min) / MAX_STEPS_PER_DIM));
    var hohmann2 = 0.5 * Math.pow((flybyA + target.a) / 2, 1.5) * YEAR_DAYS;
    var leg2Min = Math.max(60, hohmann2 * 0.4);
    var leg2Max = hohmann2 * 1.8;
    var leg2Step = Math.max(15, Math.ceil((leg2Max - leg2Min) / MAX_STEPS_PER_DIM));

    var best = null;
    for (var lMs = fromMs; lMs <= toMs; lMs += LAUNCH_STEP_DAYS * DAY_MS) {
      var earthPos = positionAt(earth, lMs, epoch);
      for (var leg1 = leg1Min; leg1 <= leg1Max; leg1 += leg1Step) {
        var fMs = lMs + leg1 * DAY_MS;
        // Sun flyby: perihelion is on the chord between launch and target
        // at flyby time.  We position the "Sun flyby point" at
        // SUN_PERIHELION_AU along the bisector between earthPos and
        // targetPos-at-flyby-time so both legs are short, low-eccentricity
        // arcs.  The Sun's gravity is the assist body, so v_sun = 0.
        var flybyPos;
        if (isSunFlyby) {
          // Anticipate roughly where target will be at end of leg-2 to set
          // a sensible perihelion direction.  Use Hohmann midpoint as proxy.
          var midMs = fMs + (leg2Min + leg2Max) / 2 * DAY_MS;
          var tposPreview = positionAt(target, midMs, epoch);
          // Direction = average of unit(earthPos.r) and unit(target.r) at
          // flyby time; if those are opposite we just take launch direction.
          var ex = earthPos.r[0], ey = earthPos.r[1];
          var em = Math.sqrt(ex*ex + ey*ey) || 1;
          var tx = tposPreview.r[0], ty = tposPreview.r[1];
          var tm = Math.sqrt(tx*tx + ty*ty) || 1;
          var dx = ex / em + tx / tm;
          var dy = ey / em + ty / tm;
          var dm = Math.sqrt(dx*dx + dy*dy);
          if (dm < 1e-6) { dx = ex / em; dy = ey / em; dm = 1; }
          flybyPos = {
            r: [SUN_PERIHELION_AU * dx / dm, SUN_PERIHELION_AU * dy / dm],
            v: [0, 0],
            theta: 0,
          };
        } else {
          flybyPos = positionAt(flyby, fMs, epoch);
        }
        var lam1 = lambert(earthPos.r, flybyPos.r, leg1 / YEAR_DAYS, true);
        if (!lam1) continue;
        var vInfLaunch = vmag(vsub(lam1.v1, earthPos.v));
        for (var leg2 = leg2Min; leg2 <= leg2Max; leg2 += leg2Step) {
          var aMs = fMs + leg2 * DAY_MS;
          var targetPos = positionAt(target, aMs, epoch);
          var lam2 = lambert(flybyPos.r, targetPos.r, leg2 / YEAR_DAYS, true);
          if (!lam2) continue;
          var vInfArrive = vmag(vsub(lam2.v2, targetPos.v));
          // Free-rotation flyby: no mismatch term.  Cost = launch C3 +
          // arrival v_inf (in AU/yr; converted for the report).
          var cost = vInfLaunch + vInfArrive;
          if (!isFinite(cost)) continue;
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

  // --- Notes describing a candidate flyby body ----------------------------
  // Used in the rendered results table.  Sun gets its own Oberth-flavoured
  // note; everything else gets a generic body-type tag.
  function flybyNote(body) {
    if (!body) return '';
    if (!body.a || body.a <= 0) return 'Solar Oberth maneuver — deep dip past the Sun';
    if (body.a < 1) return 'Inner-system flyby';
    if (body.a < 5) return 'Mid-system flyby';
    if (body.a < 30) return 'Outer-planet flyby';
    return 'Deep outer-system flyby';
  }

  // --- Multi-flyby search: scan every candidate body, rank top 5 ----------
  // Synchronous variant for tests; the DOM driver wraps this in a setTimeout
  // chain so the page stays responsive while ~60 bodies are scanned.
  function findBestFlybys(args) {
    var from = args.from, to = args.to;
    var bodies = args.bodies || [];
    var fromMs = args.fromDateMs, toMs = args.toDateMs, epoch = args.epochMs;
    var topN = args.topN || 5;

    var direct = bestDirect({
      earth: from, target: to,
      fromDateMs: fromMs, toDateMs: toMs, epochMs: epoch,
    });

    var rows = [];
    for (var i = 0; i < bodies.length; i++) {
      var b = bodies[i];
      if (!b) continue;
      if (b === from || b === to) continue;
      if (b.name === from.name || b.name === to.name) continue;
      var ga = bestTrajectory({
        earth: from, flybyBody: b, target: to,
        fromDateMs: fromMs, toDateMs: toMs, epochMs: epoch,
      });
      if (!ga) continue;
      if (!isFinite(ga.totalDvKms)) continue;
      var saved = direct ? direct.totalDvKms - ga.totalDvKms : 0;
      rows.push({
        flybyName: b.name,
        flybyBody: b,
        launchMs: ga.launchMs,
        flybyMs: ga.flybyMs,
        arriveMs: ga.arriveMs,
        vInfLaunchKms: ga.vInfLaunchKms,
        vInfArriveKms: ga.vInfArriveKms,
        totalDvKms: ga.totalDvKms,
        savedDv: saved,
        note: flybyNote(b),
      });
    }
    // Sort by saved Δv descending and trim to topN entries that beat direct.
    rows.sort(function (a, b) { return b.savedDv - a.savedDv; });
    var helpful = rows.filter(function (r) { return r.savedDv > 0; }).slice(0, topN);
    return { direct: direct, flybys: helpful };
  }

  // --- DOM binding --------------------------------------------------------
  function fmtDate(ms) { return new Date(ms).toISOString().slice(0, 10); }

  function bindDom() {
    var bodies = root.LAUNCH_WINDOW_ALL_BODIES;
    if (!bodies) return;
    var fromInput = document.getElementById('ga-from');
    var toInput = document.getElementById('ga-to');
    var dateInput = document.getElementById('ga-date');
    var submitBtn = document.getElementById('ga-submit');
    var resultBox = document.getElementById('ga-result');
    if (!fromInput || !toInput || !dateInput || !submitBtn || !resultBox) return;

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
    [fromInput, toInput].forEach(function (inp) {
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

    function renderResults(from, to, res) {
      var html = '<table><thead><tr>' +
        '<th>Route</th>' +
        '<th>Launch</th>' +
        '<th>Arrival</th>' +
        '<th>v∞ launch</th>' +
        '<th>v∞ arrival</th>' +
        '<th>Total Δv proxy</th>' +
        '<th>Δv saved</th>' +
        '<th>Notes</th>' +
        '</tr></thead><tbody>';
      if (res.direct) {
        html += '<tr>' +
          '<td><strong>' + from.name + ' → ' + to.name + '</strong> (direct)</td>' +
          '<td>' + fmtDate(res.direct.launchMs) + '</td>' +
          '<td>' + fmtDate(res.direct.arriveMs) + '</td>' +
          '<td>—</td>' +
          '<td>—</td>' +
          '<td>' + res.direct.totalDvKms.toFixed(2) + ' km/s</td>' +
          '<td>—</td>' +
          '<td>baseline (no flyby)</td>' +
          '</tr>';
      } else {
        html += '<tr><td colspan="8"><em>No direct trajectory found in window.</em></td></tr>';
      }
      res.flybys.forEach(function (r) {
        html += '<tr>' +
          '<td><strong>' + from.name + ' → ' + r.flybyName + ' → ' + to.name + '</strong></td>' +
          '<td>' + fmtDate(r.launchMs) + '</td>' +
          '<td>' + fmtDate(r.arriveMs) + '</td>' +
          '<td>' + r.vInfLaunchKms.toFixed(2) + ' km/s</td>' +
          '<td>' + r.vInfArriveKms.toFixed(2) + ' km/s</td>' +
          '<td>' + r.totalDvKms.toFixed(2) + ' km/s</td>' +
          '<td style="color:#7fd17f"><strong>+' + r.savedDv.toFixed(2) + ' km/s</strong></td>' +
          '<td>' + r.note + '</td>' +
          '</tr>';
      });
      if (res.flybys.length === 0) {
        html += '<tr><td colspan="8"><em>No gravity-assist option beats direct in this window.</em></td></tr>';
      }
      html += '</tbody></table>';
      resultBox.innerHTML = html;
    }

    // Async scan: ~60 bodies × ~50 ms each ≈ 3 s.  We yield with
    // setTimeout(0) between bodies so the page stays responsive and we can
    // stream a progress count into the result box.
    function update() {
      var v = dateInput.value;
      if (!v) return;
      var startMs = new Date(v + 'T00:00:00Z').getTime();
      if (isNaN(startMs)) return;
      var from = findBody(fromInput.value);
      var target = findBody(toInput.value);
      if (!from || !target) {
        resultBox.innerHTML = '<em>Pick valid From and To bodies from the suggestions.</em>';
        return;
      }
      if (from === target) {
        resultBox.innerHTML = '<em>From and To must be different bodies.</em>';
        return;
      }
      var epoch = Date.UTC(1959, 0, 1);
      var endMs = startMs + 10 * YEAR_MS;

      submitBtn.disabled = true;
      submitBtn.textContent = 'Calculating…';

      var direct = null;
      var rows = [];
      var idx = 0;

      function done() {
        rows.sort(function (a, b) { return b.savedDv - a.savedDv; });
        var helpful = rows.filter(function (r) { return r.savedDv > 0; }).slice(0, 5);
        renderResults(from, target, { direct: direct, flybys: helpful });
        submitBtn.disabled = false;
        submitBtn.textContent = 'Recalculate';
      }

      // First compute direct, then stream candidate flybys.
      resultBox.innerHTML = '<em>Computing direct trajectory…</em>';
      setTimeout(function () {
        direct = bestDirect({
          earth: from, target: target,
          fromDateMs: startMs, toDateMs: endMs, epochMs: epoch,
        });
        function step() {
          if (idx >= bodies.length) { done(); return; }
          var b = bodies[idx++];
          if (!b || b === from || b === target ||
              b.name === from.name || b.name === target.name) {
            resultBox.innerHTML = '<em>Scanning flyby candidates… ' +
              idx + ' / ' + bodies.length + '</em>';
            setTimeout(step, 0);
            return;
          }
          var ga = bestTrajectory({
            earth: from, flybyBody: b, target: target,
            fromDateMs: startMs, toDateMs: endMs, epochMs: epoch,
          });
          if (ga && isFinite(ga.totalDvKms)) {
            rows.push({
              flybyName: b.name,
              flybyBody: b,
              launchMs: ga.launchMs,
              flybyMs: ga.flybyMs,
              arriveMs: ga.arriveMs,
              vInfLaunchKms: ga.vInfLaunchKms,
              vInfArriveKms: ga.vInfArriveKms,
              totalDvKms: ga.totalDvKms,
              savedDv: direct ? direct.totalDvKms - ga.totalDvKms : 0,
              note: flybyNote(b),
            });
          }
          resultBox.innerHTML = '<em>Scanning flyby candidates… ' +
            idx + ' / ' + bodies.length + '</em>';
          setTimeout(step, 0);
        }
        step();
      }, 0);
    }

    submitBtn.addEventListener('click', update);
    [fromInput, toInput, dateInput].forEach(function (inp) {
      inp.addEventListener('keydown', function (e) {
        if (e.key === 'Enter') { e.preventDefault(); update(); }
      });
    });
    // Select-all on focus so the user can replace the default body name
    // with a single keystroke instead of backspacing the old value out.
    [fromInput, toInput].forEach(function (inp) {
      inp.addEventListener('focus', function () {
        setTimeout(function () { inp.select(); }, 0);
      });
    });
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', function () {
      bindDom();
    });
  }

  root.GravityAssist = {
    lambert: lambert,
    positionAt: positionAt,
    bestTrajectory: bestTrajectory,
    bestDirect: bestDirect,
    findBestFlybys: findBestFlybys,
    MU: MU,
    AU_PER_YR_TO_KM_S: AU_PER_YR_TO_KM_S,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.GravityAssist;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

// Launch-window calculator for the Earth ↔ planet pairs.
//
// Given a start date, compute the next five Earth-departure Hohmann-style
// launch windows for each other planet.  Math:
//
//   T_years  = a_AU^1.5                          (Kepler's third)
//   n        = 2π / T_years                       (mean motion)
//   t_transfer = 0.5 * ((a_earth + a_body) / 2)^1.5  (Hohmann half-orbit, years)
//   θ_required = π - n_body * t_transfer          (relative lead/lag at launch)
//   Δθ_current = θ_body(t) - θ_earth(t)
//   wait_years = ((θ_required - Δθ_current) mod 2π) / (n_earth - n_body)
//
// The body's longitude at the epoch the game ships in its orbital data is
// our anchor; we assume that epoch is 1959-01-01 (the game's earliest
// playable start), which matches the contract.dateStartActive baseline.
// The *spacing* between windows is reliable regardless of epoch choice; the
// *absolute dates* may drift by days to weeks vs. the in-game porkchop plot.

(function () {
  if (!window.LAUNCH_WINDOW_BODIES || !window.LAUNCH_WINDOW_EARTH) return;
  var EPOCH_MS = Date.UTC(1959, 0, 1);  // 1959-01-01 UTC
  var DAY_MS = 86400000;
  var YEAR_DAYS = 365.25;
  var TWO_PI = Math.PI * 2;
  var DEG = Math.PI / 180;

  var earth = window.LAUNCH_WINDOW_EARTH;
  var n_earth = TWO_PI / Math.pow(earth.a, 1.5);

  function angleAt(longitudeDeg, n, daysSinceEpoch) {
    var theta = longitudeDeg * DEG + n * (daysSinceEpoch / YEAR_DAYS);
    return ((theta % TWO_PI) + TWO_PI) % TWO_PI;
  }

  function nextWindow(body, fromDate) {
    // A "launch window" here = the time when an idealized minimum-energy
    // Hohmann transfer launched from Earth's orbit would arrive at the target
    // body just as the body reaches the transfer's aphelion (for outer) or
    // perihelion (for inner).  In phase-space terms: the relative angle
    // (theta_body - theta_earth) at launch must equal `required` below.
    var daysSinceEpoch = (fromDate.getTime() - EPOCH_MS) / DAY_MS;
    var n_body = TWO_PI / Math.pow(body.a, 1.5);
    var theta_earth = angleAt(earth.longitude, n_earth, daysSinceEpoch);
    var theta_body = angleAt(body.longitude, n_body, daysSinceEpoch);
    var rel = ((theta_body - theta_earth) % TWO_PI + TWO_PI) % TWO_PI;
    var t_transfer_years = 0.5 * Math.pow((earth.a + body.a) / 2, 1.5);
    var required = Math.PI - n_body * t_transfer_years;
    required = ((required % TWO_PI) + TWO_PI) % TWO_PI;
    // rel changes at rate R = n_body - n_earth per year.  For outer bodies
    // R < 0 (Earth laps the target); for inner bodies R > 0.  Solve
    //    rel + R * t  ≡  required  (mod 2π)
    // for smallest positive t.  This is equivalent to t = (rel - required) /
    // (n_earth - n_body), with wrap-around to keep t in [0, synodic).
    var omega = n_earth - n_body;
    var synodic_years = TWO_PI / Math.abs(omega);
    var delta = (rel - required) / omega;
    while (delta < 0) delta += synodic_years;
    while (delta >= synodic_years) delta -= synodic_years;
    var waitDays = delta * YEAR_DAYS;
    return new Date(fromDate.getTime() + waitDays * DAY_MS);
  }

  function fmtDate(d) {
    if (isNaN(d)) return '—';
    return d.toISOString().slice(0, 10);
  }

  function update() {
    var input = document.getElementById('calc-date');
    var tbody = document.querySelector('#calc-result tbody');
    if (!input || !tbody) return;
    var v = input.value;
    if (!v) return;
    var start = new Date(v + 'T00:00:00Z');
    if (isNaN(start)) return;
    tbody.innerHTML = '';
    window.LAUNCH_WINDOW_BODIES.forEach(function (body) {
      var tr = document.createElement('tr');
      var nameTd = document.createElement('td');
      nameTd.innerHTML = '<strong>' + body.name + '</strong>';
      tr.appendChild(nameTd);
      var cursor = start;
      for (var i = 0; i < 5; i++) {
        var win = nextWindow(body, cursor);
        var td = document.createElement('td');
        td.textContent = fmtDate(win);
        tr.appendChild(td);
        // Step past this window by 1 day so the same one isn't returned twice.
        cursor = new Date(win.getTime() + DAY_MS);
      }
      tbody.appendChild(tr);
    });
  }

  document.addEventListener('DOMContentLoaded', function () {
    var input = document.getElementById('calc-date');
    if (input) input.addEventListener('change', update);
    update();
  });
})();

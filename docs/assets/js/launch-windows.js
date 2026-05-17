// Launch-window calculator for arbitrary body-to-body transfers.
//
// Pure logic + DOM binding in the same file.  The pure functions are exposed
// on `globalThis.LaunchWindows` so the Node test harness can import them
// without a DOM.

(function (root) {
  // Default epoch is the game's earliest contract baseline (1959-01-01).
  // Override via the `epochMs` param to nextWindow() / nextNWindows() when
  // testing against real-world J2000-anchored data.
  var EPOCH_MS = Date.UTC(1959, 0, 1);
  var DAY_MS = 86400000;
  var YEAR_DAYS = 365.25;
  var TWO_PI = Math.PI * 2;
  var DEG = Math.PI / 180;

  function angleAt(longitudeDeg, n, daysSinceEpoch) {
    var theta = longitudeDeg * DEG + n * (daysSinceEpoch / YEAR_DAYS);
    return ((theta % TWO_PI) + TWO_PI) % TWO_PI;
  }

  function hohmannTransferYears(a_from, a_to) {
    return 0.5 * Math.pow((a_from + a_to) / 2, 1.5);
  }

  // Given (from, to, fromDate), return the next Hohmann-transfer launch
  // window for from → to.  Bodies are {a, longitude} objects in AU and
  // degrees-at-EPOCH respectively.
  function nextWindow(from, to, fromDate, epochMs) {
    if (Math.abs(from.a - to.a) < 1e-9) return null;
    var epoch = (typeof epochMs === 'number') ? epochMs : EPOCH_MS;
    var daysSinceEpoch = (fromDate.getTime() - epoch) / DAY_MS;
    var n_from = TWO_PI / Math.pow(from.a, 1.5);
    var n_to = TWO_PI / Math.pow(to.a, 1.5);
    var theta_from = angleAt(from.longitude, n_from, daysSinceEpoch);
    var theta_to = angleAt(to.longitude, n_to, daysSinceEpoch);
    var rel = ((theta_to - theta_from) % TWO_PI + TWO_PI) % TWO_PI;
    var t_transfer_years = hohmannTransferYears(from.a, to.a);
    // Required relative angle at launch so that target reaches transfer
    // far-side just as the spacecraft does.
    var required = Math.PI - n_to * t_transfer_years;
    required = ((required % TWO_PI) + TWO_PI) % TWO_PI;
    // rel changes at rate (n_to - n_from) per year; solve for next positive t.
    var omega = n_from - n_to;
    if (Math.abs(omega) < 1e-12) return null;
    var synodic_years = TWO_PI / Math.abs(omega);
    var delta = (rel - required) / omega;
    while (delta < 0) delta += synodic_years;
    while (delta >= synodic_years) delta -= synodic_years;
    return new Date(fromDate.getTime() + delta * YEAR_DAYS * DAY_MS);
  }

  function nextNWindows(from, to, fromDate, n, epochMs) {
    var out = [];
    var cursor = fromDate;
    for (var i = 0; i < n; i++) {
      var w = nextWindow(from, to, cursor, epochMs);
      if (!w) break;
      out.push(w);
      cursor = new Date(w.getTime() + DAY_MS);
    }
    return out;
  }

  function fmtDate(d) {
    if (!d || isNaN(d)) return '—';
    return d.toISOString().slice(0, 10);
  }

  // ----- DOM binding -----------------------------------------------------

  function bindDom() {
    var bodies = root.LAUNCH_WINDOW_ALL_BODIES;
    var earth = root.LAUNCH_WINDOW_EARTH;
    if (!bodies || !earth) return;

    var dateInput = document.getElementById('calc-date');
    var fromSelect = document.getElementById('calc-from');
    var toSelect = document.getElementById('calc-to');
    var resultBox = document.getElementById('calc-result');
    if (!dateInput || !fromSelect || !toSelect || !resultBox) return;

    // Populate dropdowns sorted by semi-major axis.
    var sorted = bodies.slice().sort(function (a, b) { return a.a - b.a; });
    sorted.forEach(function (b) {
      [fromSelect, toSelect].forEach(function (sel) {
        var opt = document.createElement('option');
        opt.value = b.name;
        opt.textContent = b.name;
        sel.appendChild(opt);
      });
    });
    fromSelect.value = 'Earth';
    toSelect.value = 'Mars';

    function findBody(name) {
      for (var i = 0; i < bodies.length; i++) {
        if (bodies[i].name === name) return bodies[i];
      }
      return null;
    }

    function update() {
      var v = dateInput.value;
      if (!v) return;
      var start = new Date(v + 'T00:00:00Z');
      if (isNaN(start)) return;
      var from = findBody(fromSelect.value);
      var to = findBody(toSelect.value);
      if (!from || !to) return;
      var windows = nextNWindows(from, to, start, 5);
      if (windows.length === 0) {
        resultBox.innerHTML = '<em>No transfer between identical orbits.</em>';
        return;
      }
      var transfer_days = hohmannTransferYears(from.a, to.a) * YEAR_DAYS;
      resultBox.innerHTML =
        '<p><strong>' + from.name + ' → ' + to.name + '</strong> &middot; ' +
        'transfer time ≈ ' + Math.round(transfer_days) + ' days</p>' +
        '<table><thead><tr><th>#</th><th>Launch date</th><th>Arrival date</th></tr></thead><tbody>' +
        windows.map(function (w, i) {
          var arr = new Date(w.getTime() + transfer_days * DAY_MS);
          return '<tr><td>' + (i + 1) + '</td><td>' + fmtDate(w) +
                 '</td><td>' + fmtDate(arr) + '</td></tr>';
        }).join('') +
        '</tbody></table>';
    }

    dateInput.addEventListener('change', update);
    fromSelect.addEventListener('change', update);
    toSelect.addEventListener('change', update);
    update();
  }

  // Body-name filter on the big synodic table.
  function bindFilter() {
    var input = document.getElementById('body-filter');
    var table = document.querySelector('#body-table table');
    if (!input || !table) return;
    input.addEventListener('input', function () {
      var q = input.value.trim().toLowerCase();
      table.querySelectorAll('tbody tr').forEach(function (tr) {
        var first = tr.cells[0] ? (tr.cells[0].textContent || '').toLowerCase() : '';
        tr.style.display = !q || first.indexOf(q) !== -1 ? '' : 'none';
      });
    });
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', function () {
      bindDom();
      bindFilter();
    });
  }

  // Expose pure functions for tests / external use.
  root.LaunchWindows = {
    nextWindow: nextWindow,
    nextNWindows: nextNWindows,
    hohmannTransferYears: hohmannTransferYears,
    angleAt: angleAt,
    EPOCH_MS: EPOCH_MS,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.LaunchWindows;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

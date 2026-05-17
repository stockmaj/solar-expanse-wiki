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
    var fromInput = document.getElementById('calc-from');
    var toInput = document.getElementById('calc-to');
    var datalist = document.getElementById('calc-bodies');
    var resultBox = document.getElementById('calc-result');
    if (!dateInput || !fromInput || !toInput || !datalist || !resultBox) return;

    // Populate the shared datalist alphabetically.  Browsers (Chrome,
    // Firefox, Safari) substring-match against the input value, so the
    // user can type "mar" and see Mars / Mars-orbit asteroids in the
    // suggestion popup.
    var sorted = bodies.slice().sort(function (a, b) {
      return a.name.localeCompare(b.name);
    });
    sorted.forEach(function (b) {
      var opt = document.createElement('option');
      opt.value = b.name;
      datalist.appendChild(opt);
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
      var start = new Date(v + 'T00:00:00Z');
      if (isNaN(start)) return;
      var from = findBody(fromInput.value);
      var to = findBody(toInput.value);
      if (!from || !to) {
        resultBox.innerHTML = '<em>Pick valid From and To bodies from the suggestions.</em>';
        return;
      }
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

    // Auto-update is cheap (~5 trig evaluations).  Listen to both `input`
    // (fires on each keystroke) and `change` (fires when the user picks
    // from the datalist popup, in browsers that don't fire `input` then).
    ['input', 'change'].forEach(function (ev) {
      fromInput.addEventListener(ev, update);
      toInput.addEventListener(ev, update);
    });
    dateInput.addEventListener('change', update);
    update();
  }

  // Combined text + type-checkbox filter for the body table.  The Type
  // column (2nd column) holds "Planet" / "Asteroid" / "Comet"; checkboxes
  // toggle each.  Default state is "Planets only" — see the markup in the
  // generator.
  function bindFilter() {
    var input = document.getElementById('body-filter');
    var table = document.querySelector('#body-table table');
    var checkboxes = document.querySelectorAll('.body-type-filter');
    if (!input || !table) return;

    function applyFilter() {
      var q = input.value.trim().toLowerCase();
      var enabled = {};
      checkboxes.forEach(function (cb) {
        if (cb.checked) enabled[cb.value] = true;
      });
      table.querySelectorAll('tbody tr').forEach(function (tr) {
        var nameCell = tr.cells[0] ? (tr.cells[0].textContent || '').toLowerCase() : '';
        var typeCell = tr.cells[1] ? (tr.cells[1].textContent || '').trim() : '';
        var textOk = !q || nameCell.indexOf(q) !== -1;
        var typeOk = checkboxes.length === 0 || enabled[typeCell];
        tr.style.display = textOk && typeOk ? '' : 'none';
      });
    }

    input.addEventListener('input', applyFilter);
    checkboxes.forEach(function (cb) {
      cb.addEventListener('change', applyFilter);
    });
    // Apply the default (planet-only) filter on load.
    applyFilter();
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

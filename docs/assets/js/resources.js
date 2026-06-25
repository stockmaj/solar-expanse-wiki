// Boiling-point selector for the resources page.
// Reads body pressure data embedded as JSON by gen_pages.rs, populates the
// datalist typeahead, and recalculates the Boiling pt. column whenever the
// selected body or pressure changes.
//
// Formula: TerraformationConfig.cs line 599 (Clausius-Clapeyron).
//   T_boil = P <= 0 ? T_ref : max(0, 1 / (1/T_ref - (8.314/L) * ln(P)))
// where P is in Earth atmospheres (earthPressure = 1.0 in the game config).
(function () {
  var bodiesEl = document.getElementById('boiling-bodies');
  if (!bodiesEl) return;
  var bodies;
  try {
    bodies = JSON.parse(bodiesEl.textContent);
  } catch (e) {
    return;
  }

  var bodyInput = document.getElementById('boiling-body-input');
  var pressureInput = document.getElementById('boiling-pressure');
  var datalist = document.getElementById('boiling-body-list');
  if (!bodyInput || !pressureInput || !datalist) return;

  // Populate datalist with all named bodies.
  bodies.forEach(function (b) {
    var opt = document.createElement('option');
    opt.value = b.name;
    datalist.appendChild(opt);
  });

  // Game formula — TerraformationConfig.cs line 599.
  function effectiveBoilingTemp(tRef, latentHeat, pressure) {
    if (pressure <= 0) return tRef;
    return Math.max(0, 1 / (1 / tRef - (8.314 / latentHeat) * Math.log(pressure)));
  }

  function formatTemp(k) {
    var c = Math.round(k - 273.15);
    return Math.round(k) + ' K / ' + c + ' °C';
  }

  function updateTable(pressure) {
    document.querySelectorAll('.boiling-temp').forEach(function (span) {
      var tRef = parseFloat(span.dataset.refK);
      var lh = parseFloat(span.dataset.latentHeat);
      if (isNaN(tRef) || isNaN(lh)) return;
      span.textContent = formatTemp(effectiveBoilingTemp(tRef, lh, pressure));
    });
  }

  function findBody(name) {
    var lower = name.toLowerCase();
    return bodies.find(function (b) { return b.name.toLowerCase() === lower; });
  }

  // Initialize at Earth pressure.
  var earthBody = findBody('Earth');
  var initPressure = earthBody ? earthBody.pressure : 1.0;
  pressureInput.value = initPressure;
  updateTable(initPressure);

  // Select all text on focus so the user can start typing immediately.
  bodyInput.addEventListener('focus', function () {
    bodyInput.select();
  });

  // When user finalizes a body name (select from list or blur), sync pressure.
  bodyInput.addEventListener('change', function () {
    var match = findBody(bodyInput.value);
    if (match) {
      pressureInput.value = match.pressure;
      updateTable(match.pressure);
    }
  });

  // When user types directly in the pressure field, update table immediately.
  pressureInput.addEventListener('input', function () {
    var p = parseFloat(pressureInput.value);
    if (!isNaN(p) && p >= 0) {
      bodyInput.value = '';
      updateTable(p);
    }
  });
})();

// Boiling-point selector for the resources page.
// Reads body pressure data embedded as JSON by gen_pages.rs, populates a
// custom combobox, and recalculates the Boiling pt. column whenever the
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
  var dropdown = document.getElementById('boiling-body-dropdown');
  if (!bodyInput || !pressureInput || !dropdown) return;

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

  var lastValidBody = findBody('Earth') || bodies[0];

  function selectBody(b) {
    lastValidBody = b;
    bodyInput.value = b.name;
    pressureInput.value = b.pressure;
    updateTable(b.pressure);
    closeDropdown();
  }

  function buildDropdown(filter) {
    var q = filter ? filter.toLowerCase() : '';
    var matches = q
      ? bodies.filter(function (b) { return b.name.toLowerCase().indexOf(q) !== -1; })
      : bodies;
    dropdown.innerHTML = '';
    matches.forEach(function (b) {
      var div = document.createElement('div');
      div.textContent = b.name;
      div.addEventListener('mousedown', function (e) {
        e.preventDefault(); // keep input focused
        selectBody(b);
      });
      dropdown.appendChild(div);
    });
    dropdown.hidden = matches.length === 0;
  }

  function openDropdown() {
    buildDropdown(bodyInput.value);
  }

  function closeDropdown() {
    dropdown.hidden = true;
  }

  // Show full list on focus or click; clear so user can type immediately.
  bodyInput.addEventListener('focus', function () {
    bodyInput.value = '';
    buildDropdown('');
  });
  bodyInput.addEventListener('click', function () {
    bodyInput.value = '';
    buildDropdown('');
  });

  // Filter as user types.
  bodyInput.addEventListener('input', function () {
    buildDropdown(bodyInput.value);
  });

  // On blur: close dropdown; restore last valid body if nothing was selected.
  bodyInput.addEventListener('blur', function () {
    closeDropdown();
    if (!findBody(bodyInput.value)) {
      bodyInput.value = lastValidBody ? lastValidBody.name : '';
    }
  });

  // Manual pressure input clears body label and updates table.
  pressureInput.addEventListener('input', function () {
    var p = parseFloat(pressureInput.value);
    if (!isNaN(p) && p >= 0) {
      bodyInput.value = '';
      updateTable(p);
    }
  });

  // Initialize at Earth pressure.
  if (lastValidBody) {
    bodyInput.value = lastValidBody.name;
    pressureInput.value = lastValidBody.pressure;
    updateTable(lastValidBody.pressure);
  }
})();

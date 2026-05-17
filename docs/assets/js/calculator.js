// Colony-construction calculator.
//
// Pure logic + DOM binding in one file.  Pure functions are exposed on
// `globalThis.Calculator` and via CommonJS so calculator.test.js can run
// the additive-stacking math under Node without a DOM.

(function (root) {
  // ----- Pure logic ------------------------------------------------------

  // Additive stacking — matches the game's BonusController.cs:146-159.
  // Multiplier = max(0, (100 - sum(percent_i)) / 100).
  function buildCostMultiplier(facility, checkedReductions) {
    var sum = 0;
    for (var i = 0; i < checkedReductions.length; i++) {
      var r = checkedReductions[i];
      if (r.kind !== 'BuildCost') continue;
      var matches = r.affects_all === true ||
        (Array.isArray(r.affects) && r.affects.indexOf(facility.id) !== -1);
      if (matches) sum += r.percent;
    }
    var m = (100 - sum) / 100;
    return m < 0 ? 0 : m;
  }

  // For one facility, return { resource_id: amount } after applying the
  // checked BuildCost reductions.  Amounts are rounded to whole numbers
  // (in-game costs are integers).
  function applyReductions(facility, checkedReductions) {
    var m = buildCostMultiplier(facility, checkedReductions);
    var out = {};
    (facility.build_cost || []).forEach(function (bc) {
      out[bc.resource] = Math.round(bc.amount * m);
    });
    return out;
  }

  // Sum costs across all placed facilities (each with its own count).
  // `placed` is [{ facility, count }, ...].
  function totalResources(placed, checkedReductions) {
    var totals = {};
    placed.forEach(function (p) {
      var per = applyReductions(p.facility, checkedReductions);
      Object.keys(per).forEach(function (rid) {
        totals[rid] = (totals[rid] || 0) + per[rid] * p.count;
      });
    });
    return totals;
  }

  function matches(reduction, facility) {
    return reduction.affects_all === true ||
      (Array.isArray(reduction.affects) && reduction.affects.indexOf(facility.id) !== -1);
  }

  function crewMultiplier(facility, checkedReductions) {
    var sum = 0;
    for (var i = 0; i < checkedReductions.length; i++) {
      var r = checkedReductions[i];
      if (r.kind === 'ReduceCrewRequirements' && matches(r, facility)) sum += r.percent;
    }
    var m = (100 - sum) / 100;
    return m < 0 ? 0 : m;
  }

  function powerProductionMultiplier(facility, checkedReductions) {
    var sum = 0;
    for (var i = 0; i < checkedReductions.length; i++) {
      var r = checkedReductions[i];
      if (r.kind === 'PowerProduction' && matches(r, facility)) sum += r.percent;
    }
    return (100 + sum) / 100;
  }

  function workerTotal(placed, checkedReductions) {
    var total = 0;
    placed.forEach(function (p) {
      var m = crewMultiplier(p.facility, checkedReductions);
      total += (p.facility.workers_required || 0) * m * p.count;
    });
    return Math.round(total);
  }

  // Mirrors gen_pages.rs::fmt_abbrev — 1.2k / 200k / 1.5M / 2B style.
  // Used for the per-row resource pips where horizontal space is tight.
  function fmtAbbrev(v) {
    if (v <= 0) return '0';
    var scaled, suffix;
    if (v >= 1e9) { scaled = v / 1e9; suffix = 'B'; }
    else if (v >= 1e6) { scaled = v / 1e6; suffix = 'M'; }
    else if (v >= 1e3) { scaled = v / 1e3; suffix = 'k'; }
    else return String(Math.round(v));
    if (Math.abs(scaled - Math.round(scaled)) < 0.05) return Math.round(scaled) + suffix;
    return scaled.toFixed(1) + suffix;
  }

  function addSaved(saved, name, placed) {
    var trimmed = String(name).trim();
    if (!trimmed) return saved.slice();
    var entry = { name: trimmed, placed: Object.assign({}, placed) };
    var idx = -1;
    for (var i = 0; i < saved.length; i++) {
      if (saved[i].name === trimmed) { idx = i; break; }
    }
    var next = saved.slice();
    if (idx >= 0) next[idx] = entry;
    else next.push(entry);
    return next;
  }

  function removeSaved(saved, name) {
    return saved.filter(function (s) { return s.name !== name; });
  }

  // Crew transport: ceil(humans / capacity) modules, each module's dry mass
  // plus 1 t per human (humans = "human" resource, 1 t per unit).
  function crewTransportMass(humans, transport) {
    if (!transport || humans <= 0) return { capsules: 0, mass: 0 };
    var capsules = Math.ceil(humans / transport.capacity);
    var mass = capsules * transport.mass + humans;
    return { capsules: capsules, mass: mass };
  }

  // Net power need = consumption − (production × PowerProduction bonus).
  // Positive → deficit, negative → surplus.
  function powerNetTotal(placed, checkedReductions) {
    var net = 0;
    placed.forEach(function (p) {
      var m = powerProductionMultiplier(p.facility, checkedReductions);
      net += ((p.facility.energy_consumption || 0) - (p.facility.power_production || 0) * m) * p.count;
    });
    return Math.round(net);
  }

  // ----- DOM binding -----------------------------------------------------

  var STORAGE_KEY = 'solar-expanse-calc-v1';
  var SAVED_KEY = 'solar-expanse-calc-saved-v1';

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function loadState() {
    try {
      var raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return { placed: {}, checked: {}, crewTransport: null };
      var s = JSON.parse(raw);
      return {
        placed: (s && s.placed) || {},
        checked: (s && s.checked) || {},
        crewTransport: (s && s.crewTransport) || null,
      };
    } catch (e) {
      return { placed: {}, checked: {}, crewTransport: null };
    }
  }

  function saveState(state) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch (e) { /* quota / private-mode — ignore */ }
  }

  function loadSavedList() {
    try {
      var raw = localStorage.getItem(SAVED_KEY);
      if (!raw) return [];
      var parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
      return [];
    }
  }

  function persistSavedList(saved) {
    try {
      localStorage.setItem(SAVED_KEY, JSON.stringify(saved));
    } catch (e) { /* quota / private-mode — ignore */ }
  }

  function siteBase() {
    var scripts = document.getElementsByTagName('script');
    for (var i = 0; i < scripts.length; i++) {
      var s = scripts[i].getAttribute('src') || '';
      var idx = s.indexOf('/assets/js/calculator.js');
      if (idx !== -1) return s.slice(0, idx);
    }
    return '';
  }

  function dataUrl() {
    return siteBase() + '/assets/data/calculator.json';
  }

  // The PNG files live in docs/images/resources/. Resource ids map 1:1 to the
  // filename except for `hel3`, which uses the all-caps `HEL3.png`.
  function iconFile(resourceId) {
    if (resourceId === 'hel3') return 'HEL3.png';
    return resourceId + '.png';
  }

  function iconUrl(resourceId) {
    return siteBase() + '/images/resources/' + iconFile(resourceId);
  }

  function bindDom() {
    var root = document.getElementById('calc-root');
    if (!root) return;

    fetch(dataUrl(), { cache: 'no-store' })
      .then(function (r) {
        if (!r.ok) throw new Error('HTTP ' + r.status);
        return r.json();
      })
      .then(function (data) { render(root, data); })
      .catch(function (err) { renderError(root, err); });
  }

  function renderError(root, err) {
    root.innerHTML =
      '<div class="calculator-error">' +
      '<h3>Calculator data not available</h3>' +
      '<p>Could not load <code>assets/data/calculator.json</code> ' +
      '(' + escapeHtml(String(err && err.message || err)) + ').</p>' +
      '<p>This file is produced by the wiki extraction pipeline. ' +
      'Regenerate by running the extractor from the repo root:</p>' +
      '<pre><code>cd extract &amp;&amp; ./extract.sh</code></pre>' +
      '<p>Then reload this page.</p>' +
      '</div>';
  }

  function render(root, data) {
    var state = loadState();
    pruneState(state, data);

    root.innerHTML =
      renderReductions(data, state) +
      '<div class="calculator">' +
        '<div class="calc-pane calc-pane-left">' +
          '<div class="calc-pane-header"><h3>Facilities</h3></div>' +
          '<input type="search" class="calc-filter" placeholder="Filter facilities…" id="calc-facility-filter">' +
          '<div class="calc-facility-list" id="calc-facility-list"></div>' +
        '</div>' +
        '<div class="calc-pane calc-pane-mid">' +
          '<div class="calc-pane-header">' +
            '<h3>Placed</h3>' +
            '<button type="button" class="calc-save" id="calc-save">Save…</button>' +
            '<button type="button" class="calc-reset" id="calc-reset">Reset</button>' +
          '</div>' +
          '<div class="calc-saved" id="calc-saved"></div>' +
          '<div class="calc-placed-list" id="calc-placed-list" data-drop="1"></div>' +
        '</div>' +
        '<div class="calc-pane calc-pane-right">' +
          '<div class="calc-pane-header"><h3>Resources needed</h3></div>' +
          '<div id="calc-totals"></div>' +
        '</div>' +
      '</div>';

    var ctx = {
      data: data,
      state: state,
      saved: loadSavedList(),
      facById: indexBy(data.facilities, 'id'),
      resById: indexBy(data.resources, 'id'),
      redById: indexBy(data.reductions, 'id'),
    };

    bindReductions(root, ctx);
    bindFacilityList(root, ctx);
    bindPlaced(root, ctx);
    bindReset(root, ctx);
    bindSave(root, ctx);
    setupTooltips(root);
    rerenderAll(root, ctx);
  }

  function setupTooltips(root) {
    if (root._tipBound) return;
    root._tipBound = true;

    var tip = document.createElement('div');
    tip.className = 'calc-tooltip';
    document.body.appendChild(tip);

    var showTimer = null;
    var currentTarget = null;

    function place(target) {
      var rect = target.getBoundingClientRect();
      tip.style.left = (rect.left + rect.width / 2) + 'px';
      tip.style.top = rect.top + 'px';
    }

    root.addEventListener('mouseover', function (ev) {
      var t = ev.target.closest('[data-tip]');
      if (!t || !root.contains(t) || t === currentTarget) return;
      currentTarget = t;
      clearTimeout(showTimer);
      showTimer = setTimeout(function () {
        tip.textContent = t.getAttribute('data-tip') || '';
        place(t);
        tip.classList.add('calc-tooltip-show');
      }, 100);
    });
    root.addEventListener('mouseout', function (ev) {
      var t = ev.target.closest('[data-tip]');
      if (!t) return;
      var to = ev.relatedTarget;
      if (to && t.contains(to)) return;
      currentTarget = null;
      clearTimeout(showTimer);
      tip.classList.remove('calc-tooltip-show');
    });
    window.addEventListener('scroll', function () {
      if (currentTarget) tip.classList.remove('calc-tooltip-show');
    }, true);
  }

  function indexBy(arr, key) {
    var out = {};
    (arr || []).forEach(function (x) { out[x[key]] = x; });
    return out;
  }

  // Drop placed/checked entries whose ids no longer exist in the data.
  function pruneState(state, data) {
    var facIds = {};
    (data.facilities || []).forEach(function (f) { facIds[f.id] = true; });
    Object.keys(state.placed).forEach(function (id) {
      if (!facIds[id]) delete state.placed[id];
    });
    var redIds = {};
    (data.reductions || []).forEach(function (r) { redIds[r.id] = true; });
    Object.keys(state.checked).forEach(function (id) {
      if (!redIds[id]) delete state.checked[id];
    });
  }

  // ----- Reductions block -----------------------------------------------

  var KINDS = [
    { kind: 'BuildCost', label: 'Construction Reduction' },
    { kind: 'PowerProduction', label: 'Power Production' },
    { kind: 'ReduceCrewRequirements', label: 'Crew Requirement Reduction' },
  ];

  function renderReductions(data, state) {
    var sections = KINDS.map(function (k) {
      var items = (data.reductions || [])
        .filter(function (r) { return r.kind === k.kind; })
        .slice()
        .sort(function (a, b) { return a.name.localeCompare(b.name); });
      var inner = items.length === 0
        ? '<p class="calc-empty"><em>No ' + escapeHtml(k.label.toLowerCase()) + ' research in the data.</em></p>'
        : '<ul class="calc-reduction-list">' + items.map(function (r) {
            var checked = state.checked[r.id] ? ' checked' : '';
            var pct = (typeof r.percent === 'number') ? (' (' + r.percent + '%)') : '';
            return '<li><label>' +
              '<input type="checkbox" class="calc-reduction" data-id="' + escapeHtml(r.id) + '"' + checked + '> ' +
              escapeHtml(r.name) + pct +
              '</label></li>';
          }).join('') + '</ul>';
      return '<section class="calc-reduction-section">' +
        '<h4>' + escapeHtml(k.label) + '</h4>' +
        inner +
        '</section>';
    }).join('');

    return '<details class="calc-bonuses">' +
      '<summary>Tech bonuses</summary>' +
      '<p class="calc-hint">Stacks additively, matching the game.</p>' +
      sections +
      '</details>';
  }

  function bindReductions(root, ctx) {
    root.querySelectorAll('.calc-reduction').forEach(function (cb) {
      cb.addEventListener('change', function () {
        var id = cb.getAttribute('data-id');
        if (cb.checked) ctx.state.checked[id] = true;
        else delete ctx.state.checked[id];
        saveState(ctx.state);
        rerenderTotals(root, ctx);
      });
    });
  }

  // ----- Facility list (left pane) --------------------------------------

  function renderFacilityList(ctx, filter) {
    var q = (filter || '').trim().toLowerCase();
    var byCat = {};
    var catOrder = [];
    ctx.data.facilities.forEach(function (f) {
      if (!byCat[f.category]) {
        byCat[f.category] = [];
        catOrder.push(f.category);
      }
      byCat[f.category].push(f);
    });
    catOrder.sort(function (a, b) { return a.localeCompare(b); });

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    var html = '';
    var anyShown = false;
    catOrder.forEach(function (cat) {
      var items = byCat[cat].slice().sort(function (a, b) { return a.name.localeCompare(b.name); });
      var visible = items.filter(function (f) {
        return !q || f.name.toLowerCase().indexOf(q) !== -1;
      });
      if (visible.length === 0) return;
      anyShown = true;
      html += '<div class="calc-facility-cat">' +
        '<div class="calc-facility-cat-name">' + escapeHtml(cat) + '</div>' +
        '<ul>' +
        visible.map(function (f) {
          var reduced = applyReductions(f, checked);
          var pipsHtml = (f.build_cost || []).map(function (bc) {
            var amount = reduced[bc.resource] || 0;
            var resName = (ctx.resById[bc.resource] && ctx.resById[bc.resource].name) || bc.resource;
            return '<span class="calc-pip" data-tip="' + escapeHtml(resName + ': ' + amount.toLocaleString()) + '">' +
              '<span class="calc-pip-num">' + fmtAbbrev(amount) + '</span>' +
              '<img class="calc-pip-icon" src="' + escapeHtml(iconUrl(bc.resource)) + '" alt="">' +
              '</span>';
          }).join('');
          return '<li class="calc-facility-item" draggable="true" data-id="' + escapeHtml(f.id) + '">' +
            '<span class="calc-facility-name">' + escapeHtml(f.name) + '</span>' +
            '<span class="calc-facility-cost">' + pipsHtml + '</span>' +
            '</li>';
        }).join('') +
        '</ul></div>';
    });
    if (!anyShown) {
      html = '<p class="calc-empty"><em>No facilities match.</em></p>';
    }
    return html;
  }

  function bindFacilityList(root, ctx) {
    var listEl = root.querySelector('#calc-facility-list');
    var filterEl = root.querySelector('#calc-facility-filter');

    function repaint() {
      listEl.innerHTML = renderFacilityList(ctx, filterEl.value);
      listEl.querySelectorAll('.calc-facility-item').forEach(function (li) {
        var id = li.getAttribute('data-id');
        li.addEventListener('click', function () { addFacility(root, ctx, id, 1); });
        li.addEventListener('dragstart', function (ev) {
          ev.dataTransfer.effectAllowed = 'copy';
          ev.dataTransfer.setData('text/plain', id);
        });
      });
    }

    filterEl.addEventListener('input', repaint);
    repaint();
  }

  // ----- Placed list (middle pane) --------------------------------------

  function addFacility(root, ctx, id, delta) {
    if (!ctx.facById[id]) return;
    var cur = ctx.state.placed[id] || 0;
    ctx.state.placed[id] = Math.max(0, cur + delta);
    saveState(ctx.state);
    rerenderPlaced(root, ctx);
    rerenderTotals(root, ctx);
  }

  function setFacilityCount(root, ctx, id, value) {
    if (!ctx.facById[id]) return;
    ctx.state.placed[id] = Math.max(0, Math.floor(value));
    saveState(ctx.state);
    rerenderPlaced(root, ctx);
    rerenderTotals(root, ctx);
  }

  function renderPlacedList(ctx) {
    var ids = Object.keys(ctx.state.placed);
    if (ids.length === 0) {
      return '<p class="calc-empty calc-placed-empty"><em>Click facilities on the left to add them.</em></p>';
    }
    var rows = ids
      .map(function (id) { return { id: id, fac: ctx.facById[id], count: ctx.state.placed[id] }; })
      .filter(function (r) { return r.fac; })
      .sort(function (a, b) { return a.fac.name.localeCompare(b.fac.name); });

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    return '<ul class="calc-placed">' + rows.map(function (r) {
      var reduced = applyReductions(r.fac, checked);
      var pipsHtml = (r.fac.build_cost || []).map(function (bc) {
        var amount = (reduced[bc.resource] || 0) * r.count;
        var resName = (ctx.resById[bc.resource] && ctx.resById[bc.resource].name) || bc.resource;
        return '<span class="calc-pip" data-tip="' + escapeHtml(resName + ': ' + amount.toLocaleString()) + '">' +
          '<span class="calc-pip-num">' + fmtAbbrev(amount) + '</span>' +
          '<img class="calc-pip-icon" src="' + escapeHtml(iconUrl(bc.resource)) + '" alt="">' +
          '</span>';
      }).join('');
      return '<li class="calc-placed-row" data-id="' + escapeHtml(r.id) + '">' +
        '<span class="calc-placed-name">' + escapeHtml(r.fac.name) + '</span>' +
        '<span class="calc-placed-cost">' + pipsHtml + '</span>' +
        '<span class="calc-placed-counter">' +
          '<button type="button" class="calc-dec" data-id="' + escapeHtml(r.id) + '">−</button>' +
          '<span class="calc-count" data-id="' + escapeHtml(r.id) + '" data-tip="Click to edit">' + r.count + '</span>' +
          '<button type="button" class="calc-inc" data-id="' + escapeHtml(r.id) + '">+</button>' +
        '</span>' +
        '<button type="button" class="calc-remove" data-id="' + escapeHtml(r.id) + '" data-tip="Remove">×</button>' +
        '</li>';
    }).join('') + '</ul>';
  }

  function bindPlaced(root, ctx) {
    var dropEl = root.querySelector('#calc-placed-list');

    dropEl.addEventListener('dragover', function (ev) {
      ev.preventDefault();
      ev.dataTransfer.dropEffect = 'copy';
      dropEl.classList.add('calc-drop-hover');
    });
    dropEl.addEventListener('dragleave', function () {
      dropEl.classList.remove('calc-drop-hover');
    });
    dropEl.addEventListener('drop', function (ev) {
      ev.preventDefault();
      dropEl.classList.remove('calc-drop-hover');
      var id = ev.dataTransfer.getData('text/plain');
      if (id) addFacility(root, ctx, id, 1);
    });
  }

  function attachPlacedHandlers(root, ctx) {
    root.querySelectorAll('.calc-inc').forEach(function (b) {
      b.addEventListener('click', function () { addFacility(root, ctx, b.getAttribute('data-id'), 1); });
    });
    root.querySelectorAll('.calc-dec').forEach(function (b) {
      b.addEventListener('click', function () { addFacility(root, ctx, b.getAttribute('data-id'), -1); });
    });
    root.querySelectorAll('.calc-remove').forEach(function (b) {
      b.addEventListener('click', function () {
        var id = b.getAttribute('data-id');
        delete ctx.state.placed[id];
        saveState(ctx.state);
        rerenderPlaced(root, ctx);
        rerenderTotals(root, ctx);
      });
    });
    root.querySelectorAll('.calc-count').forEach(function (span) {
      span.addEventListener('click', function () {
        var id = span.getAttribute('data-id');
        var cur = ctx.state.placed[id] || 0;
        var inp = document.createElement('input');
        inp.type = 'number';
        inp.min = '0';
        inp.value = String(cur);
        inp.className = 'calc-count-input';
        span.replaceWith(inp);
        inp.focus();
        inp.select();
        function commit() {
          var v = parseInt(inp.value, 10);
          if (!isNaN(v)) setFacilityCount(root, ctx, id, v);
          else rerenderPlaced(root, ctx);
        }
        inp.addEventListener('blur', commit);
        inp.addEventListener('keydown', function (e) {
          if (e.key === 'Enter') { e.preventDefault(); commit(); }
          if (e.key === 'Escape') { e.preventDefault(); rerenderPlaced(root, ctx); }
        });
      });
    });
  }

  function bindReset(root, ctx) {
    var btn = root.querySelector('#calc-reset');
    if (!btn) return;
    btn.addEventListener('click', function () {
      ctx.state.placed = {};
      saveState(ctx.state);
      rerenderPlaced(root, ctx);
      rerenderTotals(root, ctx);
    });
  }

  // ----- Saved lists ----------------------------------------------------

  function renderSavedList(ctx) {
    if (!ctx.saved.length) return '';
    return ctx.saved.map(function (s) {
      return '<span class="calc-saved-item" data-name="' + escapeHtml(s.name) + '">' +
        '<button type="button" class="calc-saved-load" data-name="' + escapeHtml(s.name) + '" data-tip="Load">' +
        escapeHtml(s.name) + '</button>' +
        '<button type="button" class="calc-saved-delete" data-name="' + escapeHtml(s.name) + '" data-tip="Delete">×</button>' +
      '</span>';
    }).join('');
  }

  function bindSave(root, ctx) {
    var btn = root.querySelector('#calc-save');
    if (btn) {
      btn.addEventListener('click', function () {
        if (!Object.keys(ctx.state.placed).length) return;
        var name = prompt('Name this list:');
        if (name === null) return;
        ctx.saved = addSaved(ctx.saved, name, ctx.state.placed);
        persistSavedList(ctx.saved);
        rerenderSaved(root, ctx);
      });
    }
    attachSavedHandlers(root, ctx);
  }

  function attachSavedHandlers(root, ctx) {
    root.querySelectorAll('.calc-saved-load').forEach(function (b) {
      b.addEventListener('click', function () {
        var name = b.getAttribute('data-name');
        var entry = ctx.saved.find(function (s) { return s.name === name; });
        if (!entry) return;
        ctx.state.placed = Object.assign({}, entry.placed);
        pruneState(ctx.state, ctx.data);
        saveState(ctx.state);
        rerenderPlaced(root, ctx);
        rerenderTotals(root, ctx);
      });
    });
    root.querySelectorAll('.calc-saved-delete').forEach(function (b) {
      b.addEventListener('click', function (ev) {
        ev.stopPropagation();
        var name = b.getAttribute('data-name');
        ctx.saved = removeSaved(ctx.saved, name);
        persistSavedList(ctx.saved);
        rerenderSaved(root, ctx);
      });
    });
  }

  function rerenderSaved(root, ctx) {
    var el = root.querySelector('#calc-saved');
    el.innerHTML = renderSavedList(ctx);
    attachSavedHandlers(root, ctx);
  }

  // ----- Totals (right pane) --------------------------------------------

  function renderTotals(ctx) {
    var placed = Object.keys(ctx.state.placed)
      .map(function (id) { return { facility: ctx.facById[id], count: ctx.state.placed[id] }; })
      .filter(function (p) { return p.facility; });

    if (placed.length === 0) {
      return '<p class="calc-empty"><em>Add facilities to see totals.</em></p>';
    }

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    var totals = totalResources(placed, checked);
    var resourceRows = Object.keys(totals)
      .map(function (rid) {
        return {
          rid: rid,
          name: (ctx.resById[rid] && ctx.resById[rid].name) || rid,
          amount: totals[rid],
        };
      })
      .filter(function (r) { return r.amount > 0; })
      .sort(function (a, b) { return b.amount - a.amount; });

    var workers = workerTotal(placed, checked);
    var powerNet = powerNetTotal(placed, checked);

    var grand = resourceRows.reduce(function (sum, r) { return sum + r.amount; }, 0);

    var extraRows = [];
    if (workers > 0) extraRows.push({ rid: 'human', name: 'Humans', amount: workers });
    if (powerNet !== 0) extraRows.push({ rid: 'energy', name: 'Power (net)', amount: powerNet });

    if (resourceRows.length === 0 && extraRows.length === 0) {
      return '<p class="calc-empty"><em>All costs reduced to 0.</em></p>';
    }

    return '<table class="calc-totals"><thead><tr><th>Resource</th><th>Amount</th></tr></thead><tbody>' +
      resourceRows.concat(extraRows).map(function (r) {
        return '<tr><td>' +
          '<img class="calc-total-icon" src="' + escapeHtml(iconUrl(r.rid)) + '" alt=""> ' +
          escapeHtml(r.name) + '</td>' +
          '<td class="calc-num">' + r.amount.toLocaleString() + '</td></tr>';
      }).join('') +
      '</tbody><tfoot><tr><td><strong>Total tons</strong></td>' +
      '<td class="calc-num"><strong>' + grand.toLocaleString() + '</strong></td></tr></tfoot></table>';
  }

  function rerenderAll(root, ctx) {
    rerenderPlaced(root, ctx);
    rerenderTotals(root, ctx);
    rerenderSaved(root, ctx);
  }

  function rerenderPlaced(root, ctx) {
    var el = root.querySelector('#calc-placed-list');
    el.innerHTML = renderPlacedList(ctx);
    attachPlacedHandlers(root, ctx);
  }

  function rerenderTotals(root, ctx) {
    var el = root.querySelector('#calc-totals');
    el.innerHTML = renderTotals(ctx);
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', bindDom);
  }

  root.Calculator = {
    applyReductions: applyReductions,
    buildCostMultiplier: buildCostMultiplier,
    totalResources: totalResources,
    workerTotal: workerTotal,
    powerNetTotal: powerNetTotal,
    addSaved: addSaved,
    removeSaved: removeSaved,
    iconFile: iconFile,
    fmtAbbrev: fmtAbbrev,
    crewTransportMass: crewTransportMass,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.Calculator;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

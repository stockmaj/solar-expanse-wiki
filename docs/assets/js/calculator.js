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

  function encodeShareState(state) {
    var payload = {
      p: state.placed,
      c: state.checked,
      sc: state.spacecraft,
      os: state.onSite,
    };
    return btoa(JSON.stringify(payload))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }

  function decodeShareState(encoded) {
    try {
      var b64 = encoded.replace(/-/g, '+').replace(/_/g, '/');
      while (b64.length % 4) b64 += '=';
      var payload = JSON.parse(atob(b64));
      return {
        placed: payload.p || {},
        checked: payload.c || {},
        spacecraft: payload.sc || null,
        onSite: payload.os || {},
      };
    } catch (e) { return null; }
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
  // plus 1 t per human.
  //
  // The +1 t/human term is observed for Module Crew Compartment (5-seat,
  // 5 t empty → 10 t loaded with 5 crew). The dump doesn't carry a per-human
  // mass field, so we extrapolate to Medium / Large. If the in-game launch
  // UI shows a different loaded mass for those once unlocked, update here.
  function crewTransportMass(humans, transport) {
    if (!transport || humans <= 0) return { capsules: 0, mass: 0 };
    var capsules = Math.ceil(humans / transport.capacity);
    var mass = capsules * transport.mass + humans;
    return { capsules: capsules, mass: mass };
  }

  // Sum of build_time_days × count.  Assumes serial construction (one
  // construction slot); the in-game pace can be parallelised, so this is an
  // upper bound.
  function buildDayTotal(placed) {
    var total = 0;
    placed.forEach(function (p) {
      total += (p.facility.build_time_days || 0) * p.count;
    });
    return Math.round(total);
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
      if (!raw) return { placed: {}, checked: {}, spacecraft: null, onSite: {} };
      var s = JSON.parse(raw);
      return {
        placed: (s && s.placed) || {},
        checked: (s && s.checked) || {},
        spacecraft: (s && s.spacecraft) || null,
        onSite: (s && s.onSite) || {},
      };
    } catch (e) {
      return { placed: {}, checked: {}, spacecraft: null, onSite: {} };
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
    // A `?p=<base64>` query string overrides whatever's in localStorage on
    // load. Once applied we drop the query string so a refresh doesn't keep
    // re-importing it (and we persist the merged state to localStorage).
    var state = loadState();
    var qs = (typeof location !== 'undefined' ? location.search : '') || '';
    var match = qs.match(/[?&]p=([^&]+)/);
    if (match) {
      var imported = decodeShareState(decodeURIComponent(match[1]));
      if (imported) {
        state = imported;
        saveState(state);
        if (window.history && window.history.replaceState) {
          window.history.replaceState({}, '', location.pathname);
        }
      }
    }
    pruneState(state, data);

    root.innerHTML =
      renderReductions(data, state) +
      '<div class="calculator">' +
        '<div class="calc-pane calc-pane-left">' +
          '<div class="calc-pane-header"><h3>Facilities</h3></div>' +
          '<input type="search" class="calc-filter" placeholder="Filter facilities…" id="calc-facility-filter">' +
          '<div class="calc-facility-list" id="calc-facility-list"></div>' +
        '</div>' +
        '<div class="calc-pane calc-pane-mid" id="calc-placed-drop">' +
          '<div class="calc-pane-header">' +
            '<h3>Placed</h3>' +
            '<button type="button" class="calc-save" id="calc-save">Save…</button>' +
            '<button type="button" class="calc-save" id="calc-share">Share</button>' +
            '<button type="button" class="calc-reset" id="calc-reset">Reset</button>' +
          '</div>' +
          '<div class="calc-saved" id="calc-saved"></div>' +
          '<div class="calc-placed-list" id="calc-placed-list"></div>' +
        '</div>' +
        '<div class="calc-pane calc-pane-right">' +
          '<div class="calc-pane-header"><h3>Resources needed</h3></div>' +
          '<div class="calc-crew-picker" id="calc-spacecraft-picker"></div>' +
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

    ctx.crewById = (data.crew_transports || []).reduce(function (m, c) { m[c.id] = c; return m; }, {});

    var scById = (data.spacecraft || []).reduce(function (m, s) { m[s.id] = s; return m; }, {});
    if (!ctx.state.spacecraft || !scById[ctx.state.spacecraft]) {
      // Default to the smallest, always-available chemical spacecraft (Iris).
      ctx.state.spacecraft = (data.spacecraft || [])[0] ? data.spacecraft[0].id : null;
      saveState(ctx.state);
    }
    ctx.scById = scById;

    bindReductions(root, ctx);
    bindFacilityList(root, ctx);
    bindPlaced(root, ctx);
    bindReset(root, ctx);
    bindSave(root, ctx);
    bindShare(root, ctx);
    bindSpacecraftPicker(root, ctx);
    setupTooltips(root);
    rerenderAll(root, ctx);
  }

  function bindShare(root, ctx) {
    var btn = root.querySelector('#calc-share');
    if (!btn) return;
    btn.addEventListener('click', function () {
      if (!Object.keys(ctx.state.placed).length) return;
      var encoded = encodeShareState(ctx.state);
      var url = location.origin + location.pathname + '?p=' + encoded;
      var original = btn.textContent;
      function flash(msg) {
        btn.textContent = msg;
        setTimeout(function () { btn.textContent = original; }, 1500);
      }
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(url).then(
          function () { flash('Copied!'); },
          function () { prompt('Copy this link:', url); }
        );
      } else {
        prompt('Copy this link:', url);
      }
    });
  }

  function renderSpacecraftPicker(ctx) {
    var ships = ctx.data.spacecraft || [];
    if (!ships.length) return '';
    return '<label>Spacecraft: ' +
      '<select id="calc-sc-select">' +
      ships.map(function (s) {
        var sel = s.id === ctx.state.spacecraft ? ' selected' : '';
        return '<option value="' + escapeHtml(s.id) + '"' + sel + '>' +
          escapeHtml(s.name) + ' — ' + s.cargo_capacity + ' t / trip' +
          '</option>';
      }).join('') +
      '</select></label>';
  }

  function bindSpacecraftPicker(root, ctx) {
    var el = root.querySelector('#calc-spacecraft-picker');
    if (!el) return;
    el.innerHTML = renderSpacecraftPicker(ctx);
    var sel = el.querySelector('#calc-sc-select');
    if (sel) {
      sel.addEventListener('change', function () {
        ctx.state.spacecraft = sel.value;
        saveState(ctx.state);
        rerenderTotals(root, ctx);
      });
    }
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
    var placeableIds = {};
    (data.facilities || []).forEach(function (f) { placeableIds[f.id] = true; });
    (data.crew_transports || []).forEach(function (c) { placeableIds[c.id] = true; });
    Object.keys(state.placed).forEach(function (id) {
      if (!placeableIds[id]) delete state.placed[id];
    });
    var redIds = {};
    (data.reductions || []).forEach(function (r) { redIds[r.id] = true; });
    Object.keys(state.checked).forEach(function (id) {
      if (!redIds[id]) delete state.checked[id];
    });
    // Drop on-site entries for resources we no longer recognise.
    var resIds = {};
    (data.resources || []).forEach(function (r) { resIds[r.id] = true; });
    resIds.human = true;  // human isn't in resources[] but is a valid on-site key
    Object.keys(state.onSite || {}).forEach(function (rid) {
      if (!resIds[rid]) delete state.onSite[rid];
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
    // Crew transports as their own pseudo-category, pinned to the bottom.
    var crewItems = (ctx.data.crew_transports || []).map(function (c) {
      return {
        id: c.id, name: c.name,
        _isCapsule: true, _mass: c.mass, _capacity: c.capacity, _locked: c.is_locked,
      };
    });
    if (crewItems.length) {
      byCat['Crew Transport'] = crewItems;
      catOrder.push('Crew Transport');
    }

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    var html = '';
    var anyShown = false;
    catOrder.forEach(function (cat) {
      var items = byCat[cat].slice().sort(function (a, b) {
        // Within Crew Transport, sort by capacity ascending instead of name.
        if (a._isCapsule && b._isCapsule) return a._capacity - b._capacity;
        return a.name.localeCompare(b.name);
      });
      var visible = items.filter(function (f) {
        return !q || f.name.toLowerCase().indexOf(q) !== -1;
      });
      if (visible.length === 0) return;
      anyShown = true;
      html += '<div class="calc-facility-cat">' +
        '<div class="calc-facility-cat-name">' + escapeHtml(cat) + '</div>' +
        '<ul>' +
        visible.map(function (f) {
          var pipsHtml = '';
          if (f._isCapsule) {
            var lockTag = f._locked ? ' 🔒' : '';
            pipsHtml = '<span class="calc-capsule-info" data-tip="' +
              escapeHtml(f._capacity + ' seats, ' + f._mass + 't empty (' + (f._mass + f._capacity) + 't fully loaded)') + '">' +
              f._capacity + ' seats · ' + f._mass + 't' + lockTag +
              '</span>';
          } else {
            var reduced = applyReductions(f, checked);
            pipsHtml = (f.build_cost || []).map(function (bc) {
              var amount = reduced[bc.resource] || 0;
              var resName = (ctx.resById[bc.resource] && ctx.resById[bc.resource].name) || bc.resource;
              return '<span class="calc-pip" data-tip="' + escapeHtml(resName + ': ' + amount.toLocaleString()) + '">' +
                '<span class="calc-pip-num">' + fmtAbbrev(amount) + '</span>' +
                '<img class="calc-pip-icon" src="' + escapeHtml(iconUrl(bc.resource)) + '" alt="">' +
                '</span>';
            }).join('');
          }
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
    if (!ctx.facById[id] && !ctx.crewById[id]) return;
    var cur = ctx.state.placed[id] || 0;
    ctx.state.placed[id] = Math.max(0, cur + delta);
    saveState(ctx.state);
    rerenderPlaced(root, ctx);
    rerenderTotals(root, ctx);
  }

  function setFacilityCount(root, ctx, id, value) {
    if (!ctx.facById[id] && !ctx.crewById[id]) return;
    ctx.state.placed[id] = Math.max(0, Math.floor(value));
    saveState(ctx.state);
    rerenderPlaced(root, ctx);
    rerenderTotals(root, ctx);
  }

  function renderPlacedList(ctx) {
    var ids = Object.keys(ctx.state.placed);
    if (ids.length === 0) {
      return '<p class="calc-empty calc-placed-empty"><em>Click items on the left to add them.</em></p>';
    }
    var rows = ids
      .map(function (id) {
        var fac = ctx.facById[id];
        var cap = ctx.crewById[id];
        return { id: id, fac: fac, capsule: cap, name: (fac || cap || {}).name, count: ctx.state.placed[id] };
      })
      .filter(function (r) { return r.fac || r.capsule; })
      .sort(function (a, b) { return a.name.localeCompare(b.name); });

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    return '<ul class="calc-placed">' + rows.map(function (r) {
      var pipsHtml = '';
      if (r.capsule) {
        var totalMass = r.capsule.mass * r.count;
        pipsHtml = '<span class="calc-pip" data-tip="' +
          escapeHtml(r.count + ' × ' + r.capsule.mass + 't = ' + totalMass + 't (modules empty)') + '">' +
          '<span class="calc-pip-num">' + fmtAbbrev(totalMass) + 't</span>' +
          '</span>';
      } else {
        var reduced = applyReductions(r.fac, checked);
        pipsHtml = (r.fac.build_cost || []).map(function (bc) {
          var amount = (reduced[bc.resource] || 0) * r.count;
          var resName = (ctx.resById[bc.resource] && ctx.resById[bc.resource].name) || bc.resource;
          return '<span class="calc-pip" data-tip="' + escapeHtml(resName + ': ' + amount.toLocaleString()) + '">' +
            '<span class="calc-pip-num">' + fmtAbbrev(amount) + '</span>' +
            '<img class="calc-pip-icon" src="' + escapeHtml(iconUrl(bc.resource)) + '" alt="">' +
            '</span>';
        }).join('');
      }
      return '<li class="calc-placed-row" data-id="' + escapeHtml(r.id) + '">' +
        '<span class="calc-placed-name">' + escapeHtml(r.name) + '</span>' +
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
    var dropEl = root.querySelector('#calc-placed-drop');

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
    var allPlaced = Object.keys(ctx.state.placed).map(function (id) {
      return { id: id, fac: ctx.facById[id], cap: ctx.crewById[id], count: ctx.state.placed[id] };
    }).filter(function (p) { return (p.fac || p.cap) && p.count > 0; });

    if (allPlaced.length === 0) {
      return '<p class="calc-empty"><em>Add items to see totals.</em></p>';
    }

    var facilityPlaced = allPlaced.filter(function (p) { return p.fac; })
      .map(function (p) { return { facility: p.fac, count: p.count }; });
    var capsulePlaced = allPlaced.filter(function (p) { return p.cap; });

    var checked = Object.keys(ctx.state.checked)
      .map(function (id) { return ctx.redById[id]; })
      .filter(Boolean);

    // Build cargo rows from facility build_cost. Each row carries `needed`
    // (raw demand) and `onSite` (user input); `amount` is what's left to ship.
    var totals = totalResources(facilityPlaced, checked);
    var cargoRows = Object.keys(totals).map(function (rid) {
      var needed = totals[rid];
      var onSite = (ctx.state.onSite && ctx.state.onSite[rid]) || 0;
      return {
        rid: rid,
        name: (ctx.resById[rid] && ctx.resById[rid].name) || rid,
        needed: needed,
        onSite: onSite,
        amount: Math.max(0, needed - onSite),
        editable: true,
      };
    });

    // Capsule rows — one per placed type, assumed fully loaded (capacity
    // humans inside each). Mass = count × (module + capacity × 1 t/human).
    capsulePlaced.forEach(function (p) {
      var perLoaded = p.cap.mass + p.cap.capacity;
      var totalMass = perLoaded * p.count;
      cargoRows.push({
        rid: 'human',
        name: p.cap.name + ' ×' + p.count + ' (' + (p.cap.capacity * p.count) + ' humans)',
        needed: totalMass,
        onSite: 0,
        amount: totalMass,
        editable: false,
        tip: p.count + ' × (' + p.cap.mass + 't module + ' + p.cap.capacity + 't crew) = ' + totalMass + 't',
      });
    });

    var workers = workerTotal(facilityPlaced, checked);
    var onSiteHumans = (ctx.state.onSite && ctx.state.onSite.human) || 0;
    var onboardCapacity = capsulePlaced.reduce(function (s, p) { return s + p.cap.capacity * p.count; }, 0);
    var humansToShip = Math.max(0, workers - onSiteHumans);

    cargoRows = cargoRows.filter(function (r) { return r.amount > 0 || r.editable; });
    // Sort: editable resource rows by amount desc, then capsule/human rows last.
    cargoRows.sort(function (a, b) {
      if (a.editable !== b.editable) return a.editable ? -1 : 1;
      return b.amount - a.amount;
    });

    var grand = cargoRows.reduce(function (sum, r) { return sum + r.amount; }, 0);

    var operationalRows = [];
    if (workers > 0) {
      var shortage = Math.max(0, humansToShip - onboardCapacity);
      var humansLabel = 'Humans ' + workers;
      var detail = onboardCapacity + ' onboard' +
        (onSiteHumans > 0 ? ', ' + onSiteHumans + ' on site' : '');
      operationalRows.push({
        rid: 'human',
        name: humansLabel + ' (' + detail + ')',
        amount: shortage > 0 ? '−' + shortage + ' short' : 'covered',
        tip: 'Demand ' + workers + ' = facilities × workers_required (after crew-reduction research). ' +
             'Onboard = Σ capsule capacity × count. On-site = your input (Humans row in cargo).',
      });
    }
    var powerNet = powerNetTotal(facilityPlaced, checked);
    if (powerNet !== 0) operationalRows.push({ rid: 'energy', name: 'Power (net)', amount: powerNet });
    var days = buildDayTotal(facilityPlaced);
    if (days > 0) operationalRows.push({
      name: 'Total build days', amount: days,
      tip: 'Sum of build_time × count. Assumes serial construction; in-game you can parallelize across construction equipment slots.',
    });

    if (cargoRows.length === 0 && operationalRows.length === 0) {
      return '<p class="calc-empty"><em>All costs reduced to 0.</em></p>';
    }

    function cargoRowHtml(r) {
      var tipAttr = r.tip ? ' data-tip="' + escapeHtml(r.tip) + '"' : '';
      var iconHtml = r.rid
        ? '<img class="calc-total-icon" src="' + escapeHtml(iconUrl(r.rid)) + '" alt=""> '
        : '';
      var onSiteCell = r.editable
        ? '<input type="number" min="0" class="calc-onsite" data-rid="' + escapeHtml(r.rid) +
          '" value="' + (r.onSite || '') + '" placeholder="0">'
        : '—';
      return '<tr' + tipAttr + '><td>' + iconHtml + escapeHtml(r.name) + '</td>' +
        '<td class="calc-num calc-onsite-cell">' + onSiteCell + '</td>' +
        '<td class="calc-num">' + r.amount.toLocaleString() + '</td></tr>';
    }
    function opRowHtml(r) {
      var tipAttr = r.tip ? ' data-tip="' + escapeHtml(r.tip) + '"' : '';
      var iconHtml = r.rid
        ? '<img class="calc-total-icon" src="' + escapeHtml(iconUrl(r.rid)) + '" alt=""> '
        : '';
      var amt = typeof r.amount === 'number' ? r.amount.toLocaleString() : r.amount;
      return '<tr' + tipAttr + '><td colspan="2">' + iconHtml + escapeHtml(r.name) + '</td>' +
        '<td class="calc-num">' + amt + '</td></tr>';
    }

    var html = '<table class="calc-totals">';
    if (cargoRows.length) {
      var totalRow = '<tr class="calc-total-row"><td colspan="2"><strong>Total tons</strong></td>' +
        '<td class="calc-num"><strong>' + grand.toLocaleString() + '</strong></td></tr>';
      var trips = '';
      var ship = ctx.scById && ctx.scById[ctx.state.spacecraft];
      if (ship && grand > 0) {
        var n = Math.ceil(grand / ship.cargo_capacity);
        trips = '<tr><td colspan="2">' + escapeHtml(ship.name) + ' trips ' +
          '<span class="calc-trip-note">(' + ship.cargo_capacity + ' t / trip)</span></td>' +
          '<td class="calc-num">' + n.toLocaleString() + '</td></tr>';
      }
      html += '<thead><tr><th class="calc-section">Cargo (to ship)</th><th class="calc-section calc-num">On site</th><th class="calc-section calc-num">Tons</th></tr></thead>' +
        '<tbody>' + cargoRows.map(cargoRowHtml).join('') + totalRow + trips + '</tbody>';
    }
    if (operationalRows.length) {
      html += '<thead><tr><th colspan="3" class="calc-section">Operational (on-site)</th></tr></thead>' +
        '<tbody>' + operationalRows.map(opRowHtml).join('') + '</tbody>';
    }
    html += '</table>';
    return html;
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
    // Preserve focus across the re-render so the user can keep typing into
    // an on-site input — capture the focused rid + cursor position first,
    // re-render, then restore.
    var prev = document.activeElement;
    var focusRid = (prev && prev.classList && prev.classList.contains('calc-onsite'))
      ? prev.getAttribute('data-rid') : null;
    var focusCursor = focusRid ? prev.selectionStart : null;

    el.innerHTML = renderTotals(ctx);

    if (focusRid) {
      var next = el.querySelector('.calc-onsite[data-rid="' + focusRid + '"]');
      if (next) {
        next.focus();
        try { if (focusCursor != null) next.setSelectionRange(focusCursor, focusCursor); } catch (e) {}
      }
    }

    el.querySelectorAll('.calc-onsite').forEach(function (inp) {
      inp.addEventListener('input', function () {
        var rid = inp.getAttribute('data-rid');
        var v = parseFloat(inp.value);
        if (!ctx.state.onSite) ctx.state.onSite = {};
        if (isNaN(v) || v <= 0) delete ctx.state.onSite[rid];
        else ctx.state.onSite[rid] = v;
        saveState(ctx.state);
        rerenderTotals(root, ctx);
      });
    });
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
    buildDayTotal: buildDayTotal,
    encodeShareState: encodeShareState,
    decodeShareState: decodeShareState,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.Calculator;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

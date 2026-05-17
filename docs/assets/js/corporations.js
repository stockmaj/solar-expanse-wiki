// Corporation comparison view for the /corporations/ page.
//
// The Rust generator emits a CORP_DATA blob with every scenario / corp /
// research combination, plus the three difficulty multipliers.  This file
// provides the pure render functions (buildComparison, renderTableMarkup,
// formatMoney) — easy to unit-test under Node — and a bindDom() that wires
// the two <select> dropdowns at the top of the page and re-renders the
// comparison table on change.

(function (root) {
  // The default-selected scenario when the page first loads.  Early
  // Exploration is the most playable starting epoch — every corp begins
  // with no pre-built fleet and a smaller research tree, so the comparison
  // table shows the broadest possible delta as the player progresses.
  var DEFAULT_SCENARIO_ID = 'StartGameEpoch_EarlyExploration';

  // ----- Pure logic ------------------------------------------------------

  // Format a dollar amount the same way fmt_abbrev() does on the Rust side,
  // so that the live calculator agrees with the Difficulty table the page
  // already prints from gen_pages.rs.  Cutoffs: 1e9 → B, 1e6 → M, 1e3 → k.
  // Round to whole-number suffix when the scaled value is within 0.05 of an
  // integer (matches the Rust check), otherwise one decimal place.
  function formatMoney(v) {
    if (!v || v <= 0) return '$0';
    var scaled, suffix;
    if (v >= 1e9) { scaled = v / 1e9; suffix = 'B'; }
    else if (v >= 1e6) { scaled = v / 1e6; suffix = 'M'; }
    else if (v >= 1e3) { scaled = v / 1e3; suffix = 'k'; }
    else { return '$' + Math.round(v); }
    var rounded = Math.round(scaled);
    if (Math.abs(scaled - rounded) < 0.05) {
      return '$' + rounded + suffix;
    }
    // toFixed(1) does banker-ish rounding via the float; close enough for the
    // values we ever see ($20M–$500M scale, multiplied by 0.75/1.0/1.25).
    return '$' + scaled.toFixed(1) + suffix;
  }

  function findScenario(data, scenarioId) {
    for (var i = 0; i < data.scenarios.length; i++) {
      if (data.scenarios[i].id === scenarioId) return data.scenarios[i];
    }
    return null;
  }

  function findDifficulty(data, name) {
    for (var i = 0; i < data.difficulties.length; i++) {
      if (data.difficulties[i].name === name) return data.difficulties[i];
    }
    return null;
  }

  // Given a parsed CORP_DATA blob plus scenario-id + difficulty-name,
  // produce the data the comparison table needs:
  //   { corpNames, cash, lvCounts, scCounts,
  //     researchRows: [{name, category, held: [bool…]}] }
  // Each corp's research is now [{name, category}…] — `category` is the
  // player-facing tech-tree sub-branch label (e.g. "Spacecraft", "Chemical
  // Propulsion"). The Rust generator already humanizes camelCase ids and
  // falls back to "Other" when the dump doesn't carry a sub-branch, but we
  // defensively apply the same fallback here so renderTableMarkup can
  // assume every row has a non-empty category.
  //
  // Sort: category alphabetical (primary), name alphabetical (secondary)
  // — this lets renderTableMarkup emit a category header before each
  // consecutive cluster without re-sorting.
  function buildComparison(data, scenarioId, difficultyName) {
    var scenario = findScenario(data, scenarioId);
    var diff = findDifficulty(data, difficultyName);
    if (!scenario || !diff) {
      return { corpNames: [], cash: [], lvCounts: [], scCounts: [], researchRows: [] };
    }
    var corps = scenario.corps;
    var corpNames = corps.map(function (c) { return c.name; });
    var cash = corps.map(function (c) { return c.starting_money * diff.money_multiplier; });
    var lvCounts = corps.map(function (c) { return c.lv_count; });
    var scCounts = corps.map(function (c) { return c.sc_count; });

    function entryCategory(e) {
      return (e && e.category) ? e.category : 'Other';
    }

    // Union of all research display names held by any corp in this scenario.
    // Track the category alongside the name so it survives into the row.
    var seen = Object.create(null);
    var union = [];
    corps.forEach(function (c) {
      (c.research || []).forEach(function (entry) {
        var name = entry && entry.name ? entry.name : String(entry);
        var category = entryCategory(entry);
        if (!seen[name]) {
          seen[name] = true;
          union.push({ name: name, category: category });
        }
      });
    });
    union.sort(function (a, b) {
      var byCat = a.category.localeCompare(b.category);
      return byCat !== 0 ? byCat : a.name.localeCompare(b.name);
    });

    var researchRows = union.map(function (u) {
      var held = corps.map(function (c) {
        return (c.research || []).some(function (e) {
          var nm = e && e.name ? e.name : String(e);
          return nm === u.name;
        });
      });
      return { name: u.name, category: u.category, held: held };
    });

    // Filter out parity rows — research held by every corp (all ✓) or no
    // corp (all —).  The point of the matrix is to show DIFFERENCES; rows
    // with parity convey nothing.  Track how many we dropped so the page
    // can note "N research items with parity not shown."
    var totalBeforeFilter = researchRows.length;
    researchRows = researchRows.filter(function (r) {
      var firstVal = r.held[0];
      return r.held.some(function (v) { return v !== firstVal; });
    });
    var parityHidden = totalBeforeFilter - researchRows.length;

    return {
      corpNames: corpNames,
      cash: cash,
      lvCounts: lvCounts,
      scCounts: scCounts,
      researchRows: researchRows,
      parityHidden: parityHidden,
    };
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  // Render the comparison view from the buildComparison() result.
  //
  // Layout: a flex/grid wrapper holding two side-by-side tables that share
  // the same per-corp <thead> row.
  //   - Left table:  Starting cash / Pre-built LVs / Pre-built spacecraft
  //                  (the small fleet/economy block).
  //   - Right table: Completed research, grouped by category.
  // On narrow viewports the wrapper stacks the two tables vertically
  // (see .corp-comparison-split CSS in wiki.css).
  //
  // Kept in this module so the test suite can assert on the markup shape
  // without needing jsdom.
  function renderTableMarkup(cmp) {
    if (!cmp.corpNames.length) {
      return '<p><em>No corporation data for this scenario.</em></p>';
    }
    // Per-corp <thead> row — emitted into both tables so each one is
    // self-readable when the layout stacks vertically on narrow screens.
    var head = '<tr><th>Item</th>' +
      cmp.corpNames.map(function (n) {
        return '<th>' + escapeHtml(n) + '</th>';
      }).join('') + '</tr>';

    // ----- Left table: economy / pre-built fleet -----
    var leftRows = [];
    leftRows.push('<tr><td><strong>Starting cash</strong></td>' +
      cmp.cash.map(function (v) {
        return '<td style="text-align:center">' + escapeHtml(formatMoney(v)) + '</td>';
      }).join('') + '</tr>');
    leftRows.push('<tr><td><strong title="Number of launch vehicles already assembled in the corp\'s fleet at scenario start (not how many they could research)">Pre-built launch vehicles</strong></td>' +
      cmp.lvCounts.map(function (v) {
        return '<td style="text-align:center">' + v + '</td>';
      }).join('') + '</tr>');
    leftRows.push('<tr><td><strong title="Number of spacecraft already constructed in the corp\'s fleet at scenario start (not how many craft types they could build)">Pre-built spacecraft</strong></td>' +
      cmp.scCounts.map(function (v) {
        return '<td style="text-align:center">' + v + '</td>';
      }).join('') + '</tr>');

    // ----- Right table: completed research, grouped by category -----
    var rightRows = [];
    var prevCategory = null;
    cmp.researchRows.forEach(function (r) {
      if (r.category !== prevCategory) {
        rightRows.push('<tr class="corp-research-category"><td colspan="' +
          (cmp.corpNames.length + 1) +
          '" style="padding-left:16px;color:var(--accent-dim,#88a);text-align:left;font-weight:600;font-size:0.9em;border-top:1px solid var(--border,#444);background:transparent">' +
          escapeHtml(r.category) + '</td></tr>');
        prevCategory = r.category;
      }
      rightRows.push('<tr><td style="padding-left:32px">' + escapeHtml(r.name) + '</td>' +
        r.held.map(function (h) {
          return '<td style="text-align:center">' + (h ? '✓' : '—') + '</td>';
        }).join('') + '</tr>');
    });

    var leftTable = '<table class="corp-comparison-left"><thead>' + head +
      '</thead><tbody>' + leftRows.join('') + '</tbody></table>';
    var rightTable = '<table class="corp-comparison-right"><thead>' + head +
      '</thead><tbody>' + rightRows.join('') + '</tbody></table>';

    var parityNote = '';
    if (cmp.parityHidden && cmp.parityHidden > 0) {
      parityNote = '<p class="corp-parity-note" style="font-size:12px;color:var(--fg-muted);margin-top:4px">' +
        cmp.parityHidden +
        ' research item' + (cmp.parityHidden === 1 ? '' : 's') +
        ' shared by every corp (or by none) hidden — only differences shown.</p>';
    }

    return '<div class="corp-comparison-split">' + leftTable + rightTable + '</div>' + parityNote;
  }

  // ----- DOM binding -----------------------------------------------------

  function bindDom() {
    var data = root.CORP_DATA;
    if (!data) return;
    var scenarioSel = document.getElementById('corp-scenario');
    var difficultySel = document.getElementById('corp-difficulty');
    var out = document.getElementById('corp-comparison');
    if (!scenarioSel || !difficultySel || !out) return;

    // Populate <option>s.  The Rust side already set sensible defaults
    // on the <select> elements (selected="selected"); we just need the
    // option lists to exist.
    if (scenarioSel.options.length === 0) {
      data.scenarios.forEach(function (s) {
        var o = document.createElement('option');
        o.value = s.id;
        o.textContent = s.name;
        scenarioSel.appendChild(o);
      });
      // Default to Early Exploration — the most playable starting point —
      // when the page's <select> was rendered with no pre-selected option.
      scenarioSel.value = DEFAULT_SCENARIO_ID;
    }
    if (difficultySel.options.length === 0) {
      data.difficulties.forEach(function (d) {
        var o = document.createElement('option');
        o.value = d.name;
        o.textContent = d.name;
        difficultySel.appendChild(o);
      });
      difficultySel.value = 'Pioneer';
    }

    function rerender() {
      var cmp = buildComparison(data, scenarioSel.value, difficultySel.value);
      out.innerHTML = renderTableMarkup(cmp);
    }

    scenarioSel.addEventListener('change', rerender);
    difficultySel.addEventListener('change', rerender);
    rerender();
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', bindDom);
  }

  // Expose pure functions for tests / external use.
  root.Corporations = {
    buildComparison: buildComparison,
    renderTableMarkup: renderTableMarkup,
    formatMoney: formatMoney,
    bindDom: bindDom,
    DEFAULT_SCENARIO_ID: DEFAULT_SCENARIO_ID,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = root.Corporations;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

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
  // Exploration starts every corp with zero pre-built facilities (and a
  // very small research delta), so the comparison table comes up nearly
  // empty there — players hit the page and see no facility differences.
  // The Expansion is the first scenario where the starting facility mix
  // diverges between corps, so it makes the more informative landing.
  // Users can still switch back via the scenario dropdown.
  var DEFAULT_SCENARIO_ID = 'StartGameEpoch_TheExpansion';

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
  function buildComparison(data, scenarioId, difficultyName, showAll) {
    var scenario = findScenario(data, scenarioId);
    var diff = findDifficulty(data, difficultyName);
    if (!scenario || !diff) {
      return { corpNames: [], cash: [], lvCounts: [], scCounts: [], researchRows: [], facilityTotals: [], facilityRows: [] };
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

    // Identify parity rows — research held by every corp (all ✓) or no
    // corp (all —).  Default: hide them; the point of the matrix is to
    // show DIFFERENCES.  showAll=true keeps them in.
    var totalBeforeFilter = researchRows.length;
    var nonParity = researchRows.filter(function (r) {
      var firstVal = r.held[0];
      return r.held.some(function (v) { return v !== firstVal; });
    });
    var parityCount = totalBeforeFilter - nonParity.length;

    if (!showAll) {
      researchRows = nonParity;
    }

    // ----- Starting facilities -------------------------------------------
    // Each corp ships with a `starting_facilities` array of `{name, count}`
    // objects (resolved + title-cased by the Rust generator).  We build a
    // matrix view: one row per facility name, with one count per corp
    // column (0 when the corp doesn't own that facility).
    //
    // Universal facilities (HQ, Main Building) appear on every corp at
    // every scenario and carry no comparison signal — drop them so the
    // table focuses on the interesting mining / extraction / refinery
    // differences.
    var UNIVERSAL_FACILITIES = { 'HQ': true, 'Main Building': true };
    // Summary-block total: count of EVERY starting facility per corp,
    // including universals (HQ + Main Building).  The per-facility
    // breakdown rows below the "Starting facilities" header still drop
    // universals via the filter on the next loop; only this aggregate
    // includes them so the summary reflects the corp's true build count.
    var facilityTotals = corps.map(function (c) {
      var entries = c.starting_facilities || [];
      var total = 0;
      for (var i = 0; i < entries.length; i++) {
        total += (entries[i] && entries[i].count) || 0;
      }
      return total;
    });
    var facilityNameSet = Object.create(null);
    var facilityNames = [];
    corps.forEach(function (c) {
      (c.starting_facilities || []).forEach(function (entry) {
        var name = entry && entry.name ? entry.name : null;
        if (!name || UNIVERSAL_FACILITIES[name]) return;
        if (!facilityNameSet[name]) {
          facilityNameSet[name] = true;
          facilityNames.push(name);
        }
      });
    });
    facilityNames.sort(function (a, b) { return a.localeCompare(b); });
    var facilityRows = facilityNames.map(function (n) {
      var counts = corps.map(function (c) {
        var entries = c.starting_facilities || [];
        for (var i = 0; i < entries.length; i++) {
          if (entries[i] && entries[i].name === n) {
            return entries[i].count || 0;
          }
        }
        return 0;
      });
      return { name: n, counts: counts };
    });

    return {
      corpNames: corpNames,
      cash: cash,
      lvCounts: lvCounts,
      scCounts: scCounts,
      researchRows: researchRows,
      facilityTotals: facilityTotals,
      facilityRows: facilityRows,
      // When showAll=false, parityHidden = count actively hidden.
      // When showAll=true, parityHidden is 0 but parityHiddenWhenFiltered
      // preserves the count for the "shown" footnote variant.
      parityHidden: showAll ? 0 : parityCount,
      parityHiddenWhenFiltered: parityCount,
      showAll: !!showAll,
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
    // Per-corp <thead> row.  The left table's leading column has no header
    // (the rows describe themselves: Starting cash / Pre-built ... etc.),
    // while the right table uses "Technology" to label the research column.
    var corpHeaders = cmp.corpNames.map(function (n) {
      return '<th>' + escapeHtml(n) + '</th>';
    }).join('');
    var leftHead = '<tr><th></th>' + corpHeaders + '</tr>';
    var rightHead = '<tr><th>Technology</th>' + corpHeaders + '</tr>';

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
    // True total of every facility already built at scenario start —
    // includes the universal HQ and Main Building that every corp always
    // owns.  The per-facility breakdown rows below the "Starting
    // facilities" category header drop those universals so the matrix
    // focuses on what differs; the summary total keeps them so the
    // headline number matches what the player actually sees in-game.
    leftRows.push('<tr><td><strong title="Total facilities already built at scenario start, counting HQ, Main Building, and every other facility (the per-facility breakdown below excludes universal HQ / Main Building)">Pre-built facilities</strong></td>' +
      (cmp.facilityTotals || []).map(function (v) {
        return '<td style="text-align:center">' + v + '</td>';
      }).join('') + '</tr>');

    // ----- Starting facilities -----
    // One category-header row (matching the look of the right table's
    // sub-branch headers), then one row per facility name with the per-corp
    // count (or em-dash for 0).  Section is only emitted when at least one
    // corp owns at least one facility — keeps Early Exploration (everyone
    // starts with nothing built) from rendering an empty header.
    var facilityRows = cmp.facilityRows || [];
    if (facilityRows.length > 0) {
      leftRows.push('<tr class="corp-facility-category"><td colspan="' +
        (cmp.corpNames.length + 1) +
        '" style="padding-left:16px;color:var(--accent-dim,#88a);text-align:left;font-weight:600;font-size:0.9em;border-top:1px solid var(--border,#444);background:transparent">' +
        'Starting facilities</td></tr>');
      facilityRows.forEach(function (r) {
        leftRows.push('<tr><td style="padding-left:32px">' + escapeHtml(r.name) + '</td>' +
          r.counts.map(function (n) {
            return '<td style="text-align:center">' + (n > 0 ? n : '—') + '</td>';
          }).join('') + '</tr>');
      });
    }

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

    var leftTable = '<table class="corp-comparison-left"><thead>' + leftHead +
      '</thead><tbody>' + leftRows.join('') + '</tbody></table>';
    var rightTable = '<table class="corp-comparison-right"><thead>' + rightHead +
      '</thead><tbody>' + rightRows.join('') + '</tbody></table>';

    // Parity note + Show All checkbox sit underneath the research (right)
    // table.  The checkbox toggles cmp.showAll via the DOM binding and
    // triggers a re-render with parity rows included.
    var parityControls = '';
    if (cmp.parityHidden && cmp.parityHidden > 0) {
      parityControls = '<div class="corp-parity-controls" style="font-size:12px;color:var(--fg-muted);margin-top:4px">' +
        '<label><input type="checkbox" id="corp-show-all-research"' +
        (cmp.showAll ? ' checked' : '') +
        ' style="vertical-align:middle;margin-right:4px"> Show all starting research</label>' +
        ' &middot; ' + cmp.parityHidden +
        ' research item' + (cmp.parityHidden === 1 ? '' : 's') +
        ' shared by every corp (or by none) hidden by default.</div>';
    } else if (cmp.showAll && cmp.parityHiddenWhenFiltered) {
      // When Show All is on, expose the toggle to turn it back off.
      parityControls = '<div class="corp-parity-controls" style="font-size:12px;color:var(--fg-muted);margin-top:4px">' +
        '<label><input type="checkbox" id="corp-show-all-research" checked' +
        ' style="vertical-align:middle;margin-right:4px"> Show all starting research</label>' +
        ' &middot; ' + cmp.parityHiddenWhenFiltered +
        ' research item' + (cmp.parityHiddenWhenFiltered === 1 ? '' : 's') +
        ' shared by every corp (or by none) — shown.</div>';
    }

    // Right-side cell wraps the parity controls ABOVE the research table.
    var rightCell = '<div class="corp-comparison-right-cell">' + parityControls + rightTable + '</div>';
    return '<div class="corp-comparison-split">' + leftTable + rightCell + '</div>';
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
      // Default to The Expansion — the first scenario where the starting
      // facility mix differs between corps — when the page's <select> was
      // rendered with no pre-selected option.
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

    var showAll = false;
    function rerender() {
      var cmp = buildComparison(data, scenarioSel.value, difficultySel.value, showAll);
      out.innerHTML = renderTableMarkup(cmp);
      // Wire the freshly-rendered checkbox.
      var cb = document.getElementById('corp-show-all-research');
      if (cb) {
        cb.addEventListener('change', function () {
          showAll = cb.checked;
          rerender();
        });
      }
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

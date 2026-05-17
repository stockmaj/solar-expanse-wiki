// Filter each facility table (Ground / Orbital) on the Facilities page by
// (a) text-search against the facility name column, and (b) per-table type
// checkboxes (Mining / Production / Power / …) sourced from each table's
// own set of facility types.  Filter block order in the DOM matches the
// order of the `main table` siblings (the block sits right before its
// table inside its H2 section).

(function () {
  function bind() {
    var filterBlocks = document.querySelectorAll('.facility-filter');
    var tables = document.querySelectorAll('main table');
    if (!filterBlocks.length || !tables.length) return;

    var states = [];
    filterBlocks.forEach(function (block, i) {
      states.push({
        block: block,
        search: block.querySelector('.facility-filter-search'),
        typeBoxes: Array.prototype.slice.call(
          block.querySelectorAll('.facility-type-filter')
        ),
        table: tables[i],
      });
    });

    function applyFilter(state) {
      if (!state.table) return;
      var q = state.search ? state.search.value.trim().toLowerCase() : '';
      var enabledTypes = {};
      var anyTypeFilter = state.typeBoxes.length > 0;
      state.typeBoxes.forEach(function (cb) {
        if (cb.checked) enabledTypes[cb.value] = true;
      });
      state.table.querySelectorAll('tbody tr').forEach(function (tr) {
        var name = tr.cells[0] ? (tr.cells[0].textContent || '').toLowerCase() : '';
        var type = tr.cells[1] ? (tr.cells[1].textContent || '').trim() : '';
        var textOk = !q || name.indexOf(q) !== -1;
        var typeOk = !anyTypeFilter || enabledTypes[type];
        tr.style.display = textOk && typeOk ? '' : 'none';
      });
    }

    states.forEach(function (state) {
      if (state.search) {
        state.search.addEventListener('input', function () { applyFilter(state); });
      }
      state.typeBoxes.forEach(function (cb) {
        cb.addEventListener('change', function () { applyFilter(state); });
      });
      applyFilter(state);
    });
  }

  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', bind);
    } else {
      bind();
    }
  }
})();

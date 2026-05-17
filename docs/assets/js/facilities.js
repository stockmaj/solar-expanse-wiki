// Filter the two facility tables (Ground / Orbital) on the Facilities page by
// the first-column (Facility) text.  Case-insensitive substring match.

(function () {
  function bind() {
    var input = document.getElementById('facility-filter');
    if (!input) return;
    var tables = document.querySelectorAll('main table');
    if (!tables.length) return;

    function applyFilter() {
      var q = input.value.trim().toLowerCase();
      tables.forEach(function (table) {
        table.querySelectorAll('tbody tr').forEach(function (tr) {
          var name = tr.cells[0] ? (tr.cells[0].textContent || '').toLowerCase() : '';
          tr.style.display = !q || name.indexOf(q) !== -1 ? '' : 'none';
        });
      });
    }

    input.addEventListener('input', applyFilter);
    applyFilter();
  }

  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', bind);
    } else {
      bind();
    }
  }
})();

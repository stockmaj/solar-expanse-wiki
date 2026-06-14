// Lightweight sortable-table support.  Each <th> with class="sort" becomes
// clickable; we sort the table body by that column on click and toggle
// direction.  Numeric vs. text is auto-detected from cell contents.
(function () {
  function cellValue(cell) {
    var t = (cell.textContent || '').trim();
    // Strip leading 'Cash:', resource labels, etc. and isolate the first number.
    var m = t.match(/-?\d[\d,\.]*[kMB]?/);
    if (!m) return { text: t.toLowerCase(), num: null };
    var raw = m[0].replace(/,/g, '');
    var mul = 1;
    if (raw.endsWith('k')) { mul = 1e3; raw = raw.slice(0, -1); }
    else if (raw.endsWith('M')) { mul = 1e6; raw = raw.slice(0, -1); }
    else if (raw.endsWith('B')) { mul = 1e9; raw = raw.slice(0, -1); }
    var n = parseFloat(raw);
    return { text: t.toLowerCase(), num: isNaN(n) ? null : n * mul };
  }
  function compare(a, b, asc) {
    if (a.num !== null && b.num !== null) {
      return asc ? a.num - b.num : b.num - a.num;
    }
    if (a.text < b.text) return asc ? -1 : 1;
    if (a.text > b.text) return asc ? 1 : -1;
    return 0;
  }
  var SKIP_BY_HEADER = /^(description|premise|notes|prereqs|prerequisites|unlocks|rewards|requirements|build cost|producers)$/i;
  function shouldSort(th, firstCell) {
    var label = (th.textContent || '').trim();
    if (SKIP_BY_HEADER.test(label)) return false;
    if (!firstCell) return true;
    var html = firstCell.innerHTML || '';
    if (/<br|<img|<ul|<ol/i.test(html)) return false;
    var text = (firstCell.textContent || '').trim();
    if (text.length > 80) return false;
    return true;
  }
  document.querySelectorAll('table').forEach(function (table) {
    if (table.closest('.no-sort')) return;
    var ths = table.querySelectorAll('thead th, tr:first-child th');
    var firstRow = (table.querySelector('tbody tr') || (function () {
      var trs = table.querySelectorAll('tr');
      return trs.length > 1 ? trs[1] : null;
    })());
    ths.forEach(function (th, idx) {
      var firstCell = firstRow ? firstRow.cells[idx] : null;
      if (!shouldSort(th, firstCell)) return;
      th.classList.add('sort');
      th.style.cursor = 'pointer';
      th.title = 'Sort by ' + (th.textContent || '').trim();
      th.addEventListener('click', function () {
        var tbody = table.querySelector('tbody') || table;
        var rows = Array.from(tbody.querySelectorAll('tr')).filter(function (r) {
          return r.parentElement === tbody && r.cells.length > idx;
        });
        var asc = th.getAttribute('data-sort-dir') !== 'asc';
        ths.forEach(function (other) {
          other.removeAttribute('data-sort-dir');
          other.classList.remove('sort-asc', 'sort-desc');
        });
        th.setAttribute('data-sort-dir', asc ? 'asc' : 'desc');
        th.classList.add(asc ? 'sort-asc' : 'sort-desc');
        rows.sort(function (a, b) {
          return compare(cellValue(a.cells[idx]), cellValue(b.cells[idx]), asc);
        });
        rows.forEach(function (r) { tbody.appendChild(r); });
      });
    });
  });
})();

// "Show unreleased" toggle.
(function () {
  function bind() {
    var toggles = document.querySelectorAll('.show-unreleased-toggle');
    if (!toggles.length) return;
    toggles.forEach(function (cb) {
      var node = cb.closest('.show-unreleased') || cb.parentElement;
      if (node && !node.nextElementSibling) { node = node.parentElement || node; }
      var table = null;
      while (node && node.nextElementSibling) {
        node = node.nextElementSibling;
        if (node.tagName === 'TABLE') { table = node; break; }
        var inner = node.querySelector && node.querySelector('table');
        if (inner) { table = inner; break; }
      }
      if (!table) return;
      function apply() {
        var show = cb.checked;
        table.querySelectorAll('tbody tr').forEach(function (tr) {
          if (tr.querySelector('.row-unreleased')) {
            tr.style.display = show ? '' : 'none';
          }
        });
      }
      cb.addEventListener('change', apply);
      apply();
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

// Export-CSV button injected above every <table> in <main>.
(function () {
  function cellText(cell) {
    return (cell.textContent || '').replace(/\s+/g, ' ').trim();
  }
  function csvEscape(val) {
    if (/[",\n\r]/.test(val)) return '"' + val.replace(/"/g, '""') + '"';
    return val;
  }
  function isCellVisible(cell) {
    return getComputedStyle(cell).display !== 'none';
  }
  function tableToCSV(table) {
    var lines = [];
    var headerRow = table.querySelector('thead tr') || table.querySelector('tr');
    if (headerRow) {
      lines.push(
        Array.from(headerRow.querySelectorAll('th,td'))
          .filter(isCellVisible)
          .map(function (c) { return csvEscape(cellText(c)); })
          .join(',')
      );
    }
    var tbody = table.querySelector('tbody') || table;
    Array.from(tbody.querySelectorAll('tr')).forEach(function (tr) {
      if (tr === headerRow) return;
      if (tr.style.display === 'none') return;
      var cells = Array.from(tr.querySelectorAll('td')).filter(isCellVisible);
      if (!cells.length) return;
      lines.push(cells.map(function (c) { return csvEscape(cellText(c)); }).join(','));
    });
    return lines.join('\r\n');
  }
  function nearestHeading(el) {
    var node = el.previousElementSibling;
    while (node) {
      if (/^H[1-4]$/.test(node.tagName)) return node.textContent.trim();
      node = node.previousElementSibling;
    }
    return null;
  }
  function slugify(s) {
    return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }
  var SKIP_CONTAINERS = '.calc, #calc-root, .corp-comparison-split, .corp-comparison-right';
  function bind() {
    document.querySelectorAll('main table').forEach(function (table) {
      if (table.closest(SKIP_CONTAINERS)) return;
      var btn = document.createElement('button');
      btn.className = 'export-csv-btn';
      btn.textContent = 'Export CSV';
      btn.type = 'button';
      btn.addEventListener('click', function () {
        var csv = tableToCSV(table);
        var heading = nearestHeading(table);
        var filename = (heading ? slugify(heading) : 'table') + '.csv';
        var blob = new Blob([csv], { type: 'text/csv' });
        var url = URL.createObjectURL(blob);
        var a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      });
      table.parentNode.insertBefore(btn, table);
    });
  }
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bind);
  } else {
    bind();
  }
})();

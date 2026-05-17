---
title: Calculator
layout: default
---

# Colony Construction Calculator

Plan what you need to ship to build out a colony. Pick facilities on the left;
the resource totals on the right update as you go. Check off the construction
research you've unlocked under **Tech bonuses** and the totals fold in the
discount (additive stacking, matching the game's `BonusController`).

Your selection is saved in this browser, so reloading keeps your plan.

<div id="calc-root"></div>

<script src="{{ '/assets/js/calculator.js' | relative_url }}?v={{ site.github.build_revision | default: 'dev' }}" defer></script>

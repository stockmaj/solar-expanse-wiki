# Launch Windows

**Jump to:** [Calculate a window](#window-calculator) · [Filter the body list](#body-table)

> **Heads-up:** these numbers are computed by the wiki from the orbital
> elements the game ships, *not* read from the game itself.  The in-game
> Plan Mission window uses live n-body propagation including gravitational
> perturbations and your spacecraft's specific Δv budget, so the dates and
> intervals here are a **planning approximation** — the porkchop plot is
> the source of truth at launch time.

## What counts as a launch window

A *launch window* here is the moment when an idealized **Hohmann transfer**
launched from Earth's orbit will arrive at the target body just as that body
reaches the transfer ellipse's far side.  Concretely, at the moment of
launch the target has to lead (for outer bodies) or trail (for inner bodies)
Earth by a specific phase angle so that body and spacecraft meet on arrival.
Earth–Mars windows recur every ~26 months (synodic period); the most recent
real-world ones were 2020-07, 2022-09, 2024-10.

This is a single idealised window per synodic period — *not* a multi-day
porkchop plot.  In practice the in-game planner gives you a range of days
on either side at slightly higher Δv cost; the table here is the centre of
that range.

The **synodic period** is how often the Earth-body pair returns to that
same relative geometry.  Computed from each body's semi-major axis via
Kepler's third law (`T_years = a^(3/2)`) and
`synodic = 1 / |1/T_earth − 1/T_body|`.

<div id="body-table">
<label>Filter: <input id="body-filter" type="search" placeholder="e.g., mars, ceres, 1P…"></label>

| Body | <span title="Average distance from the Sun in astronomical units (1 AU = Earth's distance)">Semi-major axis (AU)</span> | <span title="Time for one orbit around the Sun, derived from a via Kepler's third law">Orbital period</span> | <span title="Interval between consecutive Hohmann-style launch opportunities from Earth — the synodic period">Earth ↔ body window</span> |
| --- | --- | --- | --- |
| **Mercury** | 0.387 | 0.24 yr | 116 days (~3.8 months) |
| **Venus** | 0.723 | 0.62 yr | 584 days (~19.2 months) |
| **EX0-99 Extinctor** | 0.732 | 0.63 yr | 612 days (~20.1 months) |
| **99942 Apophis** | 0.923 | 0.89 yr | 7.8 years |
| **3753 Cruithne** | 0.998 | 1.00 yr | 294.0 years |
| **469219 Kamoʻoalewa** | 1.001 | 1.00 yr | 710.8 years |
| **101955 Bennu** | 1.126 | 1.20 yr | 6.1 years |
| **25143 Itokawa** | 1.324 | 1.52 yr | 2.9 years |
| **Mars** | 1.524 | 1.88 yr | 2.1 years |
| **098-Y Peppin** | 1.800 | 2.41 yr | 623 days (~20.5 months) |
| **2495 Noviomagum** | 1.918 | 2.66 yr | 586 days (~19.2 months) |
| **5426 Sharp** | 1.950 | 2.72 yr | 577 days (~19.0 months) |
| **2048 Dwornik** | 1.954 | 2.73 yr | 576 days (~18.9 months) |
| **7088 Ishtar** | 1.980 | 2.79 yr | 570 days (~18.7 months) |
| **UT7-55 Kutno** | 2.130 | 3.11 yr | 538 days (~17.7 months) |
| **8 Flora** | 2.202 | 3.27 yr | 526 days (~17.3 months) |
| **2P Encke** | 2.215 | 3.30 yr | 524 days (~17.2 months) |
| **12 Victoria** | 2.330 | 3.56 yr | 508 days (~16.7 months) |
| **4 Vesta** | 2.362 | 3.63 yr | 504 days (~16.6 months) |
| **7 Iris** | 2.386 | 3.69 yr | 501 days (~16.5 months) |
| **9 Metis** | 2.387 | 3.69 yr | 501 days (~16.5 months) |
| **6 Hebe** | 2.426 | 3.78 yr | 497 days (~16.3 months) |
| **11 Parthenope** | 2.450 | 3.83 yr | 494 days (~16.2 months) |
| **TJ66-2145** | 2.450 | 3.83 yr | 494 days (~16.2 months) |
| **5 Astraea** | 2.573 | 4.13 yr | 482 days (~15.8 months) |
| **13 Egeria** | 2.580 | 4.14 yr | 481 days (~15.8 months) |
| **KH7-23 Geraldino** | 2.640 | 4.29 yr | 476 days (~15.6 months) |
| **1036 Ganymed** | 2.663 | 4.35 yr | 474 days (~15.6 months) |
| **3 Juno** | 2.670 | 4.36 yr | 474 days (~15.6 months) |
| **FL8-09 Varsoviom** | 2.750 | 4.56 yr | 468 days (~15.4 months) |
| **1 Ceres** | 2.768 | 4.61 yr | 467 days (~15.3 months) |
| **2 Pallas** | 2.770 | 4.61 yr | 466 days (~15.3 months) |
| **267 Tirza** | 2.775 | 4.62 yr | 466 days (~15.3 months) |
| **PC0-01 Kurai** | 2.801 | 4.69 yr | 464 days (~15.3 months) |
| **PW4-13 Rider** | 3.010 | 5.22 yr | 452 days (~14.8 months) |
| **368 Haidea** | 3.070 | 5.38 yr | 449 days (~14.7 months) |
| **10 Hygiea** | 3.142 | 5.57 yr | 445 days (~14.6 months) |
| **AB2-38 Dover** | 3.220 | 5.78 yr | 442 days (~14.5 months) |
| **BG1-65 Usher** | 3.330 | 6.08 yr | 437 days (~14.4 months) |
| **MP3-87 Nosfer** | 3.450 | 6.41 yr | 433 days (~14.2 months) |
| **TT-9025** | 3.540 | 6.66 yr | 430 days (~14.1 months) |
| **ZZ9-01 Nebulavsky** | 3.670 | 7.03 yr | 426 days (~14.0 months) |
| **4P Faye** | 3.838 | 7.52 yr | 421 days (~13.8 months) |
| **KB5-98 Kris** | 3.910 | 7.73 yr | 420 days (~13.8 months) |
| **2312 Duboshin** | 3.970 | 7.91 yr | 418 days (~13.7 months) |
| **DE8-42 Sunset** | 4.000 | 8.00 yr | 417 days (~13.7 months) |
| **279 Thule** | 4.260 | 8.79 yr | 412 days (~13.5 months) |
| **659 Nestor** | 5.170 | 11.76 yr | 399 days (~13.1 months) |
| **Jupiter** | 5.203 | 11.87 yr | 399 days (~13.1 months) |
| **617 Patroclus** | 5.209 | 11.89 yr | 399 days (~13.1 months) |
| **588 Achilles** | 5.209 | 11.89 yr | 399 days (~13.1 months) |
| **1172 Aneas** | 5.218 | 11.92 yr | 399 days (~13.1 months) |
| **3317 Paris** | 5.222 | 11.93 yr | 399 days (~13.1 months) |
| **624 Hektor** | 5.257 | 12.05 yr | 398 days (~13.1 months) |
| **911 Agamemnon** | 5.277 | 12.12 yr | 398 days (~13.1 months) |
| **Saturn** | 9.537 | 29.45 yr | 378 days (~12.4 months) |
| **1P Halley** | 17.834 | 75.31 yr | 370 days (~12.2 months) |
| **Uranus** | 19.189 | 84.06 yr | 370 days (~12.1 months) |
| **Neptune** | 30.070 | 164.89 yr | 367 days (~12.1 months) |
| **Pluto** | 39.482 | 248.09 yr | 367 days (~12.0 months) |

</div>

## Practical reading

- **Earth → Mercury** opens most often — ~116 days, less than every 4 months.
- **Earth → Venus** ~19 months.
- **Earth → Mars** opens roughly every 26 months — every mid-game player has
watched their cargo manifest waiting for one of these.
- **Earth → Jupiter and beyond** are short intervals (~13 months) because the
outer planets move slowly relative to Earth, so Earth laps them almost
yearly.  The Hohmann transfer itself takes years.
- Asteroid-belt bodies sit between Mars and Jupiter — windows ~14–16 months.

Moons aren't here — launching from Earth to the Moon (or Phobos, Europa, etc.)
doesn't have a useful synodic period; you wait for your spacecraft to be
ready and the in-game flight planner handles phasing.

## Window calculator

<a id="window-calculator"></a>

Pick a *from* body, *to* body, and a start date.  The calculator lists the
next five Hohmann-transfer launch windows from that pair, plus the arrival
date for each (transfer time = `0.5 × ((a_from + a_to) / 2)^1.5` years).

Same caveat as the table above: this is a Keplerian approximation anchored
at the game's earliest contract epoch (1959-01-01).  Spacing between windows
is reliable; absolute dates may drift from the in-game porkchop plot by days
to weeks.

<div class="calc">
<label>From: <select id="calc-from"></select></label>
<label>To: <select id="calc-to"></select></label>
<label>Start date: <input type="date" id="calc-date" value="2020-01-01"></label>
<div id="calc-result"></div>
</div>

<script>
window.LAUNCH_WINDOW_ALL_BODIES = [{"name":"Earth","a":1.000001,"longitude":168},{"name":"Mercury","a":0.3870993,"longitude":252.25032},{"name":"Venus","a":0.7233357,"longitude":181.9791},{"name":"EX0-99 Extinctor","a":0.73211825,"longitude":70},{"name":"99942 Apophis","a":0.9227,"longitude":100},{"name":"3753 Cruithne","a":0.99774,"longitude":84},{"name":"469219 Kamoʻoalewa","a":1.00094,"longitude":190},{"name":"101955 Bennu","a":1.1264,"longitude":84},{"name":"25143 Itokawa","a":1.3241,"longitude":84},{"name":"Mars","a":1.52371,"longitude":0},{"name":"098-Y Peppin","a":1.8,"longitude":20},{"name":"2495 Noviomagum","a":1.9176,"longitude":125},{"name":"5426 Sharp","a":1.95,"longitude":137},{"name":"2048 Dwornik","a":1.953737,"longitude":256},{"name":"7088 Ishtar","a":1.9804,"longitude":127},{"name":"UT7-55 Kutno","a":2.13,"longitude":305},{"name":"8 Flora","a":2.2015843,"longitude":303},{"name":"2P Encke","a":2.215132,"longitude":0},{"name":"12 Victoria","a":2.33,"longitude":75},{"name":"4 Vesta","a":2.3619132,"longitude":314},{"name":"7 Iris","a":2.3858037,"longitude":29},{"name":"9 Metis","a":2.3865738,"longitude":0},{"name":"6 Hebe","a":2.4260397,"longitude":207},{"name":"11 Parthenope","a":2.45,"longitude":299},{"name":"TJ66-2145","a":2.45,"longitude":102},{"name":"5 Astraea","a":2.57348,"longitude":167},{"name":"13 Egeria","a":2.58,"longitude":150},{"name":"KH7-23 Geraldino","a":2.64,"longitude":355},{"name":"1036 Ganymed","a":2.6629,"longitude":285},{"name":"3 Juno","a":2.67,"longitude":350},{"name":"FL8-09 Varsoviom","a":2.75,"longitude":255},{"name":"1 Ceres","a":2.7679725,"longitude":0},{"name":"2 Pallas","a":2.7696,"longitude":357},{"name":"267 Tirza","a":2.77458,"longitude":305},{"name":"PC0-01 Kurai","a":2.801,"longitude":5},{"name":"PW4-13 Rider","a":3.01,"longitude":77},{"name":"368 Haidea","a":3.07,"longitude":305},{"name":"10 Hygiea","a":3.1415,"longitude":15},{"name":"AB2-38 Dover","a":3.22,"longitude":1},{"name":"BG1-65 Usher","a":3.33,"longitude":333},{"name":"MP3-87 Nosfer","a":3.45,"longitude":123},{"name":"TT-9025","a":3.54,"longitude":84},{"name":"ZZ9-01 Nebulavsky","a":3.67,"longitude":245},{"name":"4P Faye","a":3.838159,"longitude":0},{"name":"KB5-98 Kris","a":3.91,"longitude":91},{"name":"2312 Duboshin","a":3.97,"longitude":305},{"name":"DE8-42 Sunset","a":4,"longitude":334},{"name":"279 Thule","a":4.26,"longitude":111},{"name":"659 Nestor","a":5.1702,"longitude":288.33},{"name":"Jupiter","a":5.202887,"longitude":34.396442},{"name":"617 Patroclus","a":5.209,"longitude":323.7},{"name":"588 Achilles","a":5.2091,"longitude":205.11},{"name":"1172 Aneas","a":5.2182,"longitude":238.69},{"name":"3317 Paris","a":5.2223,"longitude":287.65},{"name":"624 Hektor","a":5.2571,"longitude":128.09},{"name":"911 Agamemnon","a":5.2766,"longitude":136.09},{"name":"Saturn","a":9.536676,"longitude":49.954243},{"name":"1P Halley","a":17.83416,"longitude":0},{"name":"Uranus","a":19.18917,"longitude":313.2381},{"name":"Neptune","a":30.06992,"longitude":0},{"name":"Pluto","a":39.48212,"longitude":238.92903}];
window.LAUNCH_WINDOW_EARTH = {"a":1.000001,"longitude":168};
</script>
<script src="{{ '/assets/js/launch-windows.js' | relative_url }}"></script>

## See also

- [Planets](planets.md)
- [Celestial Bodies overview](README.md)

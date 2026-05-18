# Initial habitability per scenario

Every Solar Expanse scenario ships with a pre-built save that pins every
body's starting environmental state. This page lays those values out side
by side so the four start dates can be compared at a glance — useful for
spotting how much of Mars's water has already been delivered by the time
the *Colonization Era* scenario opens, or how Venus's temperature has
budged across the timeline.

**Reading the tables.** Each row is one scenario. Values are pulled
directly from the StartGameData's `ObjectInfoSaves[].habitabilityParameters`
block — the same values the game reads at scenario load. Units (inferred
from the well-known planets):

| Column | Unit |
| --- | --- |
| Temperature | °C |
| Pressure | Earth atmospheres |
| Gravity | m/s² |
| Water | 0–1 fraction |
| Radiation | game-specific scale (Earth ≈ 1) |
| Magnetic field | game-specific scale (Earth ≈ 40) |
| Albedo | 0–1 surface reflectivity |
| Composition | 0–1 atmospheric composition score |
| Day–night ΔT | °C |

*Note: the Early Exploration save (testStartGAme in the dump) doesn't carry
a populated habitabilityParameters block, so its row reads "—" across the
board.*

## Mercury

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 233.7 | 0 | 3.70 | 0 | 30.00 | 15.00 | 0.09 | 0 | 960.9 |
| **Colonization Era** | 233.7 | 0 | 3.70 | 0 | 30.00 | 15.00 | 0.09 | 0 | 960.9 |
| **Race Beyond** | 233.7 | 0 | 3.70 | 0 | 30.00 | 15.00 | 0.09 | 0 | 960.9 |

## Venus

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 260.8 | 92.215 | 8.86 | 0 | 0 | 10.00 | 0.76 | 0 | 14.2 |
| **Colonization Era** | 260.8 | 92.215 | 8.86 | 0 | 0 | 10.00 | 0.76 | 0 | 95.5 |
| **Race Beyond** | 260.8 | 92.215 | 8.86 | 0 | 0 | 10.00 | 0.76 | 0 | 95.5 |

## Earth

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 2.4 | 1.001 | 9.79 | 0.985 | 1.00 | 40.00 | 0.29 | 0.204 | 23.7 |
| **Colonization Era** | 2.4 | 0.998 | 9.79 | 0.941 | 1.00 | 40.00 | 0.29 | 0.204 | 23.7 |
| **Race Beyond** | 2.2 | 0.989 | 9.79 | 0.790 | 1.00 | 40.00 | 0.29 | 0.205 | 23.7 |

## Moon

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 40.5 | 0 | 1.62 | 0 | 12.00 | 2.50 | 0.11 | 0 | 261.9 |
| **Colonization Era** | 40.5 | 0 | 1.62 | 0 | 12.00 | 2.50 | 0.11 | 0 | 261.9 |
| **Race Beyond** | 40.5 | 0 | 1.62 | 0 | 12.00 | 2.50 | 0.11 | 0 | 261.9 |

## Mars

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -63.3 | 0.006 | 3.71 | 0 | 10.00 | 10.00 | 0.25 | 0.001 | 80.7 |
| **Colonization Era** | -63.3 | 0.006 | 3.71 | 0 | 10.00 | 10.00 | 0.25 | 0.001 | 209.0 |
| **Race Beyond** | -63.3 | 0.006 | 3.71 | 0 | 10.00 | 10.00 | 0.25 | 0.002 | 209.3 |

## Phobos

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.01 | 0 | 10.00 | 0 | 0.07 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.01 | 0 | 10.00 | 0 | 0.07 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.01 | 0 | 10.00 | 0 | 0.07 | 0 | 0 |

## Deimos

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -16.4 | 0.000 | 0.24 | 0 | 10.00 | 0 | 0.07 | 0 | 81.0 |
| **Colonization Era** | -16.4 | 0.000 | 0.24 | 0 | 10.00 | 0 | 0.07 | 0 | 81.0 |
| **Race Beyond** | -16.3 | 0.000 | 0.24 | 0 | 10.00 | 0 | 0.07 | 0 | 81.0 |

## Jupiter

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 3962.0 | 7157325.554 | 24.77 | 0.012 | 1500.00 | 1000.00 | 0.34 | 0 | 0.0 |
| **Colonization Era** | 3962.9 | 7163198.054 | 24.77 | 0.012 | 1500.00 | 1000.00 | 0.34 | 0 | 0.0 |
| **Race Beyond** | 3962.9 | 7163198.054 | 24.77 | 0.012 | 1500.00 | 1000.00 | 0.34 | 0 | 0.0 |

## Amalthea

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.01 | 0 | 150.00 | 0 | 0.08 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.01 | 0 | 150.00 | 0 | 0.08 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.01 | 0 | 150.00 | 0 | 0.08 | 0 | 0 |

## Io

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -163.7 | 0 | 1.80 | 0 | 1000.00 | 2.50 | 0.63 | 0 | 34.4 |
| **Colonization Era** | -163.7 | 0 | 1.80 | 0 | 1000.00 | 2.50 | 0.63 | 0 | 34.4 |
| **Race Beyond** | -163.7 | 0 | 1.80 | 0 | 1000.00 | 2.50 | 0.63 | 0 | 34.4 |

## Europa

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -166.1 | 0 | 1.31 | 0 | 150.00 | 2.50 | 0.67 | 0 | 34.4 |
| **Colonization Era** | -166.1 | 0 | 1.31 | 0 | 150.00 | 2.50 | 0.67 | 0 | 34.4 |
| **Race Beyond** | -166.1 | 0 | 1.31 | 0 | 150.00 | 2.50 | 0.67 | 0 | 34.4 |

## Ganymede

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -150.3 | 0 | 1.43 | 0 | 30.00 | 25.00 | 0.43 | 0 | 34.4 |
| **Colonization Era** | -150.3 | 0 | 1.43 | 0 | 30.00 | 25.00 | 0.43 | 0 | 34.4 |
| **Race Beyond** | -150.3 | 0 | 1.43 | 0 | 30.00 | 25.00 | 0.43 | 0 | 34.4 |

## Callisto

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -139.4 | 0 | 1.23 | 0 | 15.00 | 2.50 | 0.20 | 0 | 43.8 |
| **Colonization Era** | -139.4 | 0 | 1.23 | 0 | 15.00 | 2.50 | 0.20 | 0 | 43.8 |
| **Race Beyond** | -139.4 | 0 | 1.23 | 0 | 15.00 | 2.50 | 0.20 | 0 | 43.8 |

## Saturn

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 1663.5 | 1051486.132 | 10.43 | 0 | 900.00 | 500.00 | 0.34 | 0 | 0.0 |
| **Colonization Era** | 1663.5 | 1051486.132 | 10.43 | 0 | 900.00 | 500.00 | 0.34 | 0 | 0.0 |
| **Race Beyond** | 1663.5 | 1051486.132 | 10.43 | 0 | 900.00 | 500.00 | 0.34 | 0 | 0.0 |

## Titan

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -178.9 | 1.485 | 1.35 | 0 | 1.00 | 2.50 | 0.21 | 0 | 7.4 |
| **Colonization Era** | -178.9 | 1.485 | 1.35 | 0 | 1.00 | 2.50 | 0.21 | 0 | 7.4 |
| **Race Beyond** | -178.9 | 1.485 | 1.35 | 0 | 1.00 | 2.50 | 0.21 | 0 | 7.4 |

## Enceladus

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -215.6 | 0 | 0.11 | 0 | 12.00 | 2.50 | 0.90 | 0 | 84.8 |
| **Colonization Era** | -215.6 | 0 | 0.11 | 0 | 12.00 | 2.50 | 0.90 | 0 | 84.8 |
| **Race Beyond** | -215.6 | 0 | 0.11 | 0 | 12.00 | 2.50 | 0.90 | 0 | 84.8 |

## Rhea

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -226.0 | 0 | 0.26 | 0 | 10.00 | 2.50 | 0.95 | 0 | 84.8 |
| **Colonization Era** | -226.0 | 0 | 0.26 | 0 | 10.00 | 2.50 | 0.95 | 0 | 84.8 |
| **Race Beyond** | -226.0 | 0 | 0.26 | 0 | 10.00 | 2.50 | 0.95 | 0 | 84.8 |

## Iapetus

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -174.5 | 0 | 0.22 | 0 | 1.50 | 2.50 | 0.20 | 0 | 84.8 |
| **Colonization Era** | -174.5 | 0 | 0.22 | 0 | 1.50 | 2.50 | 0.20 | 0 | 84.8 |
| **Race Beyond** | -174.5 | 0 | 0.22 | 0 | 1.50 | 2.50 | 0.20 | 0 | 84.8 |

## Tethys

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -204.0 | 0 | 0.14 | 0 | 12.00 | 2.50 | 0.80 | 0 | 84.8 |
| **Colonization Era** | -204.0 | 0 | 0.14 | 0 | 12.00 | 2.50 | 0.80 | 0 | 84.8 |
| **Race Beyond** | -204.0 | 0 | 0.14 | 0 | 12.00 | 2.50 | 0.80 | 0 | 84.8 |

## Mimas

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.06 | 0 | 20.00 | 0 | 0.80 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.06 | 0 | 20.00 | 0 | 0.80 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.06 | 0 | 20.00 | 0 | 0.80 | 0 | 0 |

## Hyperion

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.30 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.30 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.30 | 0 | 0 |

## Dione

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -196.3 | 0 | 0.23 | 0 | 12.00 | 2.50 | 0.70 | 0 | 84.8 |
| **Colonization Era** | -196.3 | 0 | 0.23 | 0 | 12.00 | 2.50 | 0.70 | 0 | 84.8 |
| **Race Beyond** | -196.3 | 0 | 0.23 | 0 | 12.00 | 2.50 | 0.70 | 0 | 84.8 |

## Uranus

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 1053.3 | 883313.648 | 9.00 | 0.000 | 7000.00 | 80.00 | 0.30 | 0 | 0.0 |
| **Colonization Era** | 1053.3 | 883313.648 | 9.00 | 0.000 | 7000.00 | 80.00 | 0.30 | 0 | 0.0 |
| **Race Beyond** | 1053.3 | 883313.648 | 9.00 | 0.000 | 7000.00 | 80.00 | 0.30 | 0 | 0.0 |

## Ariel

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -213.3 | 0 | 0.24 | 0 | 1.50 | 2.50 | 0.53 | 0 | 59.8 |
| **Colonization Era** | -213.3 | 0 | 0.24 | 0 | 1.50 | 2.50 | 0.53 | 0 | 59.8 |
| **Race Beyond** | -213.3 | 0 | 0.24 | 0 | 1.50 | 2.50 | 0.53 | 0 | 59.8 |

## Umbriel

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -205.6 | 0 | 0.23 | 0 | 1.50 | 2.50 | 0.26 | 0 | 59.8 |
| **Colonization Era** | -205.6 | 0 | 0.23 | 0 | 1.50 | 2.50 | 0.26 | 0 | 59.8 |
| **Race Beyond** | -205.6 | 0 | 0.23 | 0 | 1.50 | 2.50 | 0.26 | 0 | 59.8 |

## Titania

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -207.9 | 0 | 0.37 | 0 | 1.50 | 2.50 | 0.35 | 0 | 59.8 |
| **Colonization Era** | -207.9 | 0 | 0.37 | 0 | 1.50 | 2.50 | 0.35 | 0 | 59.8 |
| **Race Beyond** | -207.9 | 0 | 0.37 | 0 | 1.50 | 2.50 | 0.35 | 0 | 59.8 |

## Oberon

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -206.8 | 0 | 0.35 | 0 | 1.50 | 2.50 | 0.31 | 0 | 59.8 |
| **Colonization Era** | -206.8 | 0 | 0.35 | 0 | 1.50 | 2.50 | 0.31 | 0 | 59.8 |
| **Race Beyond** | -206.8 | 0 | 0.35 | 0 | 1.50 | 2.50 | 0.31 | 0 | 59.8 |

## Puck

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.12 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.12 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.12 | 0 | 0 |

## Neptune

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 200.5 | 34754.065 | 11.27 | 0.000 | 7200.00 | 60.00 | 0.29 | 0 | 0.2 |
| **Colonization Era** | 200.5 | 34754.065 | 11.27 | 0.000 | 7200.00 | 60.00 | 0.29 | 0 | 0.2 |
| **Race Beyond** | 200.5 | 34754.065 | 11.27 | 0.000 | 7200.00 | 60.00 | 0.29 | 0 | 0.2 |

## Triton

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -273.1 | 0 | 0.78 | 0 | 1.50 | 2.50 | 0.90 | 0 | 18.3 |
| **Colonization Era** | -273.1 | 0 | 0.78 | 0 | 1.50 | 2.50 | 0.90 | 0 | 18.3 |
| **Race Beyond** | -273.1 | 0 | 0.78 | 0 | 1.50 | 2.50 | 0.90 | 0 | 18.3 |

## Proteus

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.10 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.10 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.02 | 0 | 1.50 | 0 | 0.10 | 0 | 0 |

## Nereid

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | 0 | 0 | 0.06 | 0 | 1.50 | 0 | 0.20 | 0 | 0 |
| **Colonization Era** | 0 | 0 | 0.06 | 0 | 1.50 | 0 | 0.20 | 0 | 0 |
| **Race Beyond** | 0 | 0 | 0.06 | 0 | 1.50 | 0 | 0.20 | 0 | 0 |

## Pluto

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -244.4 | 0.000 | 0.66 | 0 | 1.50 | 2.50 | 0.72 | 0 | 40.1 |
| **Colonization Era** | -244.4 | 0.000 | 0.66 | 0 | 1.50 | 2.50 | 0.72 | 0 | 40.1 |
| **Race Beyond** | -244.4 | 0.000 | 0.66 | 0 | 1.50 | 2.50 | 0.72 | 0 | 40.1 |

## Charon

| Scenario | Temp (°C) | Pressure | Gravity | Water | Radiation | Magnetic | Albedo | Composition | Day–night ΔT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Early Exploration** | — | — | — | — | — | — | — | — | — |
| **The Expansion** | -231.4 | 0 | 0.27 | 0 | 1.50 | 2.50 | 0.41 | 0 | 41.7 |
| **Colonization Era** | -231.4 | 0 | 0.27 | 0 | 1.50 | 2.50 | 0.41 | 0 | 41.7 |
| **Race Beyond** | -231.4 | 0 | 0.27 | 0 | 1.50 | 2.50 | 0.41 | 0 | 41.7 |

## See also

- [Celestial Bodies overview](README.md)
- [Planets](planets.md)
- [Moons](moons.md)

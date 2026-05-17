# Resources

Resources are produced by facilities, shipped between worlds in spacecraft
cargo holds, traded on the marketplace, and consumed in construction. Three
types exist:

- **Normal** — physical materials, the bulk of the economy.
- **Energy** — power; produced and consumed in real time, with limited storage in batteries.
- **Human** — colonists; produced over time by habitats and consumed by jobs.

| Resource | <span title="Normal (physical), Energy (real-time power), or Human (colonists)">Type</span> | <span title="Starting market clearing price; supply and demand move it from there">Price</span> | <span title="Facilities that output this resource (recipes / mines / power output / byproducts)">Producers</span> | <span title="Facilities that consume this resource as a recipe input">Consumers</span> | Description |
| --- | --- | --- | --- | --- | --- |
| **Energy** | Energy | 11.5 | Carbon Power Plant<br>Chemical Reactor<br>Fusion Reactor<br>Geothermal Power<br>Hydrogen Power Plant<br>Nuclear Reactor<br>Orbital Power Station<br>Power Plant<br>Solar Array<br>Wind Power | — | The lifeblood of modern infrastructure. Must be generated or stored continuously to prevent system failure. |
| **Humans** | Human | 200 | — | — | The driving force of expansion, they need supplies to be able to work. Their survival depends on stable life support and habitability. |
| **Alloy** | Normal | 300 | Alloy Smelting<br>Steel Extractor | Consumer Goods Factory | An engineered alloy, offering greater strength and durability than raw metal. Standard material for spacecraft. |
| **Carbon** | Normal | 800 | Carbon Mine<br>Carbon Release Station<br>Co2 Electrolysis<br>Mining Facility | Agriculture Complex<br>Carbon Power Plant<br>Carbon Release Station<br>Hydroponic Farm<br>Polymers Factory<br>Polymers Production | A versatile element forming the basis of organics and polymers. Essential for life support systems. |
| **Carbon Dioxide** | Normal | 400 | CO2 Release Station<br>Carbon Dioxide Extractor<br>Carbon Power Plant | Co2 Electrolysis | A dense atmospheric gas produced by industry and respiration, dangerous in excess. Key compound in climate control and terraforming. |
| **Chemical Fuel** | Normal | 70 | Fuel Extractor<br>Fuel Mine<br>Fuel Refinery<br>Orbital Fuel Refinery | Power Plant | A propellant combining hydrogen with oxygen. Delivers powerful thrust for deep-space travel. |
| **Electronics** | Normal | 3.2k | Electronics Extractor<br>Electronics Factory | Consumer Goods Factory | High-grade electronic components required for constructing advanced buildings and modules. |
| **Exotic Alloys** | Normal | 4k | Exotic Alloy Extractor<br>Exotic Alloy Production | Consumer Goods Factory | High-strength materials used in advanced engineering. Expensive to refine but critical for high-performance structures |
| **Fissiles** | Normal | 1k | Fissile Extraction Facility<br>Fissiles Mine<br>Uran Release Station | Exotic Alloy Production<br>Helium-3 Factory<br>Nuclear Reactor | Radioactive elements capable of sustaining nuclear fission reactions. Provide immense energy output. |
| **Glass** | Normal | 400 | Glass Extractor<br>Glass Kiln | Consumer Goods Factory | A transparent material refined from silicates, essential for habitats and farms. Cheap and abundant, but heavy. |
| **Helium-3** | Normal | 10k | HEL 3 Release Station<br>Helium-3 Extractor<br>Helium-3 Factory<br>he3mine_big | Fusion Reactor | A rare isotope used in advanced fusion systems. Extremely scarce but invaluable for late-stage power generation. |
| **Hydrogen** | Normal | 100 | Electrolysis Plant<br>Hydrogen Extractor<br>Hydrogen Release Station | Fuel Refinery<br>Hydrogen Power Plant | The lightest and most abundant element, used in fuel production. Invaluable for fusion-based propulsion. |
| **Metals** | Normal | 200 | Metal Mining Base<br>Metal Release Station<br>Mining Facility | Alloy Smelting | Refined from ore, metal enables durable construction in hostile environments. Heavy to transport but indispensable. |
| **Nitrogen** | Normal | 100 | Nitrogen Extractor<br>Nitrogen Release Station | Agriculture Complex | An inert atmospheric buffer gas required to maintain safe pressure in breathable environments. |
| **Noble Gas** | Normal | 2k | Mining Facility<br>Noble Gas Extractor<br>Noble Gas Release Station | Electronics Factory | Rare, chemically inert gases used in advanced propulsion systems. Scarce but important for electric propulsion. |
| **Oxygen** | Normal | 50 | Co2 Electrolysis<br>Electrolysis Plant<br>Oxygen Extractor<br>Oxygen Release Station | Carbon Power Plant<br>Fuel Refinery<br>Hydrogen Power Plant | A highly reactive gas essential for human survival. Used for life support and fuel oxidizers. |
| **Polymers** | Normal | 1.6k | Polymers Extractor<br>Polymers Factory<br>Polymers Production | Consumer Goods Factory | Lightweight carbon-based materials, critical for efficient spacecraft and life support systems. |
| **Rare Metals** | Normal | 800 | Mining Facility<br>Rare Metal Extractors<br>Rare Metal Release Station | Electronics Factory<br>Exotic Alloy Production | Scarce and valuable metals used in precision electronics and specialized manufacturing. |
| **Silicon** | Normal | 200 | Mining Facility<br>Silicon Mine<br>Silicon Release Station | Electronics Factory<br>Glass Kiln | A semiconductor refined from common rock. Required for glass, electronics and solar technology |
| **Supplies** | Normal | 750 | Agriculture Complex<br>Hydroponic Farm | — | Food, water and air, necessary for sustaining life on spacecraft and objects without a breathable atmosphere. |
| **Water** | Normal | 50 | Hydrogen Power Plant<br>Mining Facility<br>Water Ice Extractor | Agriculture Complex<br>Electrolysis Plant<br>Fuel Refinery<br>Hydroponic Farm<br>Orbital Fuel Refinery | The foundation of survival and growth. Used for drinking, farming, industry, and hydrogen extraction. |

## Reading the table

- **Base market price** is the starting clearing price on the marketplace; supply and demand move it from there.
- **Producers** / **Consumers** come from each facility's structured recipe data — power output, mine outputs, refiner inputs/outputs, and byproducts. Static per-day rates aren't shown (some live on dynamic subclasses); the in-game tooltip is the source of truth for rate numbers.

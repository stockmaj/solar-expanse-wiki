# Resources

Resources are produced by facilities, shipped between worlds in spacecraft
cargo holds, traded on the marketplace, and consumed in construction. Three
types exist:

- **Normal** — physical materials, the bulk of the economy.
- **Energy** — power; produced and consumed in real time, with limited storage in batteries.
- **Human** — colonists; produced over time by habitats and consumed by jobs.

| Resource | <span title="Normal (physical), Energy (real-time power), or Human (colonists)">Type</span> | <span title="Starting market clearing price; supply and demand move it from there">Price</span> | Producers | Description |
| --- | --- | --- | --- | --- |
| <a id="resource-energy"></a>**Energy** | Energy | 11.5 | [Geothermal Power](../facilities/#facility-power-geothermal) | The lifeblood of modern infrastructure. Must be generated or stored continuously to prevent system failure. |
| <a id="resource-human"></a>**Humans** | Human | 200 | — | The driving force of expansion, they need supplies to be able to work. Their survival depends on stable life support and habitability. |
| <a id="resource-steel"></a>**Alloy** | Normal | 300 | [Alloy Smelting](../facilities/#facility-alloysmelting)<br>[Exotic Alloy Extractor](../facilities/#facility-alloymine)<br>[Exotic Alloy Production](../facilities/#facility-exoticalloy) | An engineered alloy, offering greater strength and durability than raw metal. Standard material for spacecraft. |
| <a id="resource-volatile"></a>**Carbon** | Normal | 800 | [Carbon Mine](../facilities/#facility-carbonmine)<br>[Carbon Power Plant](../facilities/#facility-power-carbon)<br>[Co2 Electrolysis](../facilities/#facility-co2-splitting)<br>[Polymers Factory](../facilities/#facility-polymerproduction-big)<br>[Polymers Production](../facilities/#facility-polymerproduction) | A versatile element forming the basis of organics and polymers. Essential for life support systems. |
| <a id="resource-co2"></a>**Carbon Dioxide** | Normal | 400 | — | A dense atmospheric gas produced by industry and respiration, dangerous in excess. Key compound in climate control and terraforming. |
| <a id="resource-fuel"></a>**Chemical Fuel** | Normal | 70 | [Fuel Refinery](../facilities/#facility-fuel)<br>[Orbital Fuel Refinery](../facilities/#facility-space-fuel) | A propellant combining hydrogen with oxygen. Delivers powerful thrust for deep-space travel. |
| <a id="resource-chips"></a>**Electronics** | Normal | 3.2k | [Electronics Extractor](../facilities/#facility-electronicsmine)<br>[Electronics Factory](../facilities/#facility-electronicsfactory) | High-grade electronic components required for constructing advanced buildings and modules. |
| <a id="resource-alloy"></a>**Exotic Alloys** | Normal | 4k | [Exotic Alloy Extractor](../facilities/#facility-alloymine)<br>[Exotic Alloy Production](../facilities/#facility-exoticalloy) | High-strength materials used in advanced engineering. Expensive to refine but critical for high-performance structures |
| <a id="resource-uran"></a>**Fissiles** | Normal | 1k | [Exotic Alloy Production](../facilities/#facility-exoticalloy) | Radioactive elements capable of sustaining nuclear fission reactions. Provide immense energy output. |
| <a id="resource-glass"></a>**Glass** | Normal | 400 | [Glass Extractor](../facilities/#facility-glassmine)<br>[Glass Kiln](../facilities/#facility-glass) | A transparent material refined from silicates, essential for habitats and farms. Cheap and abundant, but heavy. |
| <a id="resource-hel3"></a>**Helium-3** | Normal | 10k | [Helium-3 Extractor](../facilities/#facility-he3mine) | A rare isotope used in advanced fusion systems. Extremely scarce but invaluable for late-stage power generation. |
| <a id="resource-hydrogen"></a>**Hydrogen** | Normal | 100 | [Electrolysis Plant](../facilities/#facility-electrolysis)<br>[Fuel Refinery](../facilities/#facility-fuel-big)<br>[Hydrogen Extractor](../facilities/#facility-hydrogenmine)<br>[Hydrogen Power Plant](../facilities/#facility-power-hydrogen) | The lightest and most abundant element, used in fuel production. Invaluable for fusion-based propulsion. |
| <a id="resource-metal"></a>**Metals** | Normal | 200 | [Exotic Alloy Production](../facilities/#facility-exoticalloy) | Refined from ore, metal enables durable construction in hostile environments. Heavy to transport but indispensable. |
| <a id="resource-nitrogen"></a>**Nitrogen** | Normal | 100 | [Nitrogen Extractor](../facilities/#facility-nitrogenmine) | An inert atmospheric buffer gas required to maintain safe pressure in breathable environments. |
| <a id="resource-noblegas"></a>**Noble Gas** | Normal | 2k | — | Rare, chemically inert gases used in advanced propulsion systems. Scarce but important for electric propulsion. |
| <a id="resource-oxygen"></a>**Oxygen** | Normal | 50 | [Co2 Electrolysis](../facilities/#facility-co2-splitting)<br>[Electrolysis Plant](../facilities/#facility-electrolysis)<br>[Fuel Refinery](../facilities/#facility-fuel-big)<br>[Oxygen Extractor](../facilities/#facility-oxygenmine) | A highly reactive gas essential for human survival. Used for life support and fuel oxidizers. |
| <a id="resource-plastic"></a>**Polymers** | Normal | 1.6k | [Polymers Extractor](../facilities/#facility-plasticmine)<br>[Polymers Factory](../facilities/#facility-polymerproduction-big)<br>[Polymers Production](../facilities/#facility-polymerproduction) | Lightweight carbon-based materials, critical for efficient spacecraft and life support systems. |
| <a id="resource-raremetal"></a>**Rare Metals** | Normal | 800 | [Exotic Alloy Production](../facilities/#facility-exoticalloy) | Scarce and valuable metals used in precision electronics and specialized manufacturing. |
| <a id="resource-silicon"></a>**Silicon** | Normal | 200 | [Glass Kiln](../facilities/#facility-glass)<br>[Silicon Mine](../facilities/#facility-siliconmine) | A semiconductor refined from common rock. Required for glass, electronics and solar technology |
| <a id="resource-supply"></a>**Supplies** | Normal | 750 | — | Food, water and air, necessary for sustaining life on spacecraft and objects without a breathable atmosphere. |
| <a id="resource-water"></a>**Water** | Normal | 50 | [Electrolysis Plant](../facilities/#facility-electrolysis)<br>[Fuel Refinery](../facilities/#facility-fuel)<br>[Orbital Fuel Refinery](../facilities/#facility-space-fuel)<br>[Water Ice Extractor](../facilities/#facility-icemine) | The foundation of survival and growth. Used for drinking, farming, industry, and hydrogen extraction. |

## Reading the table

- **Base market price** is the starting clearing price on the marketplace; supply and demand move it from there.
- **Produced by** is inferred from facility tooltip text — if a facility's description mentions the resource by name, it's listed. The actual produce-rate isn't extractable from the static descriptors (it lives on dynamic facility subclasses); the in-game tooltip is the source of truth for rate numbers.

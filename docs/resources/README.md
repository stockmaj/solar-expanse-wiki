# Resources

Resources are produced by facilities, shipped between worlds in spacecraft
cargo holds, traded on the marketplace, and consumed in construction. Three
types exist:

- **Normal** — physical materials, the bulk of the economy.
- **Energy** — power; produced and consumed in real time, with limited storage in batteries.
- **Human** — colonists; produced over time by habitats and consumed by jobs.

| Resource | Type | Base market price | Produced by | Description |
| --- | --- | --- | --- | --- |
| **Energy** | Energy | ₡11.5 | Geothermal Power | The lifeblood of modern infrastructure. Must be generated or stored continuously to prevent system failure. |
| **Humans** | Human | ₡200 | — | The driving force of expansion, they need supplies to be able to work. Their survival depends on stable life support and habitability. |
| **Alloy** | Normal | ₡300 | Alloy Smelting<br>Exotic Alloy Extractor<br>Exotic Alloy Production | An engineered alloy, offering greater strength and durability than raw metal. Standard material for spacecraft. |
| **Carbon** | Normal | ₡800 | Carbon Mine<br>Carbon Power Plant<br>Co2 Electrolysis<br>Polymers Factory<br>Polymers Production | A versatile element forming the basis of organics and polymers. Essential for life support systems. |
| **Carbon Dioxide** | Normal | ₡400 | — | A dense atmospheric gas produced by industry and respiration, dangerous in excess. Key compound in climate control and terraforming. |
| **Chemical Fuel** | Normal | ₡70 | Fuel Refinery<br>Orbital Fuel Refinery | A propellant combining hydrogen with oxygen. Delivers powerful thrust for deep-space travel. |
| **Electronics** | Normal | ₡3250 | Electronics Extractor<br>Electronics Factory | High-grade electronic components required for constructing advanced buildings and modules. |
| **Exotic Alloys** | Normal | ₡4000 | Exotic Alloy Extractor<br>Exotic Alloy Production | High-strength materials used in advanced engineering. Expensive to refine but critical for high-performance structures |
| **Fissiles** | Normal | ₡1000 | Exotic Alloy Production | Radioactive elements capable of sustaining nuclear fission reactions. Provide immense energy output. |
| **Glass** | Normal | ₡400 | Glass Extractor<br>Glass Kiln | A transparent material refined from silicates, essential for habitats and farms. Cheap and abundant, but heavy. |
| **Helium-3** | Normal | ₡10000 | Helium-3 Extractor | A rare isotope used in advanced fusion systems. Extremely scarce but invaluable for late-stage power generation. |
| **Hydrogen** | Normal | ₡100 | Electrolysis Plant<br>Fuel Refinery<br>Hydrogen Extractor<br>Hydrogen Power Plant | The lightest and most abundant element, used in fuel production. Invaluable for fusion-based propulsion. |
| **Metals** | Normal | ₡200 | Exotic Alloy Production | Refined from ore, metal enables durable construction in hostile environments. Heavy to transport but indispensable. |
| **Nitrogen** | Normal | ₡100 | Nitrogen Extractor | An inert atmospheric buffer gas required to maintain safe pressure in breathable environments. |
| **Noble Gas** | Normal | ₡2000 | — | Rare, chemically inert gases used in advanced propulsion systems. Scarce but important for electric propulsion. |
| **Oxygen** | Normal | ₡50 | Co2 Electrolysis<br>Electrolysis Plant<br>Fuel Refinery<br>Oxygen Extractor | A highly reactive gas essential for human survival. Used for life support and fuel oxidizers. |
| **Polymers** | Normal | ₡1600 | Polymers Extractor<br>Polymers Factory<br>Polymers Production | Lightweight carbon-based materials, critical for efficient spacecraft and life support systems. |
| **Rare Metals** | Normal | ₡800 | Exotic Alloy Production | Scarce and valuable metals used in precision electronics and specialized manufacturing. |
| **Silicon** | Normal | ₡200 | Glass Kiln<br>Silicon Mine | A semiconductor refined from common rock. Required for glass, electronics and solar technology |
| **Supplies** | Normal | ₡750 | — | Food, water and air, necessary for sustaining life on spacecraft and objects without a breathable atmosphere. |
| **Water** | Normal | ₡50 | Electrolysis Plant<br>Fuel Refinery<br>Orbital Fuel Refinery<br>Water Ice Extractor | The foundation of survival and growth. Used for drinking, farming, industry, and hydrogen extraction. |

## Reading the table

- **Base market price** is the starting clearing price on the marketplace; supply and demand move it from there.
- **Produced by** is inferred from facility tooltip text — if a facility's description mentions the resource by name, it's listed. The actual produce-rate isn't extractable from the static descriptors (it lives on dynamic facility subclasses); the in-game tooltip is the source of truth for rate numbers.

# Terraforming

Solar Expanse simulates planetary atmospheres and surface conditions based on
per-resource thermal properties. Use these tables to understand:

- which resources will vaporize or freeze at a given temperature
- how heat capacity drives atmospheric warming and cooling
- which resources contribute to greenhouse warming (optical depth) and surface heating

## Resource thermal properties

| Resource | <span title="Phase-change temperature where the resource transitions between solid and liquid. The body's surface temperature must cross this for the resource to melt or freeze.">Melting (K / °C)</span> | <span title="Phase-change temperature where the resource transitions between liquid and gas at reference pressure. Crossing this is what gets a species into (or out of) the atmosphere.">Boiling (K / °C)</span> | <span title="Energy required to vaporize one mole of the resource. Drives how strongly evaporation cools the surface and condensation warms it.">Latent heat (J/mol)</span> | <span title="Specific heat — how much energy the resource absorbs before warming. High values smooth out temperature swings.">Heat capacity (J/(kg·K))</span> | <span title="Greenhouse strength: dimensionless coefficient (formerly `gasIRAbsorbtionCoefficient`) that scales how much outgoing infrared the gas traps.">Optical depth</span> | <span title="Triple-point pressure: the minimum atmospheric pressure for a stable liquid phase. Below this, the resource sublimates directly between solid and gas.">Triple-point pressure (atm)</span> |
| --- | --- | --- | --- | --- | --- | --- |
| <a id="terraforming-steel"></a><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;**[Alloy](../resources/#resource-steel)** | 1723 / 1450 °C | 3400 / 3127 °C | 340000 | 490 | 1 | 1 |
| <a id="terraforming-volatile"></a><img src="../images/resources/volatile.png" width="16" alt="Carbon"/>&nbsp;**[Carbon](../resources/#resource-volatile)** | 3925 / 3652 °C | 4188 / 3915 °C | 710000 | 710 | 1 | 0.99 |
| <a id="terraforming-co2"></a><img src="../images/resources/co2.png" width="16" alt="Carbon Dioxide"/>&nbsp;**[Carbon Dioxide](../resources/#resource-co2)** | 216 / -57 °C | 217 / -56 °C | 25200 | 844 | 0.04 | 5.11 |
| <a id="terraforming-fuel"></a><img src="../images/resources/fuel.png" width="16" alt="Chemical Fuel"/>&nbsp;**[Chemical Fuel](../resources/#resource-fuel)** | 65 / -208 °C | 88 / -185 °C | 8190 | 1000 | 0.0001 | 0.15 |
| <a id="terraforming-chips"></a><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;**[Electronics](../resources/#resource-chips)** | 1537 / 1264 °C | 3330 / 3057 °C | 382000 | 440 | 1 | 1 |
| <a id="terraforming-alloy"></a><img src="../images/resources/alloy.png" width="16" alt="Exotic Alloys"/>&nbsp;**[Exotic Alloys](../resources/#resource-alloy)** | 1609 / 1336 °C | 3120 / 2847 °C | 370000 | 440 | 1 | 1 |
| <a id="terraforming-uran"></a><img src="../images/resources/uran.png" width="16" alt="Fissiles"/>&nbsp;**[Fissiles](../resources/#resource-uran)** | 1405 / 1132 °C | 4404 / 4131 °C | 417000 | 116 | 1 | 1.0e-5 |
| <a id="terraforming-glass"></a><img src="../images/resources/glass.png" width="16" alt="Glass"/>&nbsp;**[Glass](../resources/#resource-glass)** | 1500 / 1227 °C | 2300 / 2027 °C | 35000 | 800 | 1 | 1 |
| <a id="terraforming-hel3"></a><img src="../images/resources/hel3.png" width="16" alt="Helium-3"/>&nbsp;**[Helium-3](../resources/#resource-hel3)** | 0.3 / -273 °C | 3.2 / -270 °C | 26 | 1 | 0.0001 | 0.001 |
| <a id="terraforming-hydrogen"></a><img src="../images/resources/hydrogen.png" width="16" alt="Hydrogen"/>&nbsp;**[Hydrogen](../resources/#resource-hydrogen)** | 14 / -259 °C | 20 / -253 °C | 449 | 14320 | 0.0001 | 0.0695 |
| <a id="terraforming-metal"></a><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;**[Metals](../resources/#resource-metal)** | 1811 / 1538 °C | 3135 / 2862 °C | 340000 | 449 | 1 | 1.0e-7 |
| <a id="terraforming-nitrogen"></a><img src="../images/resources/nitrogen.png" width="16" alt="Nitrogen"/>&nbsp;**[Nitrogen](../resources/#resource-nitrogen)** | 63 / -210 °C | 77 / -196 °C | 5560 | 1040 | 1.0e-6 | 0.123 |
| <a id="terraforming-noblegas"></a><img src="../images/resources/noblegas.png" width="16" alt="Noble Gas"/>&nbsp;**[Noble Gas](../resources/#resource-noblegas)** | 83 / -190 °C | 87 / -186 °C | 6430 | 312 | 1 | 0.681 |
| <a id="terraforming-oxygen"></a><img src="../images/resources/oxygen.png" width="16" alt="Oxygen"/>&nbsp;**[Oxygen](../resources/#resource-oxygen)** | 54 / -219 °C | 90 / -183 °C | 6820 | 918 | 1.0e-6 | 0.15 |
| <a id="terraforming-plastic"></a><img src="../images/resources/plastic.png" width="16" alt="Polymers"/>&nbsp;**[Polymers](../resources/#resource-plastic)** | 403 / 130 °C | 620 / 347 °C | 12000 | 1 | 1 | 1 |
| <a id="terraforming-raremetal"></a><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;**[Rare Metals](../resources/#resource-raremetal)** | 1337 / 1064 °C | 3130 / 2857 °C | 342000 | 129 | 1 | 1 |
| <a id="terraforming-silicon"></a><img src="../images/resources/silicon.png" width="16" alt="Silicon"/>&nbsp;**[Silicon](../resources/#resource-silicon)** | 1687 / 1414 °C | 3538 / 3265 °C | 359000 | 705 | 1 | 1.0e-5 |
| <a id="terraforming-water"></a><img src="../images/resources/water.png" width="16" alt="Water"/>&nbsp;**[Water](../resources/#resource-water)** | 220 / -53 °C | 373 / 100 °C | 50000 | 1860 | 0.002 | 0.00611 |

## Reading the table

- **Melting / Boiling** are the phase-change temperatures the body's average surface temperature must cross to keep the resource solid, liquid, or gas at reference pressure. Both columns show kelvin first with the celsius equivalent in parentheses.
- **Latent heat (J/mol)** is the energy required to vaporize one mole of the resource. It drives how strongly evaporation cools the planet's surface and how strongly condensation warms it — the same constant feeds the Clausius-Clapeyron formula the sim uses to compute saturation pressures from temperature.
- **Heat capacity (J/(kg·K))** is how much energy the resource absorbs before its temperature rises. High values smooth out day/night and seasonal temperature swings, so atmospheres dominated by high-Cp species are stabler.
- **Optical depth** is the dimensionless greenhouse contribution. Higher values trap more outgoing infrared radiation — atmospheres dominated by high-optical-depth species (CO2, water vapor) warm.
- **Triple-point pressure (atm)** is the minimum atmospheric pressure at which a stable liquid phase exists. Below this, the resource sublimates directly between solid and gas (think Mars-pressure CO2 frost).

## See also

- [Resources](../resources/) — per-resource production / consumption, market prices, and Earth licensing fees.

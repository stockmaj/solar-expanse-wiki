# Research

The tech tree drives progression. Every research node has a work-hours cost,
zero or more prerequisite research nodes, and unlocks something — a new
facility, spacecraft, launch vehicle, or a numeric bonus on existing
equipment. Research is grouped into three top-level branches (Engineering,
Physics, Biotech), each subdivided into focused sub-branches.

## Biotech

### Agriculture

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-agriculture"></a>**Agriculture** | 288k | — | — | The growth of food and plants outside of Earth presents new challenges to be solved. |

### Biotech

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-biotech"></a>**Biotech** | 288k | — | — | Application of organisms and biology in technological contexts |
| <a id="research-research-agriculture-1"></a>**Space Farming** | 1.2M | [Biotech](#research-research-category-biotech) | Builds [**Hydroponic Farm**](../facilities/#facility-farm) | Although small growth experiments have been conducted in 0g, there are still adaptations needed to allow large scale agriculture in extraterrestrial environments. Usage of appropriate aggregate medium, artificial lighting, radiation shielding, and local power sourcing could allow colonies to feed themselves. |
| <a id="research-research-biotech-1"></a>**Bioreactors** | 1.2M | [Biotech](#research-research-category-biotech) | +10 ProductionEfficiency on build_farm | Systems for containing biologically active processes that can allow us to utilize many useful organisms in a controlled manner. |
| <a id="research-research-agriculture-2"></a>**Regolith Adaptation** | 2.2M | [Space Farming](#research-research-agriculture-1) | +25 ProductionEfficiency on build_farm | Adapting local regolith into soil or aggregate, greatly expands available growth mediums, allowing more crops to be planted. |
| <a id="research-research-biotech-2"></a>**Knallgas Bacteria Farms** | 2.2M | — | — | Hydrogen-oxidizing bacteria that can use hydrogen, oxygen, and carbon dioxide as nutrition, producing primarily water that can be recycled back into hydrogen and oxygen. Knallgas bacteria farms can provide protein in a space in a resource-efficient way. |
| <a id="research-research-biotech-4"></a>**Genetic Engineering** | 2.2M | [Bioreactors](#research-research-biotech-1) | +15 LifeSupportConsumption on All | We can speed up the slow process of selective breeding by directly finding and implanting beneficial genes, improving crop yields, resistances to disease, and many others. |
| <a id="research-research-agriculture-3"></a>**Aeroponics** | 3.0M | [Regolith Adaptation](#research-research-agriculture-2) | +25 ProductionEfficiency on build_farm | An advanced form of hydroponics, with plant roots being exposed to a mist of nutrient-rich water solution. Growth of food in the absence of soil or aggregate can provide great improvements, particularly in areas of water usage, and nutrition. |
| <a id="research-research-agriculture-4"></a>**Radiotrophic Fungi** | 3.0M | — | — | Fungi capable of using radiation as an energy source have been previously found on sites of nuclear disasters. Using adapted local soil, they could be used as self-growing radiation shielding for planetary habitats. |
| <a id="research-research-biotech-3"></a>**Closed-cycle Algae** | 3.0M | — | — | Tanks of algae are utilized to recycle biological waste and CO2 back into usable nutrition and oxygen, creating a closed ecosystem within the ship. |
| <a id="research-research-biotech-5"></a>**Biological Machines** | 3.0M | [Genetic Engineering](#research-research-biotech-4) | +10 ProductionEfficiency on build_farm | Engineered assemblies of molecules and proteins can be designed to perform tasks at nanoscale. |
| <a id="research-research-agriculture-5"></a>**Low-G GMO** | 4.1M | — | — | Through the use of genetic engineering, we can adapt our crops to better grow in low- or microgravity environments. |

### Colonization

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-lifesup-9"></a>**Planetary Habitats** | 1.2M | [Life Support](#research-research-category-lifesup) | Builds [**Outpost**](../facilities/#facility-outpost) | Permanent solution for extraterrestrial bases. |
| <a id="research-research-lifesup-11"></a>**Colony Construction** | 2.2M | — | — | With our experience in construction and habitation on other planets we can finally realize the dream of permanent colonies outside of Earth. |
| <a id="research-research-lifesup-10"></a>**Regolith Shielding** | 3.0M | [Planetary Habitats](#research-research-lifesup-9) | +-35 BuildCost on build_outpost, build_habitat, build_habitatdome, build_habitatcity | We can greatly reduce the need for material by utilizing local regolith as radiation shielding in our habitats. |
| <a id="research-research-lifesup-12"></a>**On-Site 3D Printing** | 3.0M | [Planetary Habitats](#research-research-lifesup-9) | +25 BuildSpeed on Facility | With laser printing robots we can greatly speed up the assembly of ground facilities. |
| <a id="research-research-lifesup-8"></a>**Spin-Grav Habitation** | 3.0M | [Planetary Habitats](#research-research-lifesup-9) | Builds [**Rotating Habitat**](../facilities/#facility-space-0gcolony) | By spinning habitat segments, artificial gravity can be created through centrifugal force. Careful tuning of distance and speed of rotation is required, to maintain a balance between strength of simulated gravity and potential nausea from rotation. |
| <a id="research-research-lifesup-13"></a>**Permanent Orbital Presence** | 4.1M | — | — | To support our expansion into space, we need to establish a permanent orbital infrastructure. |
| <a id="research-research-lifesup-14"></a>**In-Orbit Production** | 4.8M | [Spin-Grav Habitation](#research-research-lifesup-8) | +30 BuildCost on build_space_drydock | By constructing our interplanetary vehicles directly in orbit, we're free from the size and weight constraints of our launch vehicles. |

### LifeSupport

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-lifesup"></a>**Life Support** | 648k | — | — | It is essential to develop technologies that help sustain our astronauts in good health. |
| <a id="research-research-lifesup-1"></a>**Crewed Flight** | 1.2M | [Life Support](#research-research-category-lifesup) | Builds [**module_crew_compartment**](../facilities/#facility-module-crew-compartment) | Safe orbital transport of small crews. |
| <a id="research-research-lifesup-2"></a>**Improved Capsule Construction** | 2.2M | [Crewed Flight](#research-research-lifesup-1) | +-40 BuildCost on module_crew_compartment, module_crew_medium, module_crew_large | The same technologies that have helped our launch vehicles achieve reusability can be used to improve the construction of our crew transports. |
| <a id="research-research-lifesup-5"></a>**Improved Ration Packing** | 2.2M | — | — | As human space travel becomes more regular, standardization in the form of rations provided to our astronauts is needed. |
| <a id="research-research-category-robotics"></a>**Robotics** | 3.0M | [Life Support](#research-research-category-lifesup)<br>[Nanoprocessors](#research-research-computing-proc2) | +5 ReduceCrewRequirements on All | Construction and application of robotic systems. |
| <a id="research-research-lifesup-15"></a>**Artificial Hibernation** | 3.0M | [Life Support](#research-research-category-lifesup)<br>[Genetic Engineering](#research-research-biotech-4) | +35 LifeSupportConsumptionOnShip on All | Artificially inducing a state of reduced metabolic rate and bodily activity in passengers and unessential crew will significantly reduce the amount of supplies required to sustain ships during long missions. |
| <a id="research-research-lifesup-4"></a>**STO Transports** | 3.0M | [Improved Capsule Construction](#research-research-lifesup-2) | Builds [**module_crew_large**](../facilities/#facility-module-crew-large) | By adopting design principles from recently developed passenger supersonic aircraft, we can create new shuttles capable of transporting a hundred people at a time. |
| <a id="research-research-lifesup-6"></a>**Improved Atmosphere Recycling** | 3.0M | [Crewed Flight](#research-research-lifesup-1) | +25 LifeSupportConsumption on All | The Sabatier reaction can be used to extend oxygen reserves, by recycling carbon dioxide back to oxygen, using hydrogen. |
| <a id="research-research-lifesup-16"></a>**Suspended Animation** | 4.1M | [Artificial Hibernation](#research-research-lifesup-15)<br>[Biological Machines](#research-research-biotech-5) | +35 LifeSupportConsumptionOnShip on All | Enhancements in artificial hibernation technology allow putting the body in a state of near-zero biological activity while still keeping subjects alive. This suspended animation allows passengers to be kept unconscious on long missions while consuming near minimal supplies. |
| <a id="research-research-robotics-crew1"></a>**Exoskeletons** | 4.1M | [Robotics](#research-research-category-robotics)<br>[Advanced Alloying Techniques](#research-research-mat-metal3) | +5 ReduceCrewRequirements on All | Frame of servomotors, supports, and polymers that augments the user's physical capabilities. They allow workers to do more and tire less. |
| <a id="research-research-robotics-crew2"></a>**Expert Systems** | 4.8M | [Robotics](#research-research-category-robotics)<br>[Basic AI](#research-research-computing-ai3) | +10 ReduceCrewRequirements on All | Specialized Artificial Intelligence systems, equipped with knowledge and logic designed to handle all tasks within their area of expertise. |
| <a id="research-research-robotics-crew3"></a>**Cybernetics** | 4.8M | [Machine Learning](#research-research-computing-ai1)<br>[Exoskeletons](#research-research-robotics-crew1)<br>[Graphene](#research-research-mat-fibre2) | +10 ReduceCrewRequirements on All | Integration of man and machine. Robotic limbs and neural interfaces improve the capabilities of our colonists, pushing the boundaries of humanity. |

### Spacecraft

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-interstellar"></a>**Interstellar Travel** | 10.3M | [Zeus](#research-research-sc-zeus)<br>[In-Orbit Production](#research-research-lifesup-14)<br>[Diamondoids](#research-research-mat-diamondoid) | Builds [**Interstellar Vehicle Assembly**](../facilities/#facility-space-interstellarconstruction) | We shall lay down the foundations to construct a great craft that will bring us to other stars. |

### Terraforming

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-terraforming"></a>**Terraforming** | 648k | — | — | Transformation of celestial objects into an Earth-like environment |
| <a id="research-research-terraforming-2"></a>**Carbon Release** | 1.2M | [Terraforming](#research-research-category-terraforming) | Builds [**Carbon Power Plant**](../facilities/#facility-power-carbon) | We can slowly raise the temperature and pressure of our environment by processing and releasing carbon dioxide from local minerals. |
| <a id="research-research-terraforming-6"></a>**Stellar Mirror** | 2.2M | [Carbon Release](#research-research-terraforming-2) | Builds [**Orbital Mirror**](../facilities/#facility-terraform-space-mirror) | An orbital installation of highly reflective material, redirecting sunlight onto the surface of a planet to raise its temperature. |
| <a id="research-research-terraforming-magnet1"></a>**Artificial Magnetosphere** | 3.0M | [Terraforming](#research-research-category-terraforming)<br>[Superconducting Magnets](#research-research-electromag-1)<br>[Room-Temperature Superconductors](#research-research-mat-superconductor2) | Builds [**Planetary Magnetosphere Generator**](../facilities/#facility-terraform-magnet) | Huge electromagnets made of superconductors are used to mimic a planet's magnetic field, shielding the surface from harmful radiation. |
| <a id="research-research-terraforming-1"></a>**Mars Bacteria** | 27.6M | — | — | Genetically engineered bacteria, able to survive and photosynthesize in the harsh Mars environment. |
| <a id="research-research-terraforming-3"></a>**Photosynthetic Microbes** | 27.6M | — | — | Genetically engineered bacteria, able to survive and photosynthesize in the harsh Mars environment. |
| <a id="research-research-terraforming-4"></a>**Albedo Manipulation** | 27.6M | — | — | Seeding the poles with black carbon dust can increase the amount of sunlight absorbed by it, melting the carbon dioxide ice sheets. |
| <a id="research-research-terraforming-5"></a>**Comet Redirection** | 55.3M | — | — | By redirecting comets full of volatiles, we can increase atmospheric pressure and the amount of water present on a planet. |

## Engineering

### Chemical

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-chem"></a>**Chemical Propulsion** | 288k | — | — | Powerful but inefficient, chemical engines powered the first rockets, and remain the main propulsion for launch vehicles. |
| <a id="research-research-chem-main1"></a>**Solid Propellant Rockets** | 1.2M | [Chemical Propulsion](#research-research-category-chem) | — | The simplest form of propulsion, a solid fuel is ignited and directed out of the exhaust to catch fire. While high in thrust and simple in design, the length of the burn cannot be controlled and the thruster will keep firing until out of fuel. |
| <a id="research-research-chem-main2"></a>**Kerolox** | 2.2M | [Solid Propellant Rockets](#research-research-chem-main1) | +5 ComponentExhaustV on eng_chem | Liquid oxygen and a form of highly refined kerosene - rocket fuel - provide the best performance without going fully cryogenic. |
| <a id="research-research-chem-main3"></a>**Methalox** | 3.0M | [Kerolox](#research-research-chem-main2) | +6 ComponentExhaustV on eng_chem | Combustion of liquid methane and liquid oxygen provides improved specific impulses over more traditional propellants while being the easiest to store cryogen. |
| <a id="research-research-chem-main4"></a>**Hydrolox** | 4.1M | [Methalox](#research-research-chem-main3) | +20 ComponentExhaustV on eng_chem, eng_chemsmall, eng_chemhelios, eng_chemorion | Liquid hydrogen and liquid oxygen are combusted, producing best-in-class specific impulses. While harder to store than other fuels, it burns cleanly into water vapor and can be produced from simple water electrolysis. |
| <a id="research-research-chem-main5"></a>**Solid Hydrogen** | 4.8M | [Hydrolox](#research-research-chem-main4) | +25 ComponentExhaustV on eng_chem, eng_chemsmall, eng_chemhelios, eng_chemorion | Highly pressurized and cooled hydrogen can condense into a solid. This increases density and allows to suspend free radical hydrogen within the ice, improving exhaust velocity. |
| <a id="research-research-chem-main6"></a>**Polynitrogen Fuel** | 5.5M | [Solid Hydrogen](#research-research-chem-main5) | +20 LaunchCost on LV | Polynitrogens are highly energetic molecules composed entirely of nitrogen atoms. While the exhaust velocity improvements over hydrogen vary, their over 30 times greater density allows much smaller and more compact launch vehicles, reducing the mass of propellant tanks and drag losses. |

### Colonization

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-mine-atmoscoop"></a>**Orbital Gas Extractor** | 5.5M | [In-Orbit Production](#research-research-lifesup-14) | Builds [**Orbital Gas Extractor**](../facilities/#facility-space-atmoscoop) | Orbital mining station for extracting gases from atmospheres, including gas giants. |

### Electromagnetism

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-mat-upgr3"></a>**Metamaterials** | 2.2M | — | +0 PowerProduction | Composite materials composed in a way that allows them to have properties not normally observed in naturally occurring elements. |

### LaunchFacility

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-launch"></a>**Launch Facility** | 648k | — | — | Launch infrastructure that can substantially help with reaching orbit. |
| <a id="research-research-launch-pad"></a>**Launch Pad** | 1.2M | [Launch Facility](#research-research-category-launch) | Builds [**Launch Pad**](../facilities/#facility-launch-pad) | Proper launch infrastructure can make our landings easier and safer. |
| <a id="research-research-launch-spin"></a>**Spin-Launch** | 2.2M | [Launch Pad](#research-research-launch-pad) | Builds [**Rotary Launcher**](../facilities/#facility-launch-spin) | A light payload is greatly accelerated in a centrifuge, then boosted to the final orbit by a small chemical engine. |
| <a id="research-research-launch-magrail"></a>**Magnetic Launch Rails** | 3.0M | [Spin-Launch](#research-research-launch-spin)<br>[Superconducting Magnets](#research-research-electromag-1) | Builds [**Magnetic Launch Rails**](../facilities/#facility-launch-magrails) | Using the technology pioneered by maglev trains, the launch vehicle is accelerated and launched from a track built on a mountain slope. |
| <a id="research-research-launch-massdriver"></a>**Mass Driver** | 3.0M | [Spin-Launch](#research-research-launch-spin)<br>[Superconducting Magnets](#research-research-electromag-1) | Builds [**Stationary Mass Driver**](../facilities/#facility-launch-massdriver) | Magnetic coils accelerate the payload. |
| <a id="research-research-launch-magnetic-catapult"></a>**Electromagnetic Catapult** | 4.1M | [Mass Driver](#research-research-launch-massdriver)<br>[Room-Temperature Superconductors](#research-research-mat-superconductor2) | Builds [**Electromagnetic Catapult**](../facilities/#facility-launch-magnetic-catapult) | Larger and more powerful mass driver capable of launching payloads on interplanetary trajectories by itself. |
| <a id="research-research-launch-elevator"></a>**Space Elevator** | 5.6M | [Magnetic Launch Rails](#research-research-launch-magrail)<br>[Nanotubes](#research-research-mat-fibre3) | Builds [**Space Elevator**](../facilities/#facility-launch-elevator) | Supermaterial tether reaching synchronous orbit, with an elevator able to traverse the tether and lift payloads straight to orbital height. |
| <a id="research-research-launch-massengine"></a>**Mass Driver Propulsion** | 6.5M | [Mass Driver](#research-research-launch-massdriver)<br>[Fusion Propulsion](#research-research-category-fusionprop) | Builds [**Asteroid Engine**](../facilities/#facility-asteroid-engine-module) | Crushed regolith is placed within buckets and accelerated out to 10 km/s down a mass driver. The regolith leaves the engine, transferring its momentum, while the bucket is decelerated to be reused. |

### LaunchVehicle

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-lv"></a>**Launch Vehicles** | 648k | [Chemical Propulsion](#research-research-category-chem) | — | The primary solution for surface-to-orbit transport. |
| <a id="research-research-lv-alice"></a>**Aluminium-Ice Rockets** | 864k | [Reusable Launch Vehicles](#research-research-lv-main3) | Launch Vehicle: [**Al-Ice Rocket**](../launch-vehicles/#lv-id-Rocket-RocketType5) | Microscopic particles of aluminium suspended in ice. Once ignited, the aluminium reacts with oxygen from the ice, creating heat, while the liberated hydrogen helps with burn efficiency. While not powerful or efficient enough for Earth, this type of rocket can easily be assembled on-site and does not require extensive cooling that typical cryogenic propellants need. |
| <a id="research-research-lv-main1"></a>**Early Launch Systems** | 3.0M | [Launch Vehicles](#research-research-category-lv)<br>[Kerolox](#research-research-chem-main2) | Launch Vehicle: [**Hawk**](../launch-vehicles/#lv-id-Rocket-RocketType7) | Early rockets consisted primarily of solid propellant boosters and kerosene/liquid oxygen launch vehicles. |
| <a id="research-research-lv-main2"></a>**Superheavy Launch** | 4.1M | — | — | To truly reach out into space we need ever greater capacity to lift entire spaceships into orbit. A new, heavier class of launch vehicles can help us reach the stars. |
| <a id="research-research-lv-upgr1"></a>**Improved Stack Assembly** | 4.1M | [Early Launch Systems](#research-research-lv-main1) | +20 BuildCost on LV | As spaceflight is moving out of its early experimental phase, it is time to revise our assembly procedures for mass production. |
| <a id="research-research-lv-main3"></a>**Reusable Launch Vehicles** | 4.8M | [Improved Stack Assembly](#research-research-lv-upgr1) | Launch Vehicle: [**Falcon**](../launch-vehicles/#lv-id-Rocket-RocketType3) | By adding precise guidance software and controllable surfaces to our launch vehicles, we can allow them to perform a powered landing after a successful launch, letting us use each vehicle more than once. |
| <a id="research-research-lv-main4"></a>**Nuclear Launch Vehicles** | 5.5M | [Early Launch Systems](#research-research-lv-main1)<br>[Closed-Cycle Gas Core](#research-research-nukeprop-7) | Launch Vehicle: [**Pelican**](../launch-vehicles/#lv-lv-nuke-small) | Construction of new launch vehicles, augmented with nuclear thermal rockets. |
| <a id="research-research-lv-upgr2"></a>**Optimized Heat Shields** | 5.5M | — | — | One of the greatest limiters for full launch vehicle recovery is the need for reentry on the upper stages. A better, more durable heat shield can greatly reduce stress on the vehicle. |
| <a id="research-research-lv-upgr3"></a>**Improved Reusable Rockets** | 5.5M | [Reusable Launch Vehicles](#research-research-lv-main3) | +25 BuildCost on id_Rocket_RocketType3, id_Rocket_RocketType4 | With our expertise in operating them, we can significantly optimize the cost and time required to construct reusable rockets. |
| <a id="research-research-lv2-chemadvanced"></a>**Condor** | 6.5M | [Polynitrogen Fuel](#research-research-chem-main6)<br>[Reusable Launch Vehicles](#research-research-lv-main3) | Launch Vehicle: [**Condor**](../launch-vehicles/#lv-lv-chemadvanced) | Advanced chemical launch vehicle powered by polynitrogen fuel. |
| <a id="research-research-lv2-nukemid"></a>**Magpie** | 6.5M | [Nuclear Launch Vehicles](#research-research-lv-main4) | Launch Vehicle: [**Magpie**](../launch-vehicles/#lv-lv-nuke-mid) | Powerful launch vehicle using gas core nuclear engines. |
| <a id="research-research-lv2-nukelarge"></a>**Teratorn** | 7.2M | [Magpie](#research-research-lv2-nukemid) | Launch Vehicle: [**Teratorn**](../launch-vehicles/#lv-lv-nuke-large) | The largest and most powerful launch vehicle devised, the Teratorn can lift thousands of tons and quickly come back to lift another payload. |
| <a id="research-research-lv-upgr4"></a>**Advanced Payload Fairings** | 7.2M | [Improved Reusable Rockets](#research-research-lv-upgr3)<br>[Magpie](#research-research-lv2-nukemid) | +25 MaxPayloadOnCurrentObject on lv_nuke_small, lv_nuke_mid, lv_nuke_large, lv_chemadvanced | Larger spacecraft can carry more cargo, and so our launch vehicles need even more space to deliver it to orbit. |

### Material

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-mat2"></a>**Advanced Materials** | 144k | — | — | Advanced Material Science |
| <a id="research-research-category-mat"></a>**Materials** | 648k | — | Builds [**Alloy Smelting**](../facilities/#facility-alloysmelting) | Material Science |
| <a id="research-research-mat-fibre1"></a>**Space-worthy Fibers** | 1.2M | [Materials](#research-research-category-mat) | Builds [**Polymers Production**](../facilities/#facility-polymerproduction) | Our first ventures outside the atmosphere have made us realize that to protect astronauts during excursions we need better fibers to create outfits that can survive outer space. |
| <a id="research-research-mat-metal1"></a>**Space Alloys** | 1.2M | [Materials](#research-research-category-mat) | +5 ProductionEfficiency on build_alloysmelting | The harsh environment of space requires us to adapt our materials. |
| <a id="research-research-mat-upgr1"></a>**Debris Shields** | 1.2M | — | — | A set of spaced protective layers can protect against micrometeor impacts far better and with far less mass than previous solutions. |
| <a id="research-research-mat-chips1"></a>**Circuit Production In Space** | 2.2M | [Space Alloys](#research-research-mat-metal1) | Builds [**Electronics Factory**](../facilities/#facility-electronicsfactory) | While a state of vacuum in most of the solar system seems to make creating clean rooms easier, the lower gravity and ever-present static charge build-up still make it a significant challenge to produce electronics off-world. Overcoming those challenges will let us move the supply chain further upwell. |
| <a id="research-research-mat-fibre2"></a>**Graphene** | 2.2M | [Space-worthy Fibers](#research-research-mat-fibre1) | +15 ProductionEfficiency on build_polymerproduction | A form of carbon similar to graphite, with incredible properties. |
| <a id="research-research-mat-metal2"></a>**Vacuum Forging** | 2.2M | — | — | Space brings new challenges and new opportunities in forging. By harnessing the natural conditions of space we can create better metals. |
| <a id="research-research-mat-upgr2"></a>**Aerogels** | 2.2M | — | — | Extremely lightweight materials with excellent insulating capacity and strength per unit mass. |
| <a id="research-research-mat-chips2"></a>**Improved Electronics Production** | 3.0M | [Circuit Production In Space](#research-research-mat-chips1) | +25 ProductionEfficiency on build_electronicsfactory | Adapting our space production methods further, we can improve the output of our circuit manufacturing. |
| <a id="research-research-mat-fibre3"></a>**Nanotubes** | 3.0M | [Graphene](#research-research-mat-fibre2) | +15 ProductionEfficiency on build_polymerproduction | Graphene arranged into lengths of tubes. While similar, it can provide many properties that graphene cannot. |
| <a id="research-research-mat-metal3"></a>**Advanced Alloying Techniques** | 3.0M | [Space Alloys](#research-research-mat-metal1) | Builds [**Exotic Alloy Production**](../facilities/#facility-exoticalloy) | New challenges bring new needs, and our material science divisions work tirelessly to develop new, better alloys. |
| <a id="research-research-mat-diamondoid"></a>**Diamondoids** | 4.1M | [Nanotubes](#research-research-mat-fibre3) | +10 BuildCost on Facility | Molecules with diamond-like structures can provide far greater material strength and new properties. |
| <a id="research-research-mat-fibre4"></a>**Mega-scale Carbon Allotrope Application** | 4.1M | — | — | While new carbon materials have allowed us to revolutionize material science, to complete ambitious megaprojects we dream of, we need to be prepared to apply them on a scale never heard of before. |
| <a id="research-research-mat-metal4"></a>**Improved Alloy Production** | 4.1M | [Advanced Alloying Techniques](#research-research-mat-metal3) | +10 ProductionEfficiency on build_exoticalloy, build_alloysmelting | Optimized procedures and equipment increase yields of our factories. |

### Mining

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-mine"></a>**Mining** | 648k | — | — | Extraction of resources from celestial objects |
| <a id="research-research-mine-1"></a>**In-Situ Resource Utilization** | 1.2M | [Mining](#research-research-category-mine) | Builds [**Metal Mining Base**](../facilities/#facility-metalmine) | We cannot bring everything we need where we're going. We have to be ready to extract what's needed directly from the ground we step on. |
| <a id="research-research-mine-2"></a>**Regolith Sifting** | 2.2M | [In-Situ Resource Utilization](#research-research-mine-1) | +25 MiningEfficiency on All | While rich ores cannot be always found in space, we can always extract valuable elements by sifting through the ever-present regolith. |
| <a id="research-research-mine-3"></a>**Vacuum-Optimized Boreheads** | 2.2M | [In-Situ Resource Utilization](#research-research-mine-1) | +25 MiningEfficiency on All | Better boreheads designed for operation in a vacuum environment. |
| <a id="research-research-mine-5"></a>**Radioactive Isotope Isolation** | 3.0M | [Regolith Sifting](#research-research-mine-2) | Builds [**Fissiles Mine**](../facilities/#facility-uranmine) | Nuclear power is essential for our continued presence in space. We need to be prepared for difficulties of extracting and enriching radioactive elements necessary for it on site. |
| <a id="research-research-mine-6"></a>**Advanced Mining Methods** | 3.0M | [Vacuum-Optimized Boreheads](#research-research-mine-3) | +25 MiningEfficiency on All | Further improvements in mining equipment and procedures increase yields. |
| <a id="research-research-mine-4"></a>**Helium-3 Extraction** | 4.1M | [Regolith Sifting](#research-research-mine-2)<br>[Fusion Theory](#research-research-fusionpower-1) | Builds [**Helium-3 Extractor**](../facilities/#facility-he3mine) | While sadly there aren't any viable ores of precious Helium-3, it can still be found dispersed in certain environments. As we extract other resources, it is important to make sure that traces of this valuable isotope are not lost among the tailings. |
| <a id="research-research-mine-8"></a>**Improved Fissiles Mining** | 4.1M | [Radioactive Isotope Isolation](#research-research-mine-5) | +25 MiningEfficiency on build_uranmine | Better isotope separation facilities lead to a faster mining process. |
| <a id="research-research-mine-7"></a>**Improved Helium-3 Mining** | 4.8M | [Helium-3 Extraction](#research-research-mine-4) | +75 MiningEfficiency on build_he3mine | With our experience with extracting Helium-3 in space, we can improve our mining facilities. |

### Spacecraft

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-sc2"></a>**Advanced Spacecraft** | 72k | — | — | Dedicated space vehicles for travel between orbits. |
| <a id="research-research-sc-helios"></a>**Stratos** | 144k | [Spacecraft](#research-research-category-sc) | Spacecraft: [**Stratos**](../spacecraft/#spacecraft-spacecraft-chem-large) | Powerful upper stage capable of independent operation in space, ideal workhorse for the moon and beyond. |
| <a id="research-research-sc-orion"></a>**Selene** | 432k | [Iris](#research-research-sc-iris) | Spacecraft: [**Selene**](../spacecraft/#spacecraft-spacecraft-chem-mid2) | Light craft with an electric engine for moving probes and small cargo to distant objects. |
| <a id="research-research-category-sails"></a>**Solar Sails** | 648k | [Space-worthy Fibers](#research-research-mat-fibre1) | Spacecraft: [**Daedalus**](../spacecraft/#spacecraft-spacecraft-sail-small) | Solar Sails operate using nothing but the sun's energy to maneuver through space. Though slow and limited by distance from the sun, they require no fuel or propellant. |
| <a id="research-research-category-sc"></a>**Spacecraft** | 648k | [Early Launch Systems](#research-research-lv-main1) | — | Dedicated space vehicles for travel between orbits. |
| <a id="research-research-sc-hermes"></a>**Hermes** | 864k | [Stratos](#research-research-sc-helios) | Spacecraft: [**Hermes**](../spacecraft/#spacecraft-spacecraft-electric-small) | Dedicated deep space transport, with artificial gravity, life support capacity, and a powerful array of ion engines. |
| <a id="research-research-sails-1"></a>**Optimized Payload Bus** | 1.2M | [Solar Sails](#research-research-category-sails) | +100 ComponentCargoCapacity on cargo_solar_small | Lighter, more efficiently shaped bus will allow us to load more cargo in the same sail craft. |
| <a id="research-research-sc-iris"></a>**Iris** | 1.2M | [Spacecraft](#research-research-category-sc) | Spacecraft: [**Iris**](../spacecraft/#spacecraft-spacecraft-chem-small) | A simple probe carrier craft. |
| <a id="research-research-sc-cargo1"></a>**Expanded Cargo Bays 1** | 2.2M | [Spacecraft](#research-research-category-sc)<br>[Stratos](#research-research-sc-helios) | +50 SCCargoCapacityBase on spacecraft_chem_mid2, spacecraft_chem_large | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| <a id="research-research-sc-prometheus"></a>**Prometheus** | 2.2M | [Spacecraft](#research-research-category-sc)<br>[Nuclear Propulsion](#research-research-category-nukeprop) | Spacecraft: [**Prometheus**](../spacecraft/#spacecraft-spacecraft-nuke-small) | The first true nuclear rocket, more than double the efficiency of chemical rockets while still maintaining high thrust |
| <a id="research-research-sails-2"></a>**Reflective Materials** | 3.0M | [Optimized Payload Bus](#research-research-sails-1)<br>[Graphene](#research-research-mat-fibre2) | Spacecraft: [**Talos**](../spacecraft/#spacecraft-spacecraft-sail-mid) | The propulsive effect of solar sails is achieved by reflection of photons emitted by the sun. Material improvements in reflectivity allow construction of even better sail based spacecraft. |
| <a id="research-research-sc-cargo2"></a>**Expanded Cargo Bays 2** | 3.0M | [Expanded Cargo Bays 1](#research-research-sc-cargo1)<br>[Hermes](#research-research-sc-hermes) | +20 SCCargoCapacityBase on spacecraft_electric_small, spacecraft_electric_mid, spacecraft_nuke_small | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| <a id="research-research-sc-hecate"></a>**Athena** | 3.0M | [Hermes](#research-research-sc-hermes)<br>[MPD](#research-research-electricprop-5) | Spacecraft: [**Athena**](../spacecraft/#spacecraft-spacecraft-electric-mid) | Advanced electric spacecraft, designed for slow but efficient travel |
| <a id="research-research-sails-3"></a>**Improved Sail Deployment** | 4.1M | [Reflective Materials](#research-research-sails-2) | +20 BuildCost on spacecraft_sail_small, spacecraft_sail_mid | A major issue with launching sail-based spacecraft is unfurling the sail without damaging it. Optimal performance is achieved by the thinnest sail possible, which makes them fragile. Better techniques of deploying the sail would make it far easier to construct and launch. |
| <a id="research-research-sc-hephaistos"></a>**Hephaistos** | 4.1M | [Prometheus](#research-research-sc-prometheus)<br>[Liquid Core](#research-research-nukeprop-5) | Spacecraft: [**Hephaistos**](../spacecraft/#spacecraft-spacecraft-nuke-mid) | Nuclear thermal electric spacecraft, capable of significantly cutting down travel time between planets |
| <a id="research-research-sails-4"></a>**Plasma Magnet Sail** | 4.8M | [Improved Sail Deployment](#research-research-sails-3)<br>[Superconducting Magnets](#research-research-electromag-1) | Spacecraft: [**Zephyr**](../spacecraft/#spacecraft-spacecraft-sail-long) | The plasma magnet sail replaces huge stretches of fabric with thin loops of superconducting wire that uses the charged particles of the solar wind itself to generate a far larger magnetic sail, enabling it to fly faster and farther than solar sails. |
| <a id="research-research-sc-cargo3"></a>**Expanded Cargo Bays 3** | 4.8M | [Expanded Cargo Bays 2](#research-research-sc-cargo2)<br>[Hephaistos](#research-research-sc-hephaistos) | +20 SCCargoCapacityBase on spacecraft_nuke_mid, spacecraft_nuke_large, spacecraft_nuke_nolv | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| <a id="research-research-sc-ariane"></a>**Ariane** | 5.5M | [Hephaistos](#research-research-sc-hephaistos)<br>[Closed-Cycle Gas Core](#research-research-nukeprop-7) | Spacecraft: [**Ariane**](../spacecraft/#spacecraft-spacecraft-nuke-large) | Most powerful nuclear thermal spacecraft, powered by a Nuclear Lightbulb" engine." |
| <a id="research-research-sc-cronos"></a>**Cronos** | 5.5M | [Hephaistos](#research-research-sc-hephaistos)<br>[Closed-Cycle Gas Core](#research-research-nukeprop-7) | Spacecraft: [**Cronos**](../spacecraft/#spacecraft-spacecraft-nuke-nolv) | Single-Stage-To-Orbit spacecraft powered by a closed-cycle gas core nuclear thermal rocket engine. A set of seven engines lets this colossus lift itself and a kiloton of payload into earth orbit and beyond. |
| <a id="research-research-sc-nike"></a>**Nike** | 6.5M | [Spacecraft](#research-research-category-sc)<br>[Fusion Propulsion](#research-research-category-fusionprop) | Spacecraft: [**Nike**](../spacecraft/#spacecraft-spacecraft-fusion-small) | First generation fusion spacecraft, with efficiency unmatched by previous designs. |
| <a id="research-research-sc-cargo4"></a>**Expanded Cargo Bays 4** | 7.2M | [Expanded Cargo Bays 3](#research-research-sc-cargo3)<br>[Nike](#research-research-sc-nike) | +20 SCCargoCapacityBase on spacecraft_fusion_small, spacecraft_fusion_mid, spacecraft_fusion_large | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| <a id="research-research-sc-sirius"></a>**Sirius** | 8.2M | [Nike](#research-research-sc-nike)<br>[Fuel Injection Optimization](#research-research-fusionprop-3) | Spacecraft: [**Sirius**](../spacecraft/#spacecraft-spacecraft-fusion-mid) | Advanced fusion spacecraft. |
| <a id="research-research-sc-atlas"></a>**Atlas** | 9.1M | [Sirius](#research-research-sc-sirius)<br>[Magnetic Spin Alignment](#research-research-fusionprop-1) | Spacecraft: [**Atlas**](../spacecraft/#spacecraft-spacecraft-asteroid-puller) | Enormous spacecraft able to move entire asteroids. |
| <a id="research-research-sc-zeus"></a>**Zeus** | 9.1M | [Sirius](#research-research-sc-sirius)<br>[Magnetic Spin Alignment](#research-research-fusionprop-1) | Spacecraft: [**Zeus**](../spacecraft/#spacecraft-spacecraft-fusion-large) | Operating at the limits of theoretical performance of fusion spacecraft, the Zeus can fly farther and faster than any other. |

## Physics

### Computing

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-computing"></a>**Computing** | 648k | — | — | The study of computer science and construction of processing units. |
| <a id="research-research-computing-proc1"></a>**Microprocessors** | 1.2M | [Computing](#research-research-category-computing) | — | Miniature processing units, freeing us of the need for enormous computer frames. |
| <a id="research-research-computing-proc2"></a>**Nanoprocessors** | 2.2M | [Microprocessors](#research-research-computing-proc1) | +4 ResearchProduction | Further miniaturization of processing units greatly increases processing power, allowing larger amounts of data to be analyzed, and more complex simulations to be run. |
| <a id="research-research-computing-proc4"></a>**Early QPUs** | 2.2M | — | +0 ResearchProduction | Early advancements in the construction of Quantum Processing Units allows experimental usage of new kinds of computing for specialized data processing tasks. However, the practical applications of such remain limited due to the substantial difficulty in producing circuits that can avoid decoherence. |
| <a id="research-research-computing-ai1"></a>**Machine Learning** | 2.4M | [Nanoprocessors](#research-research-computing-proc2) | +4 ResearchProduction | Statistical models capable of studying and extrapolating from data, potentially solving tasks where writing algorithms is unfeasible. |
| <a id="research-research-computing-ai2"></a>**Neural Networks** | 3.0M | — | +0 ResearchProduction | A type of machine learning inspired by the mechanism of biological neurons, Neural Networks map nodes and their connection as an analog of neurons and are shaped by further training on data. |
| <a id="research-research-computing-ai3"></a>**Basic AI** | 3.0M | [Machine Learning](#research-research-computing-ai1) | +5 ResearchProduction | As the size of our learning models grows, so do their capabilities. Impressive pattern recognition when trained on controlled samples can be harnessed for unparalleled processing of data. |
| <a id="research-research-computing-proc3"></a>**Three-Dimensional Integrated Circuits** | 3.0M | — | +0 ResearchProduction | As ever smaller circuits run into physical limits of size, further increases in density of processing power can be achieved by expanding existing circuits vertically, into the third dimension. |
| <a id="research-research-computing-proc5"></a>**Reversible Circuits** | 4.1M | — | +0 ResearchProduction | A special kind of computer circuitry operating on reversible logic, that is logic in which all inputs can be determined based on their outputs. Reversibility of the process means that numerous calculations can be done with minimal entropic losses, greatly increasing energy efficiency. |
| <a id="research-research-computing-proc6"></a>**Quantum Computing** | 4.1M | [Nanoprocessors](#research-research-computing-proc2) | +8 ResearchProduction | Practical implementation of quantum mechanical effects into computing greatly increases capacity for data processing and physical simulations. Although quantum computers cannot replace classical computing in numerous tasks, they can greatly exceed them in others, supplementing existing computing methods. |
| <a id="research-research-computing-ai4"></a>**Advanced AI** | 4.8M | [Basic AI](#research-research-computing-ai3) | +8 ResearchProduction | Refinements, iterations, and increased processing power produce even more capable models. |

### Electromagnetism

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-electromag"></a>**Electromagnetism** | 648k | — | — | The study of electricity and magnetism. |
| <a id="research-research-electromag-5"></a>**Phased-Arrays** | 864k | — | — | A set of radio antennas controlled by a computer, capable of being steered in a specific direction without moving the antenna itself. |
| <a id="research-research-electromag-7"></a>**Atomic Manipulation** | 1.2M | [Electromagnetism](#research-research-category-electromag) | +20 ProductionEfficiency on build_exoticalloy | The process of manipulating individual atoms through the use of a Scanning Tunneling Microscope. This can be used to create atomic-scale structures or unique molecules. |
| <a id="research-research-electromag-8"></a>**Lasers** | 1.2M | — | — | Precisely controlled emitters of coherent light. |
| <a id="research-research-mat-superconductor1"></a>**High-Temperature Superconductors** | 1.2M | [Electromagnetism](#research-research-category-electromag) | +10 PowerProduction on All | High-temperature superconductors are a form of superconductor capable of operating above the boiling point of nitrogen, greatly reducing the need for extensive cooling. |
| <a id="research-research-electromag-1"></a>**Superconducting Magnets** | 2.2M | [Electromagnetism](#research-research-category-electromag)<br>[High-Temperature Superconductors](#research-research-mat-superconductor1) | +10 MaintenanceReduce on Facility | Utilization of superconductors in electromagnets can greatly improve their performance. |
| <a id="research-research-mat-superconductor2"></a>**Room-Temperature Superconductors** | 2.2M | [High-Temperature Superconductors](#research-research-mat-superconductor1) | +20 MaintenanceReduce on Facility | The holy grail of superconductivity, Room-Temperature Superconductors can achieve the state of superconductivity at ambient temperatures. |

### Electroprop

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-electricprop"></a>**Electric Propulsion** | 648k | — | — | Highly efficient but low in thrust, electric drives can handle great distances but are limited by their need for power. |
| <a id="research-research-electricprop-1"></a>**Hall Thruster** | 1.2M | [Electric Propulsion](#research-research-category-electricprop) | +35 ComponentExhaustV on eng_electric, eng_electricmpd, eng_electrichermes | Ions are accelerated outside this thruster through the use of the Hall Current, giving it its name. |
| <a id="research-research-electricprop-2"></a>**Ion Engine** | 1.7M | — | — | Electrostatic acceleration of argon seeded with potassium through a grid of electrodes provides thrust in this engine. |
| <a id="research-research-electricprop-5"></a>**MPD** | 2.2M | [Hall Thruster](#research-research-electricprop-1) | +50 ComponentExhaustV on eng_electric, eng_electrichermes, eng_electricmpd | Magnetoplasmadynamic thrusters utilize the Lorentz force to propel ionized propellant through a generated magnetic field, at velocities far greater than achievable in gridded ion thrusters. |
| <a id="research-research-electricprop-6"></a>**Optimized Self-field** | 3.0M | [MPD](#research-research-electricprop-5) | +40 ComponentExhaustV on eng_electric, eng_electrichermes, eng_electricmpd | A more optimized self-field in an MPD can improve its performance. |
| <a id="research-research-electricprop-3"></a>**Electrodeless Ion Generation** | 3.5M | — | — | By removing electrodes utilized in an ion engine, as well as redesigning for certain efficiencies, we can greatly improve its performance without worrying about corrosion. |
| <a id="research-research-electricprop-4"></a>**Multi-stage Grid** | 6.9M | — | — | Additional grid stages of accelerating grids in the engine allow us to increase the exit velocity of ions. |

### Exploration

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-observation"></a>**Observation** | 648k | — | — | Astronomy, observation, and study of celestial objects |
| <a id="research-research-observation-1"></a>**Improved Optics** | 1.2M | [Observation](#research-research-category-observation) | +30 ObservationSpeed | Improvements in the creation of large lenses can help us observe distant objects. |
| <a id="research-research-observation-2"></a>**Radar Astronomy** | 1.7M | — | +30 ObservationSpeed | Although RADAR had been invented before WW2 and early observations of the moon through it took place soon after the war, it was not until recently that this technique could be reliably utilized to observe and measure bodies within our solar system. By investing in this technology further, we may find out much more about our planetary neighborhood. |
| <a id="research-research-observation-3"></a>**Space Telescopes** | 2.2M | [Improved Optics](#research-research-observation-1) | Builds [**Telescope**](../facilities/#facility-space-telescope) | Ground observations are severely limited by interference from our atmosphere. By placing a remote telescope observatory outside of Earth's atmosphere, we can observe things we couldn't before. |
| <a id="research-research-observation-4"></a>**Exoplanet Detection** | 3.0M | [Space Telescopes](#research-research-observation-3) | — | By using various techniques that detect variations in a star's characteristics, we can spot and estimate potential exoplanets circling those distant stars. |

### Fusion

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-fusionpower-1"></a>**Fusion Theory** | 3.0M | [Nuclear Power](#research-research-category-nukepower)<br>[Superconducting Magnets](#research-research-electromag-1) | — | The use of nuclear fusion to generate power. |
| <a id="research-research-electromag-2"></a>**Magnetic Field Containment** | 4.1M | [Fusion Theory](#research-research-fusionpower-1) | Builds [**Fusion Reactor**](../facilities/#facility-power-fusion) | In our quest to achieve nuclear fusion, one of the greatest problems is containing hot fusion fuel plasma inside a magnetic field, at pressures great enough to allow ignition. |
| <a id="research-research-electromag-3"></a>**Magnetic Nozzle** | 4.8M | — | — | As we create more powerful engines, the ever-greater temperatures put more stress on the material of our nozzles. While ever-greater cooling could potentially solve the issue, another concept is to do away with a physical nozzle entirely, instead containing the exhaust energies within magnetic fields. |
| <a id="research-research-category-fusionprop"></a>**Fusion Propulsion** | 5.5M | [Magnetic Field Containment](#research-research-electromag-2) | +10 PowerProduction on build_power_fusion | The application of all our fusion knowledge finally lets us harness it to power our spacecraft. |
| <a id="research-research-fusionprop-2"></a>**Improved Plasma Containment** | 6.5M | [Fusion Propulsion](#research-research-category-fusionprop) | +100 ComponentExhaustV on eng_fusion, eng_fusionnike | By working out inefficiencies in our magnetic nozzles we can more completely capture all the energy from fusion reactions, improving engine performance. |
| <a id="research-research-fusionprop-3"></a>**Fuel Injection Optimization** | 7.2M | [Improved Plasma Containment](#research-research-fusionprop-2) | +100 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius | An important element of fusion reactions is to supply fuel at optimal rate and direction to achieve as high of a burn up as possible. Improved fuel flow into the reactor chamber guarantees better efficiency. |
| <a id="research-research-fusionprop-1"></a>**Magnetic Spin Alignment** | 8.2M | [Fuel Injection Optimization](#research-research-fusionprop-3) | +200 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius | The holy grail of fusion propulsion. By precisely aligning the spin of individual fuel particles, we can control the direction of resulting neutron radiation, turning previously wasted energy into additional thrust. The reduction in required shielding, and additional exhaust energy massively improves performance of our fusion spacecraft. |
| <a id="research-research-fusionprop-4"></a>**Advanced Fusion Propulsion** | 9.1M | [Magnetic Spin Alignment](#research-research-fusionprop-1) | +25 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius, eng_fusionzeus, eng_fusionatlas | Further optimising our engines, we can improve their performance even more. |

### Nuclear

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-nukepower"></a>**Nuclear Power** | 648k | — | Builds [**Nuclear Reactor**](../facilities/#facility-power-nuke) | The use of nuclear fission to generate power. |
| <a id="research-research-nukeprop-1"></a>**High Temperature NTR** | 864k | — | — | We can increase the performance of our engines by keeping the fuel elements close to the melting point, increasing the power available for thrust. |
| <a id="research-research-nukeprop-2"></a>**Helium Loop Cycle** | 1.2M | — | — | An additional heat transfer system utilizing helium can be used to run a power-generating turbine, or to power a turboinductor device, transferring additional power to propellant. |
| <a id="research-research-category-nukeprop"></a>**Nuclear Propulsion** | 1.2M | [Nuclear Power](#research-research-category-nukepower) | — | Nuclear Thermal Rockets utilize the high energies of nuclear reactors to accelerate propellant to far greater speeds than in chemical engines while maintaining good thrust. |
| <a id="research-research-mat-refractory"></a>**Improved Reactor Vessels** | 1.2M | [Nuclear Power](#research-research-category-nukepower) | — | To handle greater temperatures, new alloys that can handle thermal loads beyond those of known elements need to be researched. |
| <a id="research-research-nukepower-1"></a>**Molten Fission Systems** | 2.2M | [Improved Reactor Vessels](#research-research-mat-refractory) | +50 PowerProduction on build_power_nuke | Nuclear reactors operating at temperatures so high that the fuel elements become molten can achieve far greater power outputs. |
| <a id="research-research-nukepower-2"></a>**Gas Core Fission** | 3.0M | [Molten Fission Systems](#research-research-nukepower-1) | +50 PowerProduction on build_power_nuke | Even greater power densities can be achieved in nuclear reactors, however this requires temperatures high enough to turn fissile elements into gas. Contained within an actively cooled pressure vessel, rapidly fissioning gas gives off tremendous power - as long as it is contained. |
| <a id="research-research-nukeprop-5"></a>**Liquid Core** | 3.0M | [Nuclear Propulsion](#research-research-category-nukeprop)<br>[Molten Fission Systems](#research-research-nukepower-1) | +100 ComponentExhaustV on eng_nuke, eng_nukeprometheus | Previously our nuclear rockets were limited by the melting point of uranium. With systems designed to handle liquid nuclear fuel, we can reach beyond that barrier and utilize even greater power in our rockets. |
| <a id="research-research-nukeprop-3"></a>**Nuclear Thermal Electric Rocket** | 4.1M | [Liquid Core](#research-research-nukeprop-5) | +100 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | Using a two-stage system of cooling loops and heat engines, we can more efficiently transfer heat to our propellant while also generating substantial power for an electric afterbooster, significantly increasing the efficiency of our nuclear rockets. |
| <a id="research-research-nukeprop-7"></a>**Closed-Cycle Gas Core** | 4.8M | [Nuclear Thermal Electric Rocket](#research-research-nukeprop-3)<br>[Gas Core Fission](#research-research-nukepower-2) | +75 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | Fissile gas is reacted inside a transparent pressure vessel - a "Nuclear Lightbulb" - emitting an enormous amount of power transferred to hydrogen propellant. |
| <a id="research-research-nukeprop-4"></a>**MPD-Based NTER** | 10.4M | — | +50 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus, eng_nukeariane | By utilizing Magnetoplasmadynamic technology in place of the arcjet, we can further boost the efficiency of our nuclear engines. |
| <a id="research-research-nukeprop-6"></a>**Advanced Liquid Core** | 13.8M | — | +50 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | An actively cooled rotating fuel drum keeps the uranium in contact with hydrogen propellant while preventing losses of fuel through the exhaust, thanks to centrifugal force. |

### Power

| Research | <span title="Cost in work-hours; divide by your labs' research output to get the actual research time in days">Cost (h)</span> | Prereqs | Unlocks | Description |
| --- | --- | --- | --- | --- |
| <a id="research-research-category-power"></a>**Power** | 648k | — | — | Provides electricity for planetary bases. |
| <a id="research-research-power-geo"></a>**Geothermal Power Turbine** | 1.2M | [Power](#research-research-category-power) | Builds [**Geothermal Power**](../facilities/#facility-power-geothermal) | Active planetary geology provides thermal gradients that can be utilized to generate power. |
| <a id="research-research-power-solar1"></a>**Solar Cell Arrays** | 1.2M | [Power](#research-research-category-power) | Builds [**Solar Array**](../facilities/#facility-power-solar) | Utilization of the photoelectric effect for power generation, solar cells have become a ubiquitous form of power supply for spacecraft. |
| <a id="research-research-power-wind"></a>**Wind Turbine** | 1.2M | [Power](#research-research-category-power) | Builds [**Wind Power**](../facilities/#facility-power-wind) | Wind power generators, similar to ones on Earth, designed for vastly different planetary conditions and ease of construction on site. |
| <a id="research-research-power-chem"></a>**Chemical Power Reactor** | 1.3M | — | Builds [**Chemical Reactor**](../facilities/#facility-power-chemical) | On planets with abundant hydrocarbon atmospheres such as Titan, energy can be produced from chemical reactions of atmospheric compounds. Hydrogenation of acetylene (or if lacking, ammonia) generates sufficient heat to produce net power. |
| <a id="research-research-electromag-9"></a>**Remote Power Transfer** | 2.2M | [Power](#research-research-category-power) | Builds [**Remote Power Receiver**](../facilities/#facility-power-receiver) | By utilizing coherent microwave beams and rectennas, we can transmit large amounts of electrical power from far away or even through the atmosphere. |
| <a id="research-research-power-solar2"></a>**Improved Solar Cells** | 2.2M | [Solar Cell Arrays](#research-research-power-solar1) | +50 PowerProduction on build_power_solar | Better materials and technologies allow us to greatly increase the power density of solar cells. |
| <a id="research-research-power-powersat"></a>**Powersat** | 3.0M | [Power](#research-research-category-power)<br>[Remote Power Transfer](#research-research-electromag-9) | Builds [**Orbital Power Station**](../facilities/#facility-power-transfer-orbit) | Large solar collectors in orbit, capable of transferring power down to the surface or other spacecraft by the use of microwave beams. |

## Reading the table

- **Cost** is in work hours and is divided by your laboratories' research output to get the actual research time in days.
- **Prerequisites** must be completed before the node becomes available.
- **Unlocks** — *Builds X* means the node makes a new facility constructable; *Spacecraft / Launch Vehicle* means the node unlocks a new ship or lifter; numeric bonuses apply to listed components.

## See also

- [Spacecraft](../spacecraft/) — propulsion research feeds directly into these
- [Launch Vehicles](../launch-vehicles/)

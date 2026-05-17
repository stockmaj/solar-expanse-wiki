# Research

The tech tree drives progression. Every research node has a work-hours cost,
zero or more prerequisite research nodes, and unlocks something — a new
facility, spacecraft, launch vehicle, or a numeric bonus on existing
equipment. Research is grouped into three top-level branches (Engineering,
Physics, Biotech), each subdivided into focused sub-branches.

## Biotech

### Agriculture

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Agriculture** | 288k | — | — | The growth of food and plants outside of Earth presents new challenges to be solved. |

### Biotech

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Biotech** | 288k | — | — | Application of organisms and biology in technological contexts |
| **Space Farming** | 1.2M | Biotech | Builds **Hydroponic Farm** | Although small growth experiments have been conducted in 0g, there are still adaptations needed to allow large scale agriculture in extraterrestrial environments. Usage of appropriate aggregate medium, artificial lighting, radiation shielding, and local power sourcing could allow colonies to feed themselves. |
| **Bioreactors** | 1.2M | Biotech | +10 ProductionEfficiency on build_farm | Systems for containing biologically active processes that can allow us to utilize many useful organisms in a controlled manner. |
| **Regolith Adaptation** | 2.2M | Space Farming | +25 ProductionEfficiency on build_farm | Adapting local regolith into soil or aggregate, greatly expands available growth mediums, allowing more crops to be planted. |
| **Knallgas Bacteria Farms** | 2.2M | — | — | Hydrogen-oxidizing bacteria that can use hydrogen, oxygen, and carbon dioxide as nutrition, producing primarily water that can be recycled back into hydrogen and oxygen. Knallgas bacteria farms can provide protein in a space in a resource-efficient way. |
| **Genetic Engineering** | 2.2M | Bioreactors | +15 LifeSupportConsumption on All | We can speed up the slow process of selective breeding by directly finding and implanting beneficial genes, improving crop yields, resistances to disease, and many others. |
| **Aeroponics** | 3.0M | Regolith Adaptation | +25 ProductionEfficiency on build_farm | An advanced form of hydroponics, with plant roots being exposed to a mist of nutrient-rich water solution. Growth of food in the absence of soil or aggregate can provide great improvements, particularly in areas of water usage, and nutrition. |
| **Radiotrophic Fungi** | 3.0M | — | — | Fungi capable of using radiation as an energy source have been previously found on sites of nuclear disasters. Using adapted local soil, they could be used as self-growing radiation shielding for planetary habitats. |
| **Closed-cycle Algae** | 3.0M | — | — | Tanks of algae are utilized to recycle biological waste and CO2 back into usable nutrition and oxygen, creating a closed ecosystem within the ship. |
| **Biological Machines** | 3.0M | Genetic Engineering | +10 ProductionEfficiency on build_farm | Engineered assemblies of molecules and proteins can be designed to perform tasks at nanoscale. |
| **Low-G GMO** | 4.1M | — | — | Through the use of genetic engineering, we can adapt our crops to better grow in low- or microgravity environments. |

### Colonization

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Planetary Habitats** | 1.2M | Life Support | Builds **Outpost** | Permanent solution for extraterrestrial bases. |
| **Colony Construction** | 2.2M | — | — | With our experience in construction and habitation on other planets we can finally realize the dream of permanent colonies outside of Earth. |
| **Regolith Shielding** | 3.0M | Planetary Habitats | +-35 BuildCost on build_outpost, build_habitat, build_habitatdome, build_habitatcity | We can greatly reduce the need for material by utilizing local regolith as radiation shielding in our habitats. |
| **On-Site 3D Printing** | 3.0M | Planetary Habitats | +25 BuildSpeed on Facility | With laser printing robots we can greatly speed up the assembly of ground facilities. |
| **Spin-Grav Habitation** | 3.0M | Planetary Habitats | Builds **Rotating Habitat** | By spinning habitat segments, artificial gravity can be created through centrifugal force. Careful tuning of distance and speed of rotation is required, to maintain a balance between strength of simulated gravity and potential nausea from rotation. |
| **Permanent Orbital Presence** | 4.1M | — | — | To support our expansion into space, we need to establish a permanent orbital infrastructure. |
| **In-Orbit Production** | 4.8M | Spin-Grav Habitation | +30 BuildCost on build_space_drydock | By constructing our interplanetary vehicles directly in orbit, we're free from the size and weight constraints of our launch vehicles. |

### LifeSupport

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Life Support** | 648k | — | — | It is essential to develop technologies that help sustain our astronauts in good health. |
| **Crewed Flight** | 1.2M | Life Support | Builds **module_crew_compartment** | Safe orbital transport of small crews. |
| **Improved Capsule Construction** | 2.2M | Crewed Flight | +-40 BuildCost on module_crew_compartment, module_crew_medium, module_crew_large | The same technologies that have helped our launch vehicles achieve reusability can be used to improve the construction of our crew transports. |
| **Improved Ration Packing** | 2.2M | — | — | As human space travel becomes more regular, standardization in the form of rations provided to our astronauts is needed. |
| **Robotics** | 3.0M | Life Support<br>Nanoprocessors | +5 ReduceCrewRequirements on All | Construction and application of robotic systems. |
| **Artificial Hibernation** | 3.0M | Life Support<br>Genetic Engineering | +35 LifeSupportConsumptionOnShip on All | Artificially inducing a state of reduced metabolic rate and bodily activity in passengers and unessential crew will significantly reduce the amount of supplies required to sustain ships during long missions. |
| **STO Transports** | 3.0M | Improved Capsule Construction | Builds **module_crew_large** | By adopting design principles from recently developed passenger supersonic aircraft, we can create new shuttles capable of transporting a hundred people at a time. |
| **Improved Atmosphere Recycling** | 3.0M | Crewed Flight | +25 LifeSupportConsumption on All | The Sabatier reaction can be used to extend oxygen reserves, by recycling carbon dioxide back to oxygen, using hydrogen. |
| **Suspended Animation** | 4.1M | Artificial Hibernation<br>Biological Machines | +35 LifeSupportConsumptionOnShip on All | Enhancements in artificial hibernation technology allow putting the body in a state of near-zero biological activity while still keeping subjects alive. This suspended animation allows passengers to be kept unconscious on long missions while consuming near minimal supplies. |
| **Exoskeletons** | 4.1M | Robotics<br>Advanced Alloying Techniques | +5 ReduceCrewRequirements on All | Frame of servomotors, supports, and polymers that augments the user's physical capabilities. They allow workers to do more and tire less. |
| **Expert Systems** | 4.8M | Robotics<br>Basic AI | +10 ReduceCrewRequirements on All | Specialized Artificial Intelligence systems, equipped with knowledge and logic designed to handle all tasks within their area of expertise. |
| **Cybernetics** | 4.8M | Machine Learning<br>Exoskeletons<br>Graphene | +10 ReduceCrewRequirements on All | Integration of man and machine. Robotic limbs and neural interfaces improve the capabilities of our colonists, pushing the boundaries of humanity. |

### Spacecraft

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Interstellar Travel** | 10.3M | Zeus<br>In-Orbit Production<br>Diamondoids | Builds **Interstellar Vehicle Assembly** | We shall lay down the foundations to construct a great craft that will bring us to other stars. |

### Terraforming

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Terraforming** | 648k | — | — | Transformation of celestial objects into an Earth-like environment |
| **Carbon Release** | 1.2M | Terraforming | Builds **Carbon Power Plant** | We can slowly raise the temperature and pressure of our environment by processing and releasing carbon dioxide from local minerals. |
| **Stellar Mirror** | 2.2M | Carbon Release | Builds **Orbital Mirror** | An orbital installation of highly reflective material, redirecting sunlight onto the surface of a planet to raise its temperature. |
| **Artificial Magnetosphere** | 3.0M | Terraforming<br>Superconducting Magnets<br>Room-Temperature Superconductors | Builds **Planetary Magnetosphere Generator** | Huge electromagnets made of superconductors are used to mimic a planet's magnetic field, shielding the surface from harmful radiation. |
| **Mars Bacteria** | 27.6M | — | — | Genetically engineered bacteria, able to survive and photosynthesize in the harsh Mars environment. |
| **Photosynthetic Microbes** | 27.6M | — | — | Genetically engineered bacteria, able to survive and photosynthesize in the harsh Mars environment. |
| **Albedo Manipulation** | 27.6M | — | — | Seeding the poles with black carbon dust can increase the amount of sunlight absorbed by it, melting the carbon dioxide ice sheets. |
| **Comet Redirection** | 55.3M | — | — | By redirecting comets full of volatiles, we can increase atmospheric pressure and the amount of water present on a planet. |

## Engineering

### Chemical

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Chemical Propulsion** | 288k | — | — | Powerful but inefficient, chemical engines powered the first rockets, and remain the main propulsion for launch vehicles. |
| **Solid Propellant Rockets** | 1.2M | Chemical Propulsion | — | The simplest form of propulsion, a solid fuel is ignited and directed out of the exhaust to catch fire. While high in thrust and simple in design, the length of the burn cannot be controlled and the thruster will keep firing until out of fuel. |
| **Kerolox** | 2.2M | Solid Propellant Rockets | +5 ComponentExhaustV on eng_chem | Liquid oxygen and a form of highly refined kerosene - rocket fuel - provide the best performance without going fully cryogenic. |
| **Methalox** | 3.0M | Kerolox | +6 ComponentExhaustV on eng_chem | Combustion of liquid methane and liquid oxygen provides improved specific impulses over more traditional propellants while being the easiest to store cryogen. |
| **Hydrolox** | 4.1M | Methalox | +20 ComponentExhaustV on eng_chem, eng_chemsmall, eng_chemhelios, eng_chemorion | Liquid hydrogen and liquid oxygen are combusted, producing best-in-class specific impulses. While harder to store than other fuels, it burns cleanly into water vapor and can be produced from simple water electrolysis. |
| **Solid Hydrogen** | 4.8M | Hydrolox | +25 ComponentExhaustV on eng_chem, eng_chemsmall, eng_chemhelios, eng_chemorion | Highly pressurized and cooled hydrogen can condense into a solid. This increases density and allows to suspend free radical hydrogen within the ice, improving exhaust velocity. |
| **Polynitrogen Fuel** | 5.5M | Solid Hydrogen | +20 LaunchCost on LV | Polynitrogens are highly energetic molecules composed entirely of nitrogen atoms. While the exhaust velocity improvements over hydrogen vary, their over 30 times greater density allows much smaller and more compact launch vehicles, reducing the mass of propellant tanks and drag losses. |

### Colonization

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Orbital Gas Extractor** | 5.5M | In-Orbit Production | Builds **Orbital Gas Extractor** | Orbital mining station for extracting gases from atmospheres, including gas giants. |

### Electromagnetism

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Metamaterials** | 2.2M | — | +0 PowerProduction | Composite materials composed in a way that allows them to have properties not normally observed in naturally occurring elements. |

### LaunchFacility

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Launch Facility** | 648k | — | — | Launch infrastructure that can substantially help with reaching orbit. |
| **Launch Pad** | 1.2M | Launch Facility | Builds **Launch Pad** | Proper launch infrastructure can make our landings easier and safer. |
| **Spin-Launch** | 2.2M | Launch Pad | Builds **Rotary Launcher** | A light payload is greatly accelerated in a centrifuge, then boosted to the final orbit by a small chemical engine. |
| **Magnetic Launch Rails** | 3.0M | Spin-Launch<br>Superconducting Magnets | Builds **Magnetic Launch Rails** | Using the technology pioneered by maglev trains, the launch vehicle is accelerated and launched from a track built on a mountain slope. |
| **Mass Driver** | 3.0M | Spin-Launch<br>Superconducting Magnets | Builds **Stationary Mass Driver** | Magnetic coils accelerate the payload. |
| **Electromagnetic Catapult** | 4.1M | Mass Driver<br>Room-Temperature Superconductors | Builds **Electromagnetic Catapult** | Larger and more powerful mass driver capable of launching payloads on interplanetary trajectories by itself. |
| **Space Elevator** | 5.6M | Magnetic Launch Rails<br>Nanotubes | Builds **Space Elevator** | Supermaterial tether reaching synchronous orbit, with an elevator able to traverse the tether and lift payloads straight to orbital height. |
| **Mass Driver Propulsion** | 6.5M | Mass Driver<br>Fusion Propulsion | Builds **Asteroid Engine** | Crushed regolith is placed within buckets and accelerated out to 10 km/s down a mass driver. The regolith leaves the engine, transferring its momentum, while the bucket is decelerated to be reused. |

### LaunchVehicle

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Launch Vehicles** | 648k | Chemical Propulsion | — | The primary solution for surface-to-orbit transport. |
| **Aluminium-Ice Rockets** | 864k | Reusable Launch Vehicles | Launch Vehicle: **id_Rocket_RocketType5** | Microscopic particles of aluminium suspended in ice. Once ignited, the aluminium reacts with oxygen from the ice, creating heat, while the liberated hydrogen helps with burn efficiency. While not powerful or efficient enough for Earth, this type of rocket can easily be assembled on-site and does not require extensive cooling that typical cryogenic propellants need. |
| **Early Launch Systems** | 3.0M | Launch Vehicles<br>Kerolox | Launch Vehicle: **id_Rocket_RocketType7** | Early rockets consisted primarily of solid propellant boosters and kerosene/liquid oxygen launch vehicles. |
| **Superheavy Launch** | 4.1M | — | — | To truly reach out into space we need ever greater capacity to lift entire spaceships into orbit. A new, heavier class of launch vehicles can help us reach the stars. |
| **Improved Stack Assembly** | 4.1M | Early Launch Systems | +20 BuildCost on LV | As spaceflight is moving out of its early experimental phase, it is time to revise our assembly procedures for mass production. |
| **Reusable Launch Vehicles** | 4.8M | Improved Stack Assembly | Launch Vehicle: **id_Rocket_RocketType3** | By adding precise guidance software and controllable surfaces to our launch vehicles, we can allow them to perform a powered landing after a successful launch, letting us use each vehicle more than once. |
| **Nuclear Launch Vehicles** | 5.5M | Early Launch Systems<br>Closed-Cycle Gas Core | Launch Vehicle: **Pelican** | Construction of new launch vehicles, augmented with nuclear thermal rockets. |
| **Optimized Heat Shields** | 5.5M | — | — | One of the greatest limiters for full launch vehicle recovery is the need for reentry on the upper stages. A better, more durable heat shield can greatly reduce stress on the vehicle. |
| **Improved Reusable Rockets** | 5.5M | Reusable Launch Vehicles | +25 BuildCost on id_Rocket_RocketType3, id_Rocket_RocketType4 | With our expertise in operating them, we can significantly optimize the cost and time required to construct reusable rockets. |
| **Condor** | 6.5M | Polynitrogen Fuel<br>Reusable Launch Vehicles | Launch Vehicle: **Condor** | Advanced chemical launch vehicle powered by polynitrogen fuel. |
| **Magpie** | 6.5M | Nuclear Launch Vehicles | Launch Vehicle: **Magpie** | Powerful launch vehicle using gas core nuclear engines. |
| **Teratorn** | 7.2M | Magpie | Launch Vehicle: **Teratorn** | The largest and most powerful launch vehicle devised, the Teratorn can lift thousands of tons and quickly come back to lift another payload. |
| **Advanced Payload Fairings** | 7.2M | Improved Reusable Rockets<br>Magpie | +25 MaxPayloadOnCurrentObject on lv_nuke_small, lv_nuke_mid, lv_nuke_large, lv_chemadvanced | Larger spacecraft can carry more cargo, and so our launch vehicles need even more space to deliver it to orbit. |

### Material

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Advanced Materials** | 144k | — | — | Advanced Material Science |
| **Materials** | 648k | — | Builds **Alloy Smelting** | Material Science |
| **Space-worthy Fibers** | 1.2M | Materials | Builds **Polymers Production** | Our first ventures outside the atmosphere have made us realize that to protect astronauts during excursions we need better fibers to create outfits that can survive outer space. |
| **Space Alloys** | 1.2M | Materials | +5 ProductionEfficiency on build_alloysmelting | The harsh environment of space requires us to adapt our materials. |
| **Debris Shields** | 1.2M | — | — | A set of spaced protective layers can protect against micrometeor impacts far better and with far less mass than previous solutions. |
| **Circuit Production In Space** | 2.2M | Space Alloys | Builds **Electronics Factory** | While a state of vacuum in most of the solar system seems to make creating clean rooms easier, the lower gravity and ever-present static charge build-up still make it a significant challenge to produce electronics off-world. Overcoming those challenges will let us move the supply chain further upwell. |
| **Graphene** | 2.2M | Space-worthy Fibers | +15 ProductionEfficiency on build_polymerproduction | A form of carbon similar to graphite, with incredible properties. |
| **Vacuum Forging** | 2.2M | — | — | Space brings new challenges and new opportunities in forging. By harnessing the natural conditions of space we can create better metals. |
| **Aerogels** | 2.2M | — | — | Extremely lightweight materials with excellent insulating capacity and strength per unit mass. |
| **Improved Electronics Production** | 3.0M | Circuit Production In Space | +25 ProductionEfficiency on build_electronicsfactory | Adapting our space production methods further, we can improve the output of our circuit manufacturing. |
| **Nanotubes** | 3.0M | Graphene | +15 ProductionEfficiency on build_polymerproduction | Graphene arranged into lengths of tubes. While similar, it can provide many properties that graphene cannot. |
| **Advanced Alloying Techniques** | 3.0M | Space Alloys | Builds **Exotic Alloy Production** | New challenges bring new needs, and our material science divisions work tirelessly to develop new, better alloys. |
| **Diamondoids** | 4.1M | Nanotubes | +10 BuildCost on Facility | Molecules with diamond-like structures can provide far greater material strength and new properties. |
| **Mega-scale Carbon Allotrope Application** | 4.1M | — | — | While new carbon materials have allowed us to revolutionize material science, to complete ambitious megaprojects we dream of, we need to be prepared to apply them on a scale never heard of before. |
| **Improved Alloy Production** | 4.1M | Advanced Alloying Techniques | +10 ProductionEfficiency on build_exoticalloy, build_alloysmelting | Optimized procedures and equipment increase yields of our factories. |

### Mining

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Mining** | 648k | — | — | Extraction of resources from celestial objects |
| **In-Situ Resource Utilization** | 1.2M | Mining | Builds **Metal Mining Base** | We cannot bring everything we need where we're going. We have to be ready to extract what's needed directly from the ground we step on. |
| **Regolith Sifting** | 2.2M | In-Situ Resource Utilization | +25 MiningEfficiency on All | While rich ores cannot be always found in space, we can always extract valuable elements by sifting through the ever-present regolith. |
| **Vacuum-Optimized Boreheads** | 2.2M | In-Situ Resource Utilization | +25 MiningEfficiency on All | Better boreheads designed for operation in a vacuum environment. |
| **Radioactive Isotope Isolation** | 3.0M | Regolith Sifting | Builds **Fissiles Mine** | Nuclear power is essential for our continued presence in space. We need to be prepared for difficulties of extracting and enriching radioactive elements necessary for it on site. |
| **Advanced Mining Methods** | 3.0M | Vacuum-Optimized Boreheads | +25 MiningEfficiency on All | Further improvements in mining equipment and procedures increase yields. |
| **Helium-3 Extraction** | 4.1M | Regolith Sifting<br>Fusion Theory | Builds **Helium-3 Extractor** | While sadly there aren't any viable ores of precious Helium-3, it can still be found dispersed in certain environments. As we extract other resources, it is important to make sure that traces of this valuable isotope are not lost among the tailings. |
| **Improved Fissiles Mining** | 4.1M | Radioactive Isotope Isolation | +25 MiningEfficiency on build_uranmine | Better isotope separation facilities lead to a faster mining process. |
| **Improved Helium-3 Mining** | 4.8M | Helium-3 Extraction | +75 MiningEfficiency on build_he3mine | With our experience with extracting Helium-3 in space, we can improve our mining facilities. |

### Spacecraft

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Advanced Spacecraft** | 72k | — | — | Dedicated space vehicles for travel between orbits. |
| **Stratos** | 144k | Spacecraft | Spacecraft: **Stratos** | Powerful upper stage capable of independent operation in space, ideal workhorse for the moon and beyond. |
| **Selene** | 432k | Iris | Spacecraft: **Selene** | Light craft with an electric engine for moving probes and small cargo to distant objects. |
| **Solar Sails** | 648k | Space-worthy Fibers | Spacecraft: **Daedalus** | Solar Sails operate using nothing but the sun's energy to maneuver through space. Though slow and limited by distance from the sun, they require no fuel or propellant. |
| **Spacecraft** | 648k | Early Launch Systems | — | Dedicated space vehicles for travel between orbits. |
| **Hermes** | 864k | Stratos | Spacecraft: **Hermes** | Dedicated deep space transport, with artificial gravity, life support capacity, and a powerful array of ion engines. |
| **Optimized Payload Bus** | 1.2M | Solar Sails | +100 ComponentCargoCapacity on cargo_solar_small | Lighter, more efficiently shaped bus will allow us to load more cargo in the same sail craft. |
| **Iris** | 1.2M | Spacecraft | Spacecraft: **Iris** | A simple probe carrier craft. |
| **Expanded Cargo Bays 1** | 2.2M | Spacecraft<br>Stratos | +50 SCCargoCapacityBase on spacecraft_chem_mid2, spacecraft_chem_large | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| **Prometheus** | 2.2M | Spacecraft<br>Nuclear Propulsion | Spacecraft: **Prometheus** | The first true nuclear rocket, more than double the efficiency of chemical rockets while still maintaining high thrust |
| **Reflective Materials** | 3.0M | Optimized Payload Bus<br>Graphene | Spacecraft: **Talos** | The propulsive effect of solar sails is achieved by reflection of photons emitted by the sun. Material improvements in reflectivity allow construction of even better sail based spacecraft. |
| **Expanded Cargo Bays 2** | 3.0M | Expanded Cargo Bays 1<br>Hermes | +20 SCCargoCapacityBase on spacecraft_electric_small, spacecraft_electric_mid, spacecraft_nuke_small | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| **Athena** | 3.0M | Hermes<br>MPD | Spacecraft: **Athena** | Advanced electric spacecraft, designed for slow but efficient travel |
| **Improved Sail Deployment** | 4.1M | Reflective Materials | +20 BuildCost on spacecraft_sail_small, spacecraft_sail_mid | A major issue with launching sail-based spacecraft is unfurling the sail without damaging it. Optimal performance is achieved by the thinnest sail possible, which makes them fragile. Better techniques of deploying the sail would make it far easier to construct and launch. |
| **Hephaistos** | 4.1M | Prometheus<br>Liquid Core | Spacecraft: **Hephaistos** | Nuclear thermal electric spacecraft, capable of significantly cutting down travel time between planets |
| **Plasma Magnet Sail** | 4.8M | Improved Sail Deployment<br>Superconducting Magnets | Spacecraft: **Zephyr** | The plasma magnet sail replaces huge stretches of fabric with thin loops of superconducting wire that uses the charged particles of the solar wind itself to generate a far larger magnetic sail, enabling it to fly faster and farther than solar sails. |
| **Expanded Cargo Bays 3** | 4.8M | Expanded Cargo Bays 2<br>Hephaistos | +20 SCCargoCapacityBase on spacecraft_nuke_mid, spacecraft_nuke_large, spacecraft_nuke_nolv | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| **Ariane** | 5.5M | Hephaistos<br>Closed-Cycle Gas Core | Spacecraft: **Ariane** | Most powerful nuclear thermal spacecraft, powered by a Nuclear Lightbulb" engine." |
| **Cronos** | 5.5M | Hephaistos<br>Closed-Cycle Gas Core | Spacecraft: **Cronos** | Single-Stage-To-Orbit spacecraft powered by a closed-cycle gas core nuclear thermal rocket engine. A set of seven engines lets this colossus lift itself and a kiloton of payload into earth orbit and beyond. |
| **Nike** | 6.5M | Spacecraft<br>Fusion Propulsion | Spacecraft: **Nike** | First generation fusion spacecraft, with efficiency unmatched by previous designs. |
| **Expanded Cargo Bays 4** | 7.2M | Expanded Cargo Bays 3<br>Nike | +20 SCCargoCapacityBase on spacecraft_fusion_small, spacecraft_fusion_mid, spacecraft_fusion_large | Improvements on our spacecraft designs allow increasing their cargo capacity. |
| **Sirius** | 8.2M | Nike<br>Fuel Injection Optimization | Spacecraft: **Sirius** | Advanced fusion spacecraft. |
| **Atlas** | 9.1M | Sirius<br>Magnetic Spin Alignment | Spacecraft: **Atlas** | Enormous spacecraft able to move entire asteroids. |
| **Zeus** | 9.1M | Sirius<br>Magnetic Spin Alignment | Spacecraft: **Zeus** | Operating at the limits of theoretical performance of fusion spacecraft, the Zeus can fly farther and faster than any other. |

## Physics

### Computing

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Computing** | 648k | — | — | The study of computer science and construction of processing units. |
| **Microprocessors** | 1.2M | Computing | — | Miniature processing units, freeing us of the need for enormous computer frames. |
| **Nanoprocessors** | 2.2M | Microprocessors | +4 ResearchProduction | Further miniaturization of processing units greatly increases processing power, allowing larger amounts of data to be analyzed, and more complex simulations to be run. |
| **Early QPUs** | 2.2M | — | +0 ResearchProduction | Early advancements in the construction of Quantum Processing Units allows experimental usage of new kinds of computing for specialized data processing tasks. However, the practical applications of such remain limited due to the substantial difficulty in producing circuits that can avoid decoherence. |
| **Machine Learning** | 2.4M | Nanoprocessors | +4 ResearchProduction | Statistical models capable of studying and extrapolating from data, potentially solving tasks where writing algorithms is unfeasible. |
| **Neural Networks** | 3.0M | — | +0 ResearchProduction | A type of machine learning inspired by the mechanism of biological neurons, Neural Networks map nodes and their connection as an analog of neurons and are shaped by further training on data. |
| **Basic AI** | 3.0M | Machine Learning | +5 ResearchProduction | As the size of our learning models grows, so do their capabilities. Impressive pattern recognition when trained on controlled samples can be harnessed for unparalleled processing of data. |
| **Three-Dimensional Integrated Circuits** | 3.0M | — | +0 ResearchProduction | As ever smaller circuits run into physical limits of size, further increases in density of processing power can be achieved by expanding existing circuits vertically, into the third dimension. |
| **Reversible Circuits** | 4.1M | — | +0 ResearchProduction | A special kind of computer circuitry operating on reversible logic, that is logic in which all inputs can be determined based on their outputs. Reversibility of the process means that numerous calculations can be done with minimal entropic losses, greatly increasing energy efficiency. |
| **Quantum Computing** | 4.1M | Nanoprocessors | +8 ResearchProduction | Practical implementation of quantum mechanical effects into computing greatly increases capacity for data processing and physical simulations. Although quantum computers cannot replace classical computing in numerous tasks, they can greatly exceed them in others, supplementing existing computing methods. |
| **Advanced AI** | 4.8M | Basic AI | +8 ResearchProduction | Refinements, iterations, and increased processing power produce even more capable models. |

### Electromagnetism

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Electromagnetism** | 648k | — | — | The study of electricity and magnetism. |
| **Phased-Arrays** | 864k | — | — | A set of radio antennas controlled by a computer, capable of being steered in a specific direction without moving the antenna itself. |
| **Atomic Manipulation** | 1.2M | Electromagnetism | +20 ProductionEfficiency on build_exoticalloy | The process of manipulating individual atoms through the use of a Scanning Tunneling Microscope. This can be used to create atomic-scale structures or unique molecules. |
| **Lasers** | 1.2M | — | — | Precisely controlled emitters of coherent light. |
| **High-Temperature Superconductors** | 1.2M | Electromagnetism | +10 PowerProduction on All | High-temperature superconductors are a form of superconductor capable of operating above the boiling point of nitrogen, greatly reducing the need for extensive cooling. |
| **Superconducting Magnets** | 2.2M | Electromagnetism<br>High-Temperature Superconductors | +10 MaintenanceReduce on Facility | Utilization of superconductors in electromagnets can greatly improve their performance. |
| **Room-Temperature Superconductors** | 2.2M | High-Temperature Superconductors | +20 MaintenanceReduce on Facility | The holy grail of superconductivity, Room-Temperature Superconductors can achieve the state of superconductivity at ambient temperatures. |

### Electroprop

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Electric Propulsion** | 648k | — | — | Highly efficient but low in thrust, electric drives can handle great distances but are limited by their need for power. |
| **Hall Thruster** | 1.2M | Electric Propulsion | +35 ComponentExhaustV on eng_electric, eng_electricmpd, eng_electrichermes | Ions are accelerated outside this thruster through the use of the Hall Current, giving it its name. |
| **Ion Engine** | 1.7M | — | — | Electrostatic acceleration of argon seeded with potassium through a grid of electrodes provides thrust in this engine. |
| **MPD** | 2.2M | Hall Thruster | +50 ComponentExhaustV on eng_electric, eng_electrichermes, eng_electricmpd | Magnetoplasmadynamic thrusters utilize the Lorentz force to propel ionized propellant through a generated magnetic field, at velocities far greater than achievable in gridded ion thrusters. |
| **Optimized Self-field** | 3.0M | MPD | +40 ComponentExhaustV on eng_electric, eng_electrichermes, eng_electricmpd | A more optimized self-field in an MPD can improve its performance. |
| **Electrodeless Ion Generation** | 3.5M | — | — | By removing electrodes utilized in an ion engine, as well as redesigning for certain efficiencies, we can greatly improve its performance without worrying about corrosion. |
| **Multi-stage Grid** | 6.9M | — | — | Additional grid stages of accelerating grids in the engine allow us to increase the exit velocity of ions. |

### Exploration

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Observation** | 648k | — | — | Astronomy, observation, and study of celestial objects |
| **Improved Optics** | 1.2M | Observation | +30 ObservationSpeed | Improvements in the creation of large lenses can help us observe distant objects. |
| **Radar Astronomy** | 1.7M | — | +30 ObservationSpeed | Although RADAR had been invented before WW2 and early observations of the moon through it took place soon after the war, it was not until recently that this technique could be reliably utilized to observe and measure bodies within our solar system. By investing in this technology further, we may find out much more about our planetary neighborhood. |
| **Space Telescopes** | 2.2M | Improved Optics | Builds **Telescope** | Ground observations are severely limited by interference from our atmosphere. By placing a remote telescope observatory outside of Earth's atmosphere, we can observe things we couldn't before. |
| **Exoplanet Detection** | 3.0M | Space Telescopes | — | By using various techniques that detect variations in a star's characteristics, we can spot and estimate potential exoplanets circling those distant stars. |

### Fusion

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Fusion Theory** | 3.0M | Nuclear Power<br>Superconducting Magnets | — | The use of nuclear fusion to generate power. |
| **Magnetic Field Containment** | 4.1M | Fusion Theory | Builds **Fusion Reactor** | In our quest to achieve nuclear fusion, one of the greatest problems is containing hot fusion fuel plasma inside a magnetic field, at pressures great enough to allow ignition. |
| **Magnetic Nozzle** | 4.8M | — | — | As we create more powerful engines, the ever-greater temperatures put more stress on the material of our nozzles. While ever-greater cooling could potentially solve the issue, another concept is to do away with a physical nozzle entirely, instead containing the exhaust energies within magnetic fields. |
| **Fusion Propulsion** | 5.5M | Magnetic Field Containment | +10 PowerProduction on build_power_fusion | The application of all our fusion knowledge finally lets us harness it to power our spacecraft. |
| **Improved Plasma Containment** | 6.5M | Fusion Propulsion | +100 ComponentExhaustV on eng_fusion, eng_fusionnike | By working out inefficiencies in our magnetic nozzles we can more completely capture all the energy from fusion reactions, improving engine performance. |
| **Fuel Injection Optimization** | 7.2M | Improved Plasma Containment | +100 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius | An important element of fusion reactions is to supply fuel at optimal rate and direction to achieve as high of a burn up as possible. Improved fuel flow into the reactor chamber guarantees better efficiency. |
| **Magnetic Spin Alignment** | 8.2M | Fuel Injection Optimization | +200 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius | The holy grail of fusion propulsion. By precisely aligning the spin of individual fuel particles, we can control the direction of resulting neutron radiation, turning previously wasted energy into additional thrust. The reduction in required shielding, and additional exhaust energy massively improves performance of our fusion spacecraft. |
| **Advanced Fusion Propulsion** | 9.1M | Magnetic Spin Alignment | +25 ComponentExhaustV on eng_fusion, eng_fusionnike, eng_fusionsirius, eng_fusionzeus, eng_fusionatlas | Further optimising our engines, we can improve their performance even more. |

### Nuclear

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Nuclear Power** | 648k | — | Builds **Nuclear Reactor** | The use of nuclear fission to generate power. |
| **High Temperature NTR** | 864k | — | — | We can increase the performance of our engines by keeping the fuel elements close to the melting point, increasing the power available for thrust. |
| **Helium Loop Cycle** | 1.2M | — | — | An additional heat transfer system utilizing helium can be used to run a power-generating turbine, or to power a turboinductor device, transferring additional power to propellant. |
| **Nuclear Propulsion** | 1.2M | Nuclear Power | — | Nuclear Thermal Rockets utilize the high energies of nuclear reactors to accelerate propellant to far greater speeds than in chemical engines while maintaining good thrust. |
| **Improved Reactor Vessels** | 1.2M | Nuclear Power | — | To handle greater temperatures, new alloys that can handle thermal loads beyond those of known elements need to be researched. |
| **Molten Fission Systems** | 2.2M | Improved Reactor Vessels | +50 PowerProduction on build_power_nuke | Nuclear reactors operating at temperatures so high that the fuel elements become molten can achieve far greater power outputs. |
| **Gas Core Fission** | 3.0M | Molten Fission Systems | +50 PowerProduction on build_power_nuke | Even greater power densities can be achieved in nuclear reactors, however this requires temperatures high enough to turn fissile elements into gas. Contained within an actively cooled pressure vessel, rapidly fissioning gas gives off tremendous power - as long as it is contained. |
| **Liquid Core** | 3.0M | Nuclear Propulsion<br>Molten Fission Systems | +100 ComponentExhaustV on eng_nuke, eng_nukeprometheus | Previously our nuclear rockets were limited by the melting point of uranium. With systems designed to handle liquid nuclear fuel, we can reach beyond that barrier and utilize even greater power in our rockets. |
| **Nuclear Thermal Electric Rocket** | 4.1M | Liquid Core | +100 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | Using a two-stage system of cooling loops and heat engines, we can more efficiently transfer heat to our propellant while also generating substantial power for an electric afterbooster, significantly increasing the efficiency of our nuclear rockets. |
| **Closed-Cycle Gas Core** | 4.8M | Nuclear Thermal Electric Rocket<br>Gas Core Fission | +75 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | Fissile gas is reacted inside a transparent pressure vessel - a "Nuclear Lightbulb" - emitting an enormous amount of power transferred to hydrogen propellant. |
| **MPD-Based NTER** | 10.4M | — | +50 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus, eng_nukeariane | By utilizing Magnetoplasmadynamic technology in place of the arcjet, we can further boost the efficiency of our nuclear engines. |
| **Advanced Liquid Core** | 13.8M | — | +50 ComponentExhaustV on eng_nuke, eng_nukenter, eng_nukeprometheus | An actively cooled rotating fuel drum keeps the uranium in contact with hydrogen propellant while preventing losses of fuel through the exhaust, thanks to centrifugal force. |

### Power

| Research | Cost (work hours) | Prerequisites | Unlocks | Description |
| --- | --- | --- | --- | --- |
| **Power** | 648k | — | — | Provides electricity for planetary bases. |
| **Geothermal Power Turbine** | 1.2M | Power | Builds **Geothermal Power** | Active planetary geology provides thermal gradients that can be utilized to generate power. |
| **Solar Cell Arrays** | 1.2M | Power | Builds **Solar Array** | Utilization of the photoelectric effect for power generation, solar cells have become a ubiquitous form of power supply for spacecraft. |
| **Wind Turbine** | 1.2M | Power | Builds **Wind Power** | Wind power generators, similar to ones on Earth, designed for vastly different planetary conditions and ease of construction on site. |
| **Chemical Power Reactor** | 1.3M | — | Builds **Chemical Reactor** | On planets with abundant hydrocarbon atmospheres such as Titan, energy can be produced from chemical reactions of atmospheric compounds. Hydrogenation of acetylene (or if lacking, ammonia) generates sufficient heat to produce net power. |
| **Remote Power Transfer** | 2.2M | Power | Builds **Remote Power Receiver** | By utilizing coherent microwave beams and rectennas, we can transmit large amounts of electrical power from far away or even through the atmosphere. |
| **Improved Solar Cells** | 2.2M | Solar Cell Arrays | +50 PowerProduction on build_power_solar | Better materials and technologies allow us to greatly increase the power density of solar cells. |
| **Powersat** | 3.0M | Power<br>Remote Power Transfer | Builds **Orbital Power Station** | Large solar collectors in orbit, capable of transferring power down to the surface or other spacecraft by the use of microwave beams. |

## Reading the table

- **Cost** is in work hours and is divided by your laboratories' research output to get the actual research time in days.
- **Prerequisites** must be completed before the node becomes available.
- **Unlocks** — *Builds X* means the node makes a new facility constructable; *Spacecraft / Launch Vehicle* means the node unlocks a new ship or lifter; numeric bonuses apply to listed components.

## See also

- [Spacecraft](../spacecraft/) — propulsion research feeds directly into these
- [Launch Vehicles](../launch-vehicles/)

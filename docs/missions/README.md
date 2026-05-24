# Missions

This page covers two related concepts, both of which the game calls
"missions" depending on context.

1. **Contracts** — the in-game *Contracts* tab. See [Contracts](../contracts/)
for the full list and dependency chain.
2. **Flight missions** — an individual scheduled trip you plan in Plan
Mission (Earth → Mars on day N).  Flight missions are runtime state,
not static data — see the **planning flow** section below for how to
set one up.

## Planning flow

Plan Mission walks you through five steps:

1. **Destination** — pick the target body (and landing type if applicable).
2. **Spacecraft** — pick the craft to send.
3. **Cargo** — pick the payload.
4. **Launch Vehicle** — pick the lifter (only required for missions launching from a planet's surface).
5. **Flight Plan** — pick the launch and arrival windows from the porkchop plot.

### Mission types

Mission-type labels surfaced by the in-game Plan Mission window (locale
keys `Game.UI.Windows.Windows.PlanMissionWindow.*`):

- **Direct flight** (the default Plan Mission flow)
- **Gravity Assist** (toggle in Plan Mission — `PlanMission.GravityAssistOn`)
- **Cyclical Mission** (`UI.WindowMain.Layers.CycleMission`, planned via the *Plan Cyclical Mission* window)
- **Asteroid Pulling** (specialised mission scheduled from an Asteroid Engine Module)
- **Probe Deployment** (probe payload dropped on arrival)

The mechanics behind each type aren't captured in the shipped data files;
see the in-game UI for the authoritative behaviour.

For launch-window timing for any destination, see [Launch Windows](../celestial-bodies/launch-windows.md).

## See also

- [Contracts](../contracts/)
- [Spacecraft](../spacecraft/)
- [Launch Vehicles](../launch-vehicles/)
- [Launch Windows](../celestial-bodies/launch-windows.md)

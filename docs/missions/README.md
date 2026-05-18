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

### Mission types (from in-game UI)

| Type | Notes |
| --- | --- |
| **Direct** | Single Hohmann-style transfer to the destination. |
| **Gravity Assist** | Uses another body's gravity to bend the trajectory and save Δv. The game lets you choose whether cargo drops at the assist target or continues on. |
| **Cyclical** | A repeating supply route between two or more bodies. |
| **Asteroid Pulling** | Specialised mission to push an asteroid into a different orbit using an Asteroid Engine Module. |
| **Probe Deployment** | Drops a small probe at a destination (typically the first thing you send anywhere). |

For launch-window timing for any destination, see [Launch Windows](../celestial-bodies/launch-windows.md).

## See also

- [Contracts](../contracts/)
- [Spacecraft](../spacecraft/)
- [Launch Vehicles](../launch-vehicles/)
- [Launch Windows](../celestial-bodies/launch-windows.md)

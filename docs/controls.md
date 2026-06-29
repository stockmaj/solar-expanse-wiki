# Controls

Keyboard shortcuts extracted from the game's source code and confirmed present in the active game scenes.

## Camera

Configured in the `InputData` ScriptableObjects and handled by `InputManager`. Hold a key to move continuously. Hold **Left Shift** at the same time to move at 4× speed.

| Key | Alternate | Action |
| --- | --- | --- |
| W | Up Arrow | Pan camera up |
| S | Down Arrow | Pan camera down |
| A | Left Arrow | Pan camera left |
| D | Right Arrow | Pan camera right |
| Q | — | Lower camera elevation (orbit view angle) |
| E | — | Raise camera elevation (orbit view angle) |

## Time

| Key | Action |
| --- | --- |
| Space | Pause / unpause simulation |
| 1 | Set time speed 1 |
| 2 | Set time speed 2 |
| 3 | Set time speed 3 |
| 4 | Set time speed 4 |
| 5 | Set time speed 5 |

The five speed levels are multipliers of the game's base time scale — their exact real-time ratios are scaled from the economic configuration and vary.

## UI

| Key / input | Action |
| --- | --- |
| Escape | Close the current open window |
| Left Shift + click | Open a second info panel for the clicked object (Shift+click on a second different object updates that panel) |
| Left Shift + numeric up/down button | Increment or decrement by 10 instead of 1 |

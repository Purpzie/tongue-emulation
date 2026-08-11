# Tongue Emulation for VRChat
Notably, the Quest Pro lacks directional tongue tracking.
This tiny program moves your tongue around using other parts of your face.

Left and right are controlled by your jaw.
Up is controlled by lip pucker, and down is controlled by jaw open.

## Avatar support
Currently, this only works for avatars with these parameters (such as [Jerry's Template](https://github.com/Adjerry91/VRCFaceTracking-Templates)):
- `FT/v2/TongueX` (float)
- `FT/v2/TongueX1` (bool)
- `FT/v2/TongueX2` (bool)
- `FT/v2/TongueX4` (bool)
- `FT/v2/TongueXNegative` (bool)
- `FT/v2/TongueY` (float)
- `FT/v2/TongueY1` (bool)
- `FT/v2/TongueY2` (bool)
- `FT/v2/TongueY4` (bool)
- `FT/v2/TongueYNegative` (bool)

The latest version of [Pawlygon's Template](https://github.com/PawlygonStudio/VRC-Facetracking) works, but you'll need to [edit the avatar's OSC config](https://docs.vrchat.com/docs/osc-avatar-parameters#avatar-parameters--config-files) to remap parameters to the names above.

## Optional Puppet
Avatars can override emulation with a two axis puppet.
- Parameter: `TongueEmulation/PuppetActive` (bool)
- Horizontal: `TongueEmulation/PuppetX` (float)
- Vertical: `TongueEmulation/PuppetY` (float)

Don't mark them as synced!

## Configuration
VRChat's OSC sockets are used by default.
You can change them by passing these arguments:

```
tongue-emulation.exe [listening socket] [sending socket]
```
Example with defaults:
```
tongue-emulation.exe 127.0.0.1:9001 127.0.0.1:9000
```

None of these are guaranteed, but they'd be nice to have.

- [ ] Detect and respond to any VRCFT-compatible avatar parameters, not just Jerry's template
- [ ] Use OSCQuery to avoid taking control of VRC's OSC socket
  - Nothing should ever leave localhost
  - [vrchat_osc](https://docs.rs/vrchat_osc) is an option, but would require a full rewrite of TongueEmulation.
    Also, it doesn't take references when sending for some reason, so that would cause unnecessary allocations.
  - Alternatively, OSC router programs exist and people can use one of those instead.
- [ ] Attenuate tongue in-out with jaw open
  - Tongue down currently uses jaw open. What should it use instead? Or maybe it would just need more to activate?
  - VRCFT writes to tongue out already. It might be possible to edit the avatar's config to send VRCFT to a dummy parameter, which then gets read, modified, and written to the real ones. This would break your tongue if TongueEmu isn't running, though...
- [ ] Customizable parameter remapping
- [x] Optional ingame avatar menu for extra control and puppeting

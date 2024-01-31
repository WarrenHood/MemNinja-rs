# HoodMem

A WIP memory hacking crate.

## Supported Platforms
- Windows (Run as Administrator)
- Linux (Run `memninja` with `sudo -EH` if on Wayland)

## This crate aims to eventually support the following features:

- Attach to a process by PID or window name
- Get a list of "writable" process memory regions
- Read arbitrary process memory
- Write arbitrary process memory
- Memory scanning
- Scripting (Lua)
- Spawn threads in a process
- Inject DLLs into a process (Windows only)
- TODO: Think of more things

It'd be nice to be able to write game hacks in Rust rather than C++.
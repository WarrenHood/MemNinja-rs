# HoodMem

A WIP Windows memory hacking crate.

## This crate aims to eventually support the following features:

- Get a `HANDLE` to a process by PID or window name
- Get a list of "writable" process memory regions
- Read arbitrary process memory
- Write arbitrary process memory
- Memory scanning
- Scripting (Lua)
- Spawn threads in a process
- Inject DLLs into a process
- TODO: Think of more things

It'd be nice to be able to write game hacks in Rust rather than C++.
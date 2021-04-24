# Rust DFU

Command Line utility for programming devices using the DFU protocol. The tool
is written in Rust, and is intended to be _very_ simple to use. 

This is currently very much a work in progress project, not yet even remotely complete.

The goal is to support the following source formats:
- BIN files
- Intel HEX files
- ELF files
- DFU files

Further the main goal is to be able to automatically detect DFU devices and their memory layout.
This is sadly not part of the DFU protocol itself, and will likely have to depend on non-generic
implementations depending on the device type. The goal is to support:
- Devices using a STM32 compatible DFU implementation (memory layout given via Endpoint descriptors)
- ...


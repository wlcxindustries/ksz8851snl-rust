# KSZ8851SNL Low Level Driver
This is a `no-std` Rust low-level driver for the Microchip [KSZ8851SNL] SPI ethernet controller 

There are two broad parts to this crate.
- Register definitions, using [embedded-registers]
- Low level driver functions implementing TX/RX and configuration operations

** Warning! This is still very much beta-quality. It works, but isn't battle tested and isn't as flexible as it should be **

[embedded-registers]: https://docs.rs/embedded-registers/latest/embedded_registers/
[KSZ8851SNL]: https://www.microchip.com/en-us/product/KSZ8851

# KSZ8851SNL Low Level Driver
This is a `no-std` Rust low-level driver for the Microchip [KSZ8851SNL] SPI ethernet controller

There are two broad parts to this crate.
- Register definitions and accessors, using [device-driver]
- Low level driver functions implementing TX/RX and configuration operations

> [!WARNING]
> This is still very much beta-quality. It works, but isn't battle tested and isn't as flexible as it should be!

[device-driver]: https://docs.rs/device-driver/latest/device_driver/
[KSZ8851SNL]: https://www.microchip.com/en-us/product/KSZ8851

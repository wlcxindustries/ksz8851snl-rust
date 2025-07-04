//! Low level register descriptions and driver for the KSZ8851SNL SPI Ethernet controller"
#![no_std]
// FIXME: can be removed when this stabilises in 1.89 (hopefully?)
#![feature(generic_arg_infer)]
pub mod driver;
pub mod registers;

# MPR121-hal

[![crates.io](https://img.shields.io/crates/v/mpr121-hal.svg)](https://crates.io/crates/mpr121-hal)
[![docs.rs](https://img.shields.io/docsrs/mpr121-hal?style=plastic)](https://docs.rs/mpr121-hal/latest/)

[Mpr121](https://www.nxp.com/docs/en/data-sheet/MPR121.pdf) proximity capacitive touch sensor controller driver. Used for instance on the [Adafruit Mpr121 module](https://www.adafruit.com/product/1982).

## Overview

The crate uses the common [embeded-hal](https://crates.io/crates/embedded-hal) crate to provided a generic implementation for multiple HALs.

The implementation provides a similar API to the C++ Adafruit library. It should work with similar boards using the Mpr121 as well.

## Example

<!-- TODO
The esp32-hal example creates the I²C master and tries to read sensor data on pin 1 and 2. Build and run via:
```shell
cargo espflash --release /dev/ttyUSB0 && espmonitor /dev/ttyUSB0
```

-->
## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

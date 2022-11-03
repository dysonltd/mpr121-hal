# FS1027-DG-hal

[![crates.io](https://img.shields.io/crates/v/fs1027-dg-hal.svg)](https://crates.io/crates/fs1027-dg-hal)
[![docs.rs](https://img.shields.io/docsrs/fs1027-dg-hal?style=plastic)](https://docs.rs/fs1027-dg-hal/latest/)

[FS1027-DG Gas Flow Sensor](https://www.renesas.com/us/en/document/dst/fs1027-dg-datasheet?language=en&r=1488711) Module driver for no_std embedded-hal

## Overview

The crate uses the common [embeded-hal](https://crates.io/crates/embedded-hal) crate to provided a generic implementation for multiple HALs.

It implements value verification as well as conversion into liters/min.

## Example

The esp32-hal example creates the I²C master and tries to read sensor data on pin 1 and 2. Build and run via:
```shell
cargo espflash --release /dev/ttyUSB0 && espmonitor /dev/ttyUSB0
```

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

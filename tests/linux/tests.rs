use crate::i2c_driver;
use tests_common::*;

pub async fn test_create_mpr121() {
    let i2c_bus = i2c_driver::setup_i2c().expect("I2C Bus failed to acquire");
    generic_test_create_mpr121(i2c_bus);
}

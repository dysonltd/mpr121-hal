#![no_std]
use embedded_hal::i2c::I2c;
use mpr121_hal::mpr121::Mpr121;

#[maybe_async::maybe_async]
pub async fn generic_test_create_mpr121<I2C>(i2c: I2C)
where
    I2C: I2c,
{
    let mpr121_sensor = Mpr121::new(i2c, mpr121_hal::Mpr121Address::Default, true, true).await;
    assert!(mpr121_sensor.is_ok());
}

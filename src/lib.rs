//! # MPR121-Hal
//!
//! This crate follows the [Adafruit implementation](https://github.com/adafruit/Adafruit_MPR121) closely but exposes it in terms of the embedded-hal project.
//!
//! The main responsibility of this crate is returning the current on/off state of all the (up to) 12 pins.
//!
//! The chip's data sheet can be found [here](https://www.nxp.com/docs/en/data-sheet/MPR121.pdf). The implementation however mostly mirrors the Adafruit implementation,
//! since this is probably the most widely used one.
//!
#![deny(
    unsafe_code,
//    warnings
)]
#![no_std]

extern crate embedded_hal as hal;
use core::fmt::Display;

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

const TOUCHSTATUS_L: u8 = 0x00;
const TOUCHSTATUS_H: u8 = 0x01;
const FILTDATA_0L: u8 = 0x04;
const FILTDATA_0H: u8 = 0x05;
const BASELINE_0: u8 = 0x1E;
const MHDR: u8 = 0x2B;
const NHDR: u8 = 0x2C;
const NCLR: u8 = 0x2D;
const FDLR: u8 = 0x2E;
const MHDF: u8 = 0x2F;
const NHDF: u8 = 0x30;
const NCLF: u8 = 0x31;
const FDLF: u8 = 0x32;
const NHDT: u8 = 0x33;
const NCLT: u8 = 0x34;
const FDLT: u8 = 0x35;

const TOUCHTH_0: u8 = 0x41;
const RELEASETH_0: u8 = 0x42;
const DEBOUNCE: u8 = 0x5B;
const CONFIG1: u8 = 0x5C;
const CONFIG2: u8 = 0x5D;
const CHARGECURR_0: u8 = 0x5F;
const CHARGETIME_1: u8 = 0x6C;
const ECR: u8 = 0x5E;
const AUTOCONFIG0: u8 = 0x7B;
const AUTOCONFIG1: u8 = 0x7C;
const UPLIMIT: u8 = 0x7D;
const LOWLIMIT: u8 = 0x7E;
const TARGETLIMIT: u8 = 0x7F;

const GPIODIR: u8 = 0x76;
const GPIOEN: u8 = 0x77;
const GPIOSET: u8 = 0x78;
const GPIOCLR: u8 = 0x79;
const GPIOTOGGLE: u8 = 0x7A;

const SOFTRESET: u8 = 0x80;


#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Mpr121Error{
    ///If an operation exceeds the channel count (typically 12).
    ChannelExceed,
    ///If a read operation failed, contains the address that failed.
    ReadError(u8),
    ///If a write operation failed, contains the address that failed.
    WriteError(u8),
    ///If sending the reset signal failed, contains the register that failed.
    ResetFailed{
        was_read: bool,
        reg: u8
    },
    ///If the reset did not happen as expected
    InitFailed,
}

///The four values the sensor can be addressed as. Note that the address of the device is determined by
/// where the `ADDR` pin is connected to. Default is used if no connection, or a connction to `VSS` is made.
///
/// Have a look at page 4 "serial communication" for further specification.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mpr121Address{
    Default = 0x5a,
    Vdd = 0x5b,
    Sda = 0x5c,
    Scl = 0x5d
}

///Main device definition.
pub struct Mpr121<I2C: Write + WriteRead> {
    i2c: I2C,
    addr: Mpr121Address,
}


impl<I2C: Write + WriteRead> Mpr121<I2C> {

    pub const DEFAULT_I2CADDR: u8 = 0x5a;
    pub const DEFAULT_TOUCH_THRESHOLD: u8 = 12;
    pub const DEFAULT_RELEASE_THRESOLD: u8 = 6;

    ///Creates the driver for the given I²C ports. Assumes that the I²C port is configured as master.
    /// If `use_auto_config` is set, the controller will use its auto configuration routine to setup
    /// charging parameters whenever it is transitioned from STOP to START mode.
    ///
    /// Note that we use the same default values as the Adafruit implementation, except for threshold values.
    /// Use [set_thresholds](Self::set_thresholds) to define those.
    pub fn new(i2c: I2C, addr: Mpr121Address, use_auto_config: bool) -> Result<Self, Mpr121Error> {
        let mut dev = Mpr121 {
            i2c,
            addr,
        };

        //reset
        dev.write_register(SOFTRESET, 0x63).map_err(
            |e| match e{
                Mpr121Error::ReadError(reg) => Mpr121Error::ResetFailed { was_read: true, reg },
                Mpr121Error::WriteError(reg) => Mpr121Error::ResetFailed{was_read: false, reg },
                _ => Mpr121Error::ResetFailed{was_read: false, reg: 0xff}
            }
        )?;
        //Stop
        dev.write_register(ECR, 0x0)?;
        //read config register
        let config = dev.read_reg8(CONFIG2)?;

        //Check if it is 0x24, which it should be according to the specification.
        // Otherwise bail.
        if config != 0x24{
            return Err(Mpr121Error::InitFailed);
        }

        //Initialise the device to the similar settings as Adafruit
        dev.set_thresholds(0x20, 0x15);

        //Setup Filters MHD==MaximumHalfDelta, NHD=NoiseHalfDelta
        // Have a look at 5.5 in the data sheet for more information.
        dev.write_register(MHDR, 0x01)?;
        dev.write_register(NHDR, 0x01)?;
        dev.write_register(NCLR, 0x0e)?;
        dev.write_register(FDLR, 0x00)?;

        dev.write_register(MHDF, 0x01)?;
        dev.write_register(NHDF, 0x05)?;
        dev.write_register(NCLF, 0x01)?;
        dev.write_register(FDLF, 0x00)?;

        dev.write_register(NHDT, 0x00)?;
        dev.write_register(NCLT, 0x00)?;
        dev.write_register(FDLT, 0x00)?;

        dev.write_register(DEBOUNCE, 0x0)?;
        dev.write_register(CONFIG1, 0x10)?;
        dev.write_register(CONFIG2, 0x20)?;

        if use_auto_config{
            dev.write_register(AUTOCONFIG0, 0x0b)?;

            //Use 3.3V VDD
            dev.write_register(UPLIMIT, 200)?; // = ((Vdd - 0.7)/Vdd) * 256;
            dev.write_register(TARGETLIMIT, 180)?; // = UPLIMIT * 0.9
            dev.write_register(LOWLIMIT, 130)?; // = UPLIMIT * 0.65
        }

        //enable electrodes and return to start mode
        let ecr_setting = 0b10000000 + 12; // enable all 12 electrodes
        dev.write_register(ECR, ecr_setting)?;

        Ok(dev)
    }

    ///Initializes the driver assuming the sensors address is the default one (0x5a).
    /// If this fails, consider searching for the driver.
    /// Or following the documentation on setting a driver address, and use [new](Self::new) to specify the address.
    ///
    /// Have a look at [new](Self::new) for further documentation.
    pub fn new_default(i2c: I2C) -> Result<Self, Mpr121Error> {
        Self::new(i2c, Mpr121Address::Default, false)
    }


    ///Set the touch and release threshold for all channels. Usually the touch threshold is a little bigger than the release
    /// threshold. This creates some debounce characteristics. The correct thresholds depend on the application.
    ///
    /// Have a look at [note AN3892]() of the mpr121 guidelines.
    pub fn set_thresholds(&mut self, touch: u8, release: u8){
        for i in 0..12{
            //Note ignoring false set thresholds
            let _ = self.write_register(TOUCHTH_0 + 2 * i, touch);
            let _ = self.write_register(RELEASETH_0 + 2 * i, release);
        }
    }

    ///Sets the count for both touch and release. See 5.7 of the data sheet.
    ///
    /// value must be 0..8, is clamped if it exceeds.
    pub fn set_debounce(&mut self, debounce_count: u8){
        let debounce = debounce_count.min(7);
        let bits = (debounce << 4) | (debounce);
        let _ = self.write_register(DEBOUNCE, bits);
    }

    ///Reads the filtered data form channel t. Noise gets filtered out by the
    /// chip. See 5.3 in the data sheet.
    ///
    /// Note that the resulting value is only 10bit wide.
    ///
    /// Note that 0 is returned, if `channel > 12`.
    pub fn get_filtered(&mut self, channel: u8) -> Result<u16, Mpr121Error>{
        if channel >  12{
            return Err(Mpr121Error::ChannelExceed);
        }

        self.read_reg16(FILTDATA_0L + channel * 2)
    }

    ///Reads the baseline data for the channel. Note that this has only a resolution of 8bit.
    ///
    /// Note that 0 is returned, if `channel > 12`, or reading failed
    pub fn get_baseline(&mut self, channel: u8) -> Result<u8, Mpr121Error>{
        if channel > 12{
            return Err(Mpr121Error::ChannelExceed);
        }

        //NOTE: the original reads a 8bit value and left shifts 2bit.
        //      While the shift is correct the data sheet mentions:
        //
        //      Although internally the baseline value is 10-bit,
        //      users can only access the 8 MSB of the 10-bit baseline value through the
        //      baseline value registers. The read out from the baseline register must
        //      be left shift two bits before comparing it with the 10-bit
        //      electrode data.
        //
        //      reading only 8bit and shifting 2bit effectively reduces the resolution to
        //      6bit, since we loose the 2MSB.
        //
        //      Therefore we read 16bit, mask out the top 6, and then shift
        let value = self.read_reg16(BASELINE_0 + channel)? & 0b00000011_11111100;
        let cast = (value << 2).try_into().unwrap_or(0);
        Ok(cast)
    }

    ///Reads the *touched* state of all channels. Returns a u16 where each bit 0..12 indicates whether the
    /// pin is touched or not. Use bit shifting / masking to generate a mask, or, if only one sensor's value is
    /// needed, use [get_touch_state](Self::get_touch_state).
    ///
    /// Returns 0 if reading failed.
    pub fn get_touched(&mut self) -> Result<u16, Mpr121Error>{
        //mask upper four bits returns the rest
        let unmasked = self.read_reg16(TOUCHSTATUS_L)?;
        Ok(unmasked & 0x0fff)
    }

    ///Returns the touch state of the given sensor.
    ///
    /// Returns false if `channel>11`, or reading failed.
    pub fn get_sensor_touch(&mut self, channel: u8) -> bool{
        if channel>11{
            return false;
        }

        //Masks all bits except for our channel, then returns true if the bit is set
        self.get_touched().unwrap_or(0) & (1 << channel) > 0
    }

    //Write implementation. Returns an error if a read or write operation failed. The error contains the failing register.
    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), Mpr121Error>{
        //MPR121 must be in Stop mode for most reg writes. This is not true for all, but
        // we are conservative here.
        let mut stop_required = true;
        //ECR and 0x73..0x71 don't need stop. makes this a bit faster
        if reg == ECR || (0x73 <= reg && reg <= 0x7a){
            stop_required = false;
        }
        //Check in which mode we are by reading ECR.
        let ecr_state = self.read_reg8(ECR)?;

        if stop_required{
            //set to stop
            self.i2c.write(self.addr as u8, &[ECR, 0x00]).map_err(|_| Mpr121Error::WriteError(ECR))?;
        }

        //actual write
        self.i2c.write(self.addr as u8, &[reg, value]).map_err(|_| Mpr121Error::WriteError(reg))?;

        //reset to old ecr state
        if stop_required{
            self.i2c.write(self.addr as u8, &[ECR, ecr_state]).map_err(|_| Mpr121Error::WriteError(ECR))?;
        }

        Ok(())
    }

    //Reads the value, returns Err, if reading failed.
    fn read_reg8(&mut self, reg: u8) -> Result<u8, Mpr121Error>{
        let mut val = [0u8];
        if let Err(_) = self.i2c.write_read(self.addr as u8, &[reg], val.as_mut_slice()){
            return Err(Mpr121Error::ReadError(reg));
        }
        Ok(val[0])
    }

    //Reads the value, returns Err, if reading failed.
    fn read_reg16(&mut self, reg: u8) -> Result<u16, Mpr121Error>{
        let mut val = [0u8, 0u8];
        if let Err(_) = self.i2c.write_read(self.addr as u8, &[reg], &mut val){
            return Err(Mpr121Error::ReadError(reg));
        }
        Ok(u16::from_le_bytes(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        unimplemented!()
    }
}

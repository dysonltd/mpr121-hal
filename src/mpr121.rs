#[cfg(feature = "sync")]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

use crate::{registers::*, NUM_TOUCH_CHANNELS};
use crate::{Mpr121Address, Mpr121Error};

pub struct Mpr121<I2C: I2c> {
    pub(crate) i2c: I2C,
    pub(crate) addr: Mpr121Address,
}

impl<I2C: I2c> Mpr121<I2C> {
    ///Creates the driver for the given I²C ports. Assumes that the I²C port is configured as master.
    ///
    /// If `use_auto_config` is set, the controller will use its auto configuration routine to setup
    /// charging parameters whenever it is transitioned from STOP to START mode.
    ///
    /// If `check_reset_flags` is set, the reset will be checked by reading back the 0x5C register. Note however, that
    /// sometime circuit configurations might be too slow/setup-incorrectly for that check. Which is why it is optional.
    ///
    /// Note that we use the same default values as the Adafruit implementation, except for threshold values.
    /// Use [set_thresholds](Self::set_thresholds) to define those.
    #[maybe_async::maybe_async]
    pub async fn new(
        i2c: I2C,
        addr: Mpr121Address,
        use_auto_config: bool,
        check_reset_flags: bool,
    ) -> Result<Self, Mpr121Error> {
        let mut dev = Mpr121 { i2c, addr };
        //reset
        let error = dev.write_register(SOFTRESET, 0x63).await.err();
        error.map(|e| match e {
            Mpr121Error::ReadError(reg) => Mpr121Error::ResetFailed {
                was_read: true,
                reg,
            },
            Mpr121Error::WriteError(reg) => Mpr121Error::ResetFailed {
                was_read: false,
                reg,
            },
            _ => Mpr121Error::ResetFailed {
                was_read: false,
                reg: 0xff,
            },
        });

        // Stop
        dev.write_register(ECR, 0x0).await?;

        if check_reset_flags {
            // read config register
            let config = dev.read_reg8(CONFIG2).await?;

            // Check if it is 0x24, which is the default configuration.
            // Otherwise bail.
            if config != 0x24 {
                return Err(Mpr121Error::InitFailed {
                    over_current_protection: dev.is_over_current_set().await?,
                });
            }
        }
        //Initialise the device to the similar settings as Adafruit
        dev.set_thresholds(
            crate::DEFAULT_TOUCH_THRESHOLD,
            crate::DEFAULT_RELEASE_THRESOLD,
        )
        .await?;
        dev.initialise_registers(use_auto_config).await?;

        Ok(dev)
    }

    #[maybe_async::maybe_async]
    async fn initialise_registers(&mut self, use_auto_config: bool) -> Result<(), Mpr121Error> {
        //Setup Filters MHD==MaximumHalfDelta, NHD=NoiseHalfDelta
        // Have a look at 5.5 in the data sheet for more information.

        self.write_register(MHDR, 0x01).await?;
        self.write_register(NHDR, 0x01).await?;
        self.write_register(NCLR, 0x0e).await?;
        self.write_register(FDLR, 0x00).await?;

        self.write_register(MHDF, 0x01).await?;
        self.write_register(NHDF, 0x05).await?;
        self.write_register(NCLF, 0x01).await?;
        self.write_register(FDLF, 0x00).await?;

        self.write_register(NHDT, 0x00).await?;
        self.write_register(NCLT, 0x00).await?;
        self.write_register(FDLT, 0x00).await?;

        self.write_register(DEBOUNCE, 0x0).await?;
        self.write_register(CONFIG1, 0x10).await?;
        self.write_register(CONFIG2, 0x20).await?;

        if use_auto_config {
            self.write_register(AUTOCONFIG0, 0x0b).await?;

            //Use 3.3V VDD
            self.write_register(UPLIMIT, 200).await?; // = ((Vdd - 0.7)/Vdd) * 256;
            self.write_register(TARGETLIMIT, 180).await?; // = UPLIMIT * 0.9
            self.write_register(LOWLIMIT, 130).await?; // = UPLIMIT * 0.65
        }

        //enable electrodes and return to start mode
        let ecr_setting = 0b10000000 + NUM_TOUCH_CHANNELS; // enable all 12 electrodes
        self.write_register(ECR, ecr_setting).await?;
        Ok(())
    }

    ///Initializes the driver assuming the sensors address is the default one (0x5a).
    /// If this fails, consider searching for the driver.
    /// Or following the documentation on setting a driver address, and use [new](Self::new) to specify the address.
    ///
    /// Have a look at [new](Self::new) for further documentation.
    #[maybe_async::maybe_async]
    pub async fn new_default(i2c: I2C) -> Result<Self, Mpr121Error> {
        let result = Self::new(i2c, Mpr121Address::Default, false, true).await?;

        Ok(result)
    }

    /// Returns true if over-current is detected by the device.
    /// In that case you probably have to check your circuit
    #[maybe_async::maybe_async]
    pub async fn is_over_current_set(&mut self) -> Result<bool, Mpr121Error> {
        const OVER_CURRENT_PROTECTION_FLAG_MASK: u8 = 0b1000_0000;
        let read = self.read_reg8(TOUCHSTATUS_H).await?;
        //If bit D7 is set, we have OVCF
        Ok((read & (OVER_CURRENT_PROTECTION_FLAG_MASK)) > 0)
    }

    /// Set the touch and release threshold for all channels. Usually the touch threshold is a little bigger than the release
    /// threshold. This creates some debounce characteristics. The correct thresholds depend on the application.
    ///
    /// Have a look at [note AN3892](https://www.nxp.com/docs/en/application-note/AN3892.pdf) of the mpr121 guidelines.
    #[maybe_async::maybe_async]
    pub async fn set_thresholds(&mut self, touch: u8, release: u8) -> Result<(), Mpr121Error> {
        for i in 0..NUM_TOUCH_CHANNELS {
            //Note ignoring false set thresholds
            self.write_register(TOUCHTH_0 + 2 * i, touch).await?;
            self.write_register(RELEASETH_0 + 2 * i, release).await?;
        }
        Ok(())
    }

    /// Sets the count for both touch and release. See 5.7 of the data sheet.
    ///
    /// value must be 0..8, is clamped if it exceeds.
    #[maybe_async::maybe_async]
    pub async fn set_debounce(&mut self, debounce_count: u8) -> Result<(), Mpr121Error> {
        let debounce = debounce_count.min(7);
        let bits = (debounce << 4) | (debounce);
        self.write_register(DEBOUNCE, bits).await?;

        Ok(())
    }

    /// Reads the filtered data from touch channels. Noise gets filtered out by the
    /// chip. See 5.3 in the data sheet.
    ///
    /// Note that the resulting value is only 10bit wide.
    ///
    /// Note that an error is returned, if `channel > 11`.
    #[maybe_async::maybe_async]
    pub async fn get_filtered(&mut self, channel: u8) -> Result<u16, Mpr121Error> {
        if channel > NUM_TOUCH_CHANNELS - 1 {
            return Err(Mpr121Error::ChannelExceed);
        }
        let result = self.read_reg16(FILTDATA_0L + channel * 2).await?;
        Ok(result)
    }

    /// Reads the baseline data for the channel. Note that this has only a resolution of 8bit.
    ///
    /// Note that an error is returned if `channel > 11`, or reading failed
    #[maybe_async::maybe_async]
    pub async fn get_baseline(&mut self, channel: u8) -> Result<u8, Mpr121Error> {
        if channel > NUM_TOUCH_CHANNELS - 1 {
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
        let mut value = self.read_reg16(BASELINE_0 + channel).await?;
        value &= 0b00000011_11111100;
        let cast = (value << 2).try_into().unwrap_or(0);
        Ok(cast)
    }

    /// Reads the *touched* state of all channels. Returns a u16 where each bit 0..12 indicates whether the
    /// pin is touched or not. Use bit shifting / masking to generate a mask, or, if only one sensor's value is
    /// needed, use [get_touch_state](Self::get_sensor_touch).
    ///
    /// Returns an error if reading failed.
    #[maybe_async::maybe_async]
    pub async fn get_touched(&mut self) -> Result<u16, Mpr121Error> {
        //mask upper four bits returns the rest
        let unmasked = self.read_reg16(TOUCHSTATUS_L).await?;
        Ok(unmasked & 0x0fff)
    }

    ///Returns the touch state of the given sensor.
    ///
    /// Returns false if `channel>11`, or reading failed.
    #[maybe_async::maybe_async]
    pub async fn get_sensor_touch(&mut self, channel: u8) -> bool {
        if channel > NUM_TOUCH_CHANNELS - 1 {
            return false;
        }

        //Masks all bits except for our channel, then returns true if the bit is set
        let result = self.get_touched().await;
        result.unwrap_or(0) & (1 << channel) > 0
    }
}

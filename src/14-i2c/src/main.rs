#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_imports)]
use aux14::{entry, iprint, iprintln, prelude::*};
use aux14::i2c1::RegisterBlock;

// Slave address
const MAGNETOMETER: u16 = 0b0011_1100;

// Addresses of the magnetometer's registers
const OUT_X_H_M: u8 = 0x03;
const IRA_REG_M: u8 = 0x0A;
const WHO_AM_I_M: u8 = 0x4F;

const CFG_REG_A_M: u8 = 0x60;
const OUTX_L_REG_M: u8 = 0x068;

const OUT_X_ACC: u8 = 0x28;


fn set_mode_continuous__LSM303AGR(i2c1: &RegisterBlock) -> (u8) {
    {
        // Broadcast START
        // Broadcast the MAGNETOMETER address with the R/W bit set to Write
        i2c1.cr2.write(|w| {
            w.start().set_bit();
            w.sadd().bits(MAGNETOMETER);
            w.rd_wrn().clear_bit();
            w.nbytes().bits(2);
            w.autoend().clear_bit()
        });

        // Wait until we can send more data
        while i2c1.isr.read().txis().bit_is_clear() {}

        // Send the address of the register that we want to read: WHO_AM_I_M
        i2c1.txdr.write(|w| w.txdata().bits(CFG_REG_A_M));
        i2c1.txdr.write(|w| w.txdata().bits(0x0));

        // Wait until the previous byte has been transmitted
        while i2c1.isr.read().tc().bit_is_clear() {}
    }

    let cfg_reg_a_m_byte = {
        // Broadcast RESTART
        // Broadcast the MAGNETOMETER address with the R/W bit set to Read.
        i2c1.cr2.modify(|_, w| {
            w.start().set_bit();
            w.nbytes().bits(1);
            w.rd_wrn().set_bit();
            w.autoend().set_bit()
        });

        // Wait until we have received the contents of the register
        while i2c1.isr.read().rxne().bit_is_clear() {}

        // Broadcast STOP (automatic because of `AUTOEND = 1`)

        i2c1.rxdr.read().rxdata().bits()
    };

    cfg_reg_a_m_byte
}

fn who_am_i__LSM303AGR(i2c1: &RegisterBlock) -> u8 {

    // Stage 1: Send the address of the register we want to read to the
    // magnetometer
    {
        // Broadcast START
        // Broadcast the MAGNETOMETER address with the R/W bit set to Write
        i2c1.cr2.write(|w| {
            w.start().set_bit();
            w.sadd().bits(MAGNETOMETER);
            w.rd_wrn().clear_bit();
            w.nbytes().bits(1);
            w.autoend().clear_bit()
        });

        // Wait until we can send more data
        while i2c1.isr.read().txis().bit_is_clear() {}

        // Send the address of the register that we want to read: WHO_AM_I_M
        i2c1.txdr.write(|w| w.txdata().bits(WHO_AM_I_M));

        // Wait until the previous byte has been transmitted
        while i2c1.isr.read().tc().bit_is_clear() {}
    }

    // Stage 2: Receive the contents of the register we asked for
    let byte = {
        // Broadcast RESTART
        // Broadcast the MAGNETOMETER address with the R/W bit set to Read.
        i2c1.cr2.modify(|_, w| {
            w.start().set_bit();
            w.nbytes().bits(1);
            w.rd_wrn().set_bit();
            w.autoend().set_bit()
        });

        // Wait until we have received the contents of the register
        while i2c1.isr.read().rxne().bit_is_clear() {}

        // Broadcast STOP (automatic because of `AUTOEND = 1`)

        i2c1.rxdr.read().rxdata().bits()
    };

    byte
}

#[entry]
fn main() -> ! {
    let (i2c1, mut delay, mut itm) = aux14::init();

    let cfg_reg_a_m_byte: u8 = set_mode_continuous__LSM303AGR(i2c1);
    // Expected output:  0x60 - 0b00000000
    iprintln!(&mut itm.stim[0], "0x{:02X} - 0b{:08b}", CFG_REG_A_M, cfg_reg_a_m_byte);

    let whoami: u8 = who_am_i__LSM303AGR(i2c1);
    // Expected output:  0x4F - 0b01000000
    iprintln!(&mut itm.stim[0], "0x{:02X} - 0b{:08b}", WHO_AM_I_M, whoami);

    // sample magnetometer
    loop{
        // ask for an array of 6 register values starting at OUTX_L_REG_M (0x68)
        {
            // Broadcast START
            // Broadcast the MAGNETOMETER address with the R/W bit set to Write
            i2c1.cr2.write(|w| {
                w.start().set_bit();
                w.sadd().bits(MAGNETOMETER);
                w.rd_wrn().clear_bit();
                w.nbytes().bits(1);
                w.autoend().clear_bit()
            });

            // Wait until we can send more data
            while i2c1.isr.read().txis().bit_is_clear() {}

            // Send the address of the register that we want to read: WHO_AM_I_M
            i2c1.txdr.write(|w| w.txdata().bits(OUTX_L_REG_M));

            // Wait until the previous byte has been transmitted
            while i2c1.isr.read().tc().bit_is_clear() {}
        }

        i2c1.cr2.modify(|_, w| {
            w.start().set_bit();
            w.nbytes().bits(6);
            w.rd_wrn().set_bit();
            w.autoend().set_bit()
        });

        let mut buffer = [0u8; 6];
        for byte in &mut buffer {
            // Wait until we have received something
            while i2c1.isr.read().rxne().bit_is_clear() {}

            *byte = i2c1.rxdr.read().rxdata().bits();
        }
        // Broadcast STOP (automatic because of `AUTOEND = 1`)

        let x_l = u16::from(buffer[0]);
        let x_h = u16::from(buffer[1]);
        let z_l = u16::from(buffer[2]);
        let z_h = u16::from(buffer[3]);
        let y_l = u16::from(buffer[4]);
        let y_h = u16::from(buffer[5]);

        let x = ((x_h << 8) + x_l) as i16;
        let y = ((y_h << 8) + y_l) as i16;
        let z = ((z_h << 8) + z_l) as i16;

        iprintln!(&mut itm.stim[0], "{:?}", (x, y, z));

        delay.delay_ms(1_000_u16);
    }
}

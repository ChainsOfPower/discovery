#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;

use microbit::{
    hal::prelude::*,
    hal::twim,
    hal::{uarte::{self, Parity, Baudrate}, Twim},
    pac::{twim0::frequency::FREQUENCY_A, UARTE0, TWIM0},
};

mod serial_setup;
use serial_setup::UartePort;

use lsm303agr::{
    AccelOutputDataRate, Lsm303agr,
};

enum Command {
    Magnetometer,
    Accelerometer,
    Error
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    
    let mut sensor: Lsm303agr<lsm303agr::interface::I2cInterface<Twim<TWIM0>>, lsm303agr::mode::MagOneShot> = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(lsm303agr::MagOutputDataRate::Hz50).unwrap();
    let mut sensor: Lsm303agr<lsm303agr::interface::I2cInterface<Twim<TWIM0>>, lsm303agr::mode::MagContinuous> = sensor.into_mag_continuous().ok().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    loop {
        let command = parse_command(&mut serial);
        execute_command(&mut serial, command, &mut sensor);
    }
}

fn parse_command(serial: &mut UartePort<UARTE0>) -> Command {
    write!(serial, "Parsing command\r\n").unwrap();
    let mut buffer: Vec<u8, 32> = Vec::new();
    
    loop {
        let byte = nb::block!(serial.read()).unwrap();

        if buffer.push(byte).is_err() {
            write!(serial, "error: buffer full\r\n").unwrap();
            return Command::Error;
        }

        if byte == 13 {
            let command = core::str::from_utf8(&buffer).unwrap();

            write!(serial, "{}\r\n", command);
            
            return match command {
                "magnetometer" => Command::Magnetometer,
                "accelerometer" => Command::Accelerometer,
                _ => {
                    write!(serial, "error: unknown command\r\n").unwrap();
                    Command::Error
                }
            };
        }
    }
}

fn execute_command(serial: &mut UartePort<UARTE0>, command: Command, sensor: &mut Lsm303agr<lsm303agr::interface::I2cInterface<Twim<TWIM0>>, lsm303agr::mode::MagContinuous>) {
    
    match command {
        Command::Accelerometer => {
            if sensor.accel_status().unwrap().xyz_new_data {
                let data = sensor.accel_data().unwrap();
                write!(serial, "Acceleration: x {} y {} z {}", data.x, data.y, data.z);
            }
        },
        Command::Magnetometer => {
            if sensor.mag_status().unwrap().xyz_new_data {
                let data = sensor.mag_data().unwrap();
                write!(serial, "Magnetic field: x {} y {} z {}", data.x, data.y, data.z);
            }
        },
        Command::Error => {
            write!(serial, "Unknown command");
        },
    }
}

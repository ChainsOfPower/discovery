#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{prelude::*, Timer},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut matrix = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    loop {
        for i in 1..5 {
            matrix[0][i - 1] = 0;
            matrix[0][i] = 1;
            display.show(&mut timer, matrix, 50);
        }

        for i in 1..5 {
            matrix[i - 1][4] = 0;
            matrix[i][4] = 1;
            display.show(&mut timer, matrix, 50);
        }

        for i in (1..5).rev() {
            matrix[4][i] = 0;
            matrix[4][i - 1] = 1;
            display.show(&mut timer, matrix, 50);
        }

        for i in (1..5).rev() {
            matrix[i][0] = 0;
            matrix[i - 1][0] = 1;
            display.show(&mut timer, matrix, 50);
        }
    }
}

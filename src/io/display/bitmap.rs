use microbit_bsp::display::{fonts::frame_5x5, Frame};

#[rustfmt::skip]
/// A heart bitmap.
pub const HEART: Frame<5, 5> = frame_5x5(&[
    0b01010,
    0b10101,
    0b10001,
    0b01010,
    0b00100,
]);

#[rustfmt::skip]
/// A smile bitmap.
pub const SMILE: Frame<5, 5> = frame_5x5(&[
    0b00000,
    0b01010,
    0b00000,
    0b10001,
    0b01110,
]);

#[rustfmt::skip]
/// A sad bitmap.
pub const SAD: Frame<5, 5> = frame_5x5(&[
    0b00000,
    0b01010,
    0b00000,
    0b01110,
    0b10001,
]);

#[rustfmt::skip]
/// An up arrow bitmap.
pub const ARROW_UP: Frame<5, 5> = frame_5x5(&[
    0b00100,
    0b01110,
    0b10101,
    0b00100,
    0b00100,
]);

#[rustfmt::skip]
/// A down arrow bitmap.
pub const ARROW_DOWN: Frame<5, 5> = frame_5x5(&[
    0b00100,
    0b00100,
    0b10101,
    0b01110,
    0b00100,
]);

#[rustfmt::skip]
/// A question mark bitmap.
pub const QUESTION_MARK: Frame<5, 5> = frame_5x5(&[
    0b01100,
    0b10010,
    0b00100,
    0b00000,
    0b00100,
]);

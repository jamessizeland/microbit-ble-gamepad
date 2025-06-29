use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::{Duration, Timer};
use microbit_bsp::{
    display::{fonts::*, Brightness, Frame},
    LedMatrix,
};

use self::bitmap::{ARROW_DOWN, ARROW_UP};
pub mod bitmap;

pub static DISPLAY_CHANNEL: Channel<ThreadModeRawMutex, DisplayAction, 64> = Channel::new();

type DisplayQueue = Sender<'static, ThreadModeRawMutex, DisplayAction, 64>;

#[derive(Clone, Copy)]
pub struct AsyncDisplay {
    sender: DisplayQueue,
}

impl AsyncDisplay {
    pub fn new(spawner: Spawner, display: LedMatrix) -> Self {
        // Spawn the display driver task
        defmt::unwrap!(spawner.spawn(display_driver_task(display)));
        Self {
            sender: DISPLAY_CHANNEL.sender(),
        }
    }

    pub async fn set_brightness(&self, brightness: Brightness) {
        self.sender
            .send(DisplayAction::SetBrightness(brightness))
            .await;
    }

    #[allow(unused)]
    pub async fn clear(&self) {
        self.sender.send(DisplayAction::Clear).await;
    }

    pub async fn scroll(&self, text: &'static str) {
        self.sender.send(DisplayAction::Scroll(text)).await;
    }

    /// Non-blocking display
    pub async fn display(&self, frame: DisplayFrame, duration: Duration) {
        self.sender
            .send(DisplayAction::SetFrame {
                frame,
                duration: Some(duration),
            })
            .await;
    }
    /// Blocking display
    pub async fn display_blocking(&self, frame: DisplayFrame, duration: Duration) {
        self.sender
            .send(DisplayAction::SetFrame {
                frame,
                duration: Some(duration),
            })
            .await;
        Timer::after(duration).await;
    }
}

#[allow(unused)]
pub enum DisplayFrame {
    Custom(Frame<5, 5>),
    /// Display a single pixel at the given coordinates, where (0,0) is the center of the display.
    Coord {
        x: i8,
        y: i8,
    },
    Letter(char),
    Heart,
    Smile,
    Sad,
    QuestionMark,
    Left,
    Right,
    Up,
    Down,
}

impl DisplayFrame {
    fn to_frame(&self) -> Frame<5, 5> {
        match self {
            DisplayFrame::Custom(frame) => *frame,
            DisplayFrame::Heart => bitmap::HEART,
            DisplayFrame::Smile => bitmap::SMILE,
            DisplayFrame::Sad => bitmap::SAD,
            DisplayFrame::QuestionMark => bitmap::QUESTION_MARK,
            DisplayFrame::Left => ARROW_LEFT,
            DisplayFrame::Right => ARROW_RIGHT,
            DisplayFrame::Up => ARROW_UP,
            DisplayFrame::Down => ARROW_DOWN,
            DisplayFrame::Coord { x, y } => {
                let mut frame = Frame::empty();
                // convert from cartesian coordinates to display coordinates
                let x = (x + 2).clamp(0, 4);
                let y = (y + 2).clamp(0, 4);
                frame.set(x as usize, y as usize);
                frame
            }
            DisplayFrame::Letter(c) => {
                // temporary
                match c {
                    'A' => bitmap::SMILE,
                    'B' => bitmap::SAD,
                    'C' => ARROW_LEFT,
                    'D' => ARROW_UP,
                    'E' => ARROW_DOWN,
                    'F' => ARROW_RIGHT,
                    _ => bitmap::QUESTION_MARK,
                }
            }
        }
    }
}

#[allow(unused)]
pub enum DisplayAction {
    /// Set the brightness of the display.
    SetBrightness(Brightness),
    /// Display a frame.
    SetFrame {
        frame: DisplayFrame,
        duration: Option<Duration>,
    },
    Clear,
    Scroll(&'static str),
}

/// A task to update the display asynchronously, will wait for new inputs to be sent to it from a queue.
#[embassy_executor::task]
async fn display_driver_task(mut display: LedMatrix) {
    info!("Display driver task started");
    loop {
        match DISPLAY_CHANNEL.receive().await {
            DisplayAction::SetBrightness(brightness) => {
                display.set_brightness(brightness);
            }
            DisplayAction::Clear => {
                display.clear();
            }
            DisplayAction::Scroll(text) => {
                display.scroll(text).await;
            }
            DisplayAction::SetFrame { frame, duration } => match duration {
                Some(duration) => {
                    display.display(frame.to_frame(), duration).await;
                }
                None => {
                    unimplemented!("Displaying a frame without a duration is not yet implemented")
                }
            },
        }
    }
}

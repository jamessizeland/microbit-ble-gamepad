use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::Duration;
use microbit_bsp::{
    display::{fonts::*, Brightness, Frame},
    LedMatrix,
};

use self::bitmap::{ARROW_DOWN, ARROW_UP};
pub mod bitmap;

pub static DISPLAY_CHANNEL: Channel<ThreadModeRawMutex, DisplayAction, 64> = Channel::new();

pub struct AsyncDisplay {
    sender: Sender<'static, ThreadModeRawMutex, DisplayAction, 64>,
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

    pub async fn clear(&self) {
        self.sender.send(DisplayAction::Clear).await;
    }

    pub async fn scroll(&self, text: &'static str) {
        self.sender.send(DisplayAction::Scroll(text)).await;
    }

    pub async fn display(&self, frame: DisplayFrame, duration: Duration) {
        self.sender
            .send(DisplayAction::SetFrame {
                frame,
                duration: Some(duration),
            })
            .await;
    }
}

pub enum DisplayFrame {
    DisplayFrame(Frame<5, 5>),
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
            DisplayFrame::DisplayFrame(frame) => frame.clone(),
            DisplayFrame::Heart => bitmap::HEART,
            DisplayFrame::Smile => bitmap::SMILE,
            DisplayFrame::Sad => bitmap::SAD,
            DisplayFrame::QuestionMark => bitmap::QUESTION_MARK,
            DisplayFrame::Left => ARROW_LEFT,
            DisplayFrame::Right => ARROW_RIGHT,
            DisplayFrame::Up => ARROW_UP,
            DisplayFrame::Down => ARROW_DOWN,
        }
    }
}

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

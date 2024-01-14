use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{
    peripherals::{P0_00, PWM0},
    pwm::SimplePwm,
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use microbit_bsp::speaker::{self, Note};

pub static AUDIO_CHANNEL: Channel<ThreadModeRawMutex, AudioAction, 64> = Channel::new();

pub enum AudioAction {
    PlayNote(Note),
    PlayTune,
}

pub struct AsyncAudio {
    sender: Sender<'static, ThreadModeRawMutex, AudioAction, 64>,
}

impl AsyncAudio {
    /// Create a new instance of the audio driver
    pub fn new(spawner: Spawner, pwm0: PWM0, speaker: P0_00) -> Self {
        // Spawn the audio driver task
        defmt::unwrap!(spawner.spawn(audio_driver_task(pwm0, speaker)));
        Self {
            sender: AUDIO_CHANNEL.sender(),
        }
    }
    /// Play a note on the speaker
    pub async fn play_note(&self, note: Note) {
        self.sender.send(AudioAction::PlayNote(note)).await;
    }
}

/// The audio driver task
#[embassy_executor::task]
async fn audio_driver_task(pwm0: PWM0, speaker: P0_00) {
    info!("Audio driver task started");
    let pwm = SimplePwm::new_1ch(pwm0, speaker);
    let mut speaker = speaker::PwmSpeaker::new(pwm);
    loop {
        match AUDIO_CHANNEL.receive().await {
            AudioAction::PlayNote(note) => {
                speaker.play(&note).await;
            }
            AudioAction::PlayTune => unimplemented!("play tune"),
        }
    }
}

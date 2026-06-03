use crate::internal::{AudioParameter, PLAYING_SOUND_CONTROLLERS};
use tokio::sync::oneshot;

/// A controller for a sound actively playing.
///
/// Make a sound controllable by calling [`SoundBuilder.controller()`][super::SoundBuilder::controller].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controller(u64);

impl Controller {
    // internal
    pub(crate) fn new(id: u64) -> Self {
        // inserted into PLAYING_SOUND_CONTROLLERS in audio::play
        Self(id)
    }

    /// Return `true` if the sound has finished playing or has been stopped.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        #[expect(clippy::missing_panics_doc)]
        let map = PLAYING_SOUND_CONTROLLERS.get().unwrap().read().unwrap();
        map.get(&self.0).is_none()
    }

    /// Wait until the sound has finished playing or has been stopped.
    pub async fn wait_until_finished(self) {
        if self.is_finished() {
            return;
        }
        // Single threaded reliance: Its fine to check if finished and then insert the notifier since
        // we only receive notifications when we poll and our WASM executor is single threaded.
        let receiver = {
            #[expect(clippy::missing_panics_doc)]
            let mut map = PLAYING_SOUND_CONTROLLERS.get().unwrap().write().unwrap();
            let (sender, receiver) = oneshot::channel();
            map.entry(self.0).or_default().push(sender);
            receiver
        };
        let _ = receiver.await;
    }

    /// Pause or unpause the sound.
    ///
    /// `paused` is `true` if the sound should be paused, `false` if it should be unpaused.
    pub fn set_paused(&self, paused: bool) {
        let value = if paused { 1. } else { 0. };
        self.set_controller_parameter(AudioParameter::Pause, value)
    }

    /// Set the volume of the sound.
    ///
    /// The samples are multiplied by `multiplier` so 1.0 would leave the Sound
    /// unchanged. 0.5 would reduce the sample values by half and 2.0 would
    /// double them (saturating if larger than the max value).
    pub fn set_volume(&self, multiplier: f32) {
        self.set_controller_parameter(AudioParameter::Volume, multiplier)
    }

    /// Set the playback speed of the sound.
    ///
    /// `multiplier` is a linear scale factor: `1.0` = original speed, `2.0` = double speed.
    /// Pitch is adjusted proportionally to speed.
    pub fn set_speed(&self, multiplier: f32) {
        self.set_controller_parameter(AudioParameter::Speed, multiplier)
    }

    /// Stop the sound immediately.
    ///
    /// A stopped sound can not be restarted. Waiting controllers will receive
    /// their finished notification.
    pub fn stop(self) {
        self.set_controller_parameter(AudioParameter::Stop, 1.0)
    }

    fn set_controller_parameter(&self, param: AudioParameter, value: f32) {
        (crate::internal::AUDIO_CONTROLLER_MODIFY_PARAMS_FN
            .get()
            .unwrap())(self.0, param, value);
    }
}

//! Audio playback utilities.
mod controller;
mod sound_builder;
mod sound_instruction;

use std::{borrow::Cow, path::PathBuf, str::FromStr};

pub use controller::Controller;
pub use sound_builder::{ControllerOpts, SoundBuilder};
pub use sound_instruction::{ControllerParams, SoundInstruction};

use crate::language::{LanguageTag, system_language};

/// Configuration parameters for `number_speaker`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct NumberSpeakerConfig {
    /// The path prefix where files will be searched. Defaults to `numbers`.
    /// NOTE: It's important that the files you use are formatted the same as the files in `numbers/en-US`.
    pub prefix: PathBuf,
    /// The file extension of the expected audio files. Defaults to `qoa`.
    pub extension: Cow<'static, str>,
    /// The language to use for conversion. Defaults to the active device language.
    pub language: LanguageTag,
}

impl Default for NumberSpeakerConfig {
    fn default() -> Self {
        Self {
            prefix: PathBuf::from_str("numbers").unwrap(),
            extension: Cow::Borrowed("qoa"),
            language: system_language(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::audio::NumberSpeakerConfig;

    #[test]
    fn config_deserialize() {
        assert_eq!(
            NumberSpeakerConfig::default(),
            serde_json::from_str(
                r#"{"prefix": "numbers", "extension": "qoa", "language": "en-US"}"#
            )
            .expect("should be able to deserialize")
        );
    }
}

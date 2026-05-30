use std::time::Duration;

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::from_value;

/// Instructions on how to create a sound for playback.
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum SoundInstruction {
    /// Path to an audio file to play.
    ///
    /// If a relative path is provided, its path is relative to a directory
    /// dependent on the context (e.g. in boppo_wasm it is relative to the
    /// directory the wasm file is located in, and in boppo_websocket it is
    /// relative to /sd/activities/user/).
    ///
    /// If an absolute path is provided, it is relative to the activities
    /// directory (i.e. /sd/activities/).
    ///
    /// JSON representation: `"<File Path>"`
    PlayFile(String),
    /// Play sounds one after the other.
    ///
    /// JSON representation: `[<SoundInstruction>, ...]`
    List(Vec<SoundInstruction>),
    /// Play sounds at the same time.
    ///
    /// JSON representation: `{"i": "simultaneous", "sounds": [<SoundInstruction>, ...]}`
    Simultaneous(Vec<SoundInstruction>),
    /// Repeat a SoundInstruction
    ///
    /// `sound` must not contain a Controller either directly or indirectly.
    ///
    /// JSON representation: `{"i": "repeat", "sound": <SoundInstruction>, "times": 5}`
    ///
    /// - `times` is optional; if omitted the sound is repeated indefinitely.
    Repeat(Box<SoundInstruction>, Option<u64>),
    /// Make the contained sound controllable
    ///
    /// You can control the speed, volume, and paused state of the sound as well as stop the sound completely.
    /// You can also get notifications when the sound has finished playing.
    ///
    /// A Controller is not allowed to be nested within a Repeat.
    ///
    /// JSON representation: `{"i": "controller", "sound": <SoundInstruction>, "id": 1, "speed": 1.0, "volume": 1.0, "paused": false}`
    ///
    /// - `id`: the unique identifier for the controller. Must be greater than or equal to 0 and less than 2^53.
    /// - `speed`: the initial speed of the sound (optional, defaults to 1.0)
    /// - `volume`: the initial volume of the sound (optional, defaults to 1.0)
    /// - `paused`: whether the sound should start paused (optional, defaults to false)
    Controller(Box<SoundInstruction>, ControllerParams),
    /// Play a silence for the specified duration.
    ///
    /// Useful for pausing or delaying the playback of a sound (e.g. in the
    /// middle of a `SoundInstruction::List`)
    ///
    /// JSON Representation: `{"i": "silence", "millis": 1500}`
    Silence(Duration),
    /// Speak a number aloud using the stitched together sound files.
    ///
    /// The number is spoken in the system language.
    ///
    /// JSON representation: `{"i": "speak_number", "number": 42}`
    SpeakNumber(i64),
    /// Timed Commands allow executing commands at specific times during the playback of a sound.
    ///
    /// JSON representation: `{"i": "timed_commands", "commands_path": <PATH>, "sound": <SoundInstruction>, "commands": ["MILLIS ..."]}`
    ///
    /// `commands_path` and `commands` are both optional. If both are present the commands are merged.
    TimedCommands(Option<String>, Box<SoundInstruction>, Vec<String>),
    /// An error beep that plays briefly.
    ///
    /// JSON representation: `{"i": "error_sound"}`
    ErrorSound,
    /// A sound that immediately returns.
    ///
    /// JSON representation: `{"i": "empty_sound"}`
    #[default]
    EmptySound,
}

/// Parameters for a [`SoundInstruction::Controller`] variant.
#[derive(Debug, Clone, PartialEq)]
pub struct ControllerParams {
    /// Unique identifier for this controller.
    pub id: u64,
    /// Initial playback speed multiplier (defaults to `1.0`).
    pub speed: Option<f32>,
    /// Initial volume multiplier (defaults to `1.0`).
    pub volume: Option<f32>,
    /// Whether the sound should start paused (defaults to `false`).
    pub paused: Option<bool>,
}

impl<'de> serde::Deserialize<'de> for SoundInstruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        match value {
            Value::String(s) => Ok(Self::PlayFile(s)),
            Value::Array(arr) => {
                let instrs: Result<Vec<SoundInstruction>, _> = arr
                    .into_iter()
                    .map(from_value::<SoundInstruction>)
                    .collect();
                let instrs = instrs.map_err(D::Error::custom)?;
                Ok(SoundInstruction::List(instrs))
            }
            Value::Object(mut map) => {
                let instr = map
                    .get("i")
                    .and_then(Value::as_str)
                    .ok_or(D::Error::custom("object missing instruction member"))?;
                Ok(match instr {
                    "simultaneous" => {
                        let sounds_val = map
                            .remove("sounds")
                            .ok_or(D::Error::custom("missing sounds array"))?;

                        let Value::Array(arr) = sounds_val else {
                            return Err(D::Error::custom("sounds should be an array"));
                        };

                        let instrs: Result<Vec<SoundInstruction>, _> = arr
                            .into_iter()
                            .map(from_value::<SoundInstruction>)
                            .collect();
                        let instrs = instrs.map_err(D::Error::custom)?;
                        SoundInstruction::Simultaneous(instrs)
                    }
                    "repeat" => {
                        let times = map
                            .get("times")
                            .and_then(Value::as_number)
                            .and_then(serde_json::Number::as_u64);

                        let sound_json = map
                            .remove("sound")
                            .ok_or(D::Error::custom("repeat requires sound field"))?;
                        let sound: SoundInstruction =
                            serde_json::from_value(sound_json).map_err(D::Error::custom)?;

                        SoundInstruction::Repeat(Box::new(sound), times)
                    }
                    "silence" => {
                        let millis = map
                            .get("millis")
                            .and_then(Value::as_number)
                            .and_then(serde_json::Number::as_u64)
                            .unwrap_or(1_000);
                        SoundInstruction::Silence(Duration::from_millis(millis))
                    }
                    "speak_number" => {
                        let number = map
                            .get("number")
                            .and_then(Value::as_number)
                            .and_then(serde_json::Number::as_i64)
                            .ok_or(D::Error::custom("speak_number requires number field"))?;
                        SoundInstruction::SpeakNumber(number)
                    }

                    "timed_commands" => {
                        let commands_path = map
                            .get("commands_path")
                            .and_then(Value::as_str)
                            .map(String::from)
                            .to_owned();
                        let sound_json = map
                            .remove("sound")
                            .ok_or(D::Error::custom("timed_commands requires sound field"))?;
                        let commands = if let Some(commands) = map.remove("commands") {
                            let command_strs = commands
                                .as_array()
                                .ok_or(D::Error::custom("commands should be an array"))?;
                            let mut commands: Vec<String> = vec![];
                            for command_str in command_strs {
                                let command_str = command_str.as_str().ok_or(D::Error::custom(
                                    "commands should be an array of strings",
                                ))?;
                                commands.push(command_str.to_owned());
                            }
                            commands
                        } else {
                            vec![]
                        };

                        let sound: SoundInstruction =
                            serde_json::from_value(sound_json).map_err(D::Error::custom)?;
                        SoundInstruction::TimedCommands(commands_path, Box::new(sound), commands)
                    }
                    "controller" => {
                        let id = map
                            .get("id")
                            .and_then(Value::as_u64)
                            .ok_or(D::Error::custom("controller requires id field"))?;
                        if id > 2 ^ 53 {
                            return Err(D::Error::custom("controller id must be less than 2^53"));
                        }
                        let speed = map.get("speed").and_then(Value::as_f64).map(|v| v as f32);
                        let volume = map.get("volume").and_then(Value::as_f64).map(|v| v as f32);
                        let paused = map.get("paused").and_then(Value::as_bool);
                        let sound_json = map
                            .remove("sound")
                            .ok_or(D::Error::custom("controller requires sound field"))?;
                        let sound: SoundInstruction = serde_json::from_value(sound_json)
                            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
                        SoundInstruction::Controller(
                            Box::new(sound),
                            ControllerParams {
                                id,
                                speed,
                                volume,
                                paused,
                            },
                        )
                    }
                    "error_sound" => SoundInstruction::ErrorSound,
                    "empty_sound" => SoundInstruction::EmptySound,
                    unknown => {
                        return Err(D::Error::custom(format!(
                            "unknown instruction: {}",
                            unknown
                        )));
                    }
                })
            }
            _ => Err(D::Error::custom("unexpected JSON type")),
        }
    }
}

impl Serialize for SoundInstruction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SoundInstruction::PlayFile(path) => path.serialize(serializer),
            SoundInstruction::List(sounds) => sounds.serialize(serializer),
            SoundInstruction::Simultaneous(sounds) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("i", "simultaneous")?;
                map.serialize_entry("sounds", sounds)?;
                map.end()
            }
            SoundInstruction::Repeat(sound, times) => {
                let len = if times.is_some() { 3 } else { 2 };
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("i", "repeat")?;
                map.serialize_entry("sound", sound)?;
                if let Some(t) = times {
                    map.serialize_entry("times", t)?;
                }
                map.end()
            }
            SoundInstruction::Silence(duration) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("i", "silence")?;
                let millis: u64 = duration.as_millis().try_into().unwrap();
                map.serialize_entry("millis", &millis)?;
                map.end()
            }
            SoundInstruction::SpeakNumber(number) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("i", "speak_number")?;
                map.serialize_entry("number", number)?;
                map.end()
            }
            SoundInstruction::Controller(sound, params) => {
                let mut len = 3;
                if params.speed.is_some() {
                    len += 1;
                }
                if params.volume.is_some() {
                    len += 1;
                }
                if params.paused.is_some() {
                    len += 1;
                }
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("i", "controller")?;
                map.serialize_entry("id", &params.id)?;
                map.serialize_entry("sound", sound)?;
                if let Some(speed) = params.speed {
                    map.serialize_entry("speed", &speed)?;
                }
                if let Some(volume) = params.volume {
                    map.serialize_entry("volume", &volume)?;
                }
                if let Some(paused) = params.paused {
                    map.serialize_entry("paused", &paused)?;
                }
                map.end()
            }
            SoundInstruction::TimedCommands(commands_path, sound, commands) => {
                let len = 2 + commands_path.is_some() as usize + (!commands.is_empty()) as usize;
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("i", "timed_commands")?;
                map.serialize_entry("sound", sound)?;
                if let Some(path) = commands_path {
                    map.serialize_entry("commands_path", path)?;
                }
                if !commands.is_empty() {
                    map.serialize_entry("commands", commands)?;
                }
                map.end()
            }
            SoundInstruction::ErrorSound => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("i", "error_sound")?;
                map.end()
            }
            SoundInstruction::EmptySound => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("i", "empty_sound")?;
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(instr: &SoundInstruction) -> SoundInstruction {
        let json = serde_json::to_string(instr).expect("serialize failed");
        serde_json::from_str(&json).expect("deserialize failed")
    }

    #[test]
    fn play_file() {
        let instr = SoundInstruction::PlayFile("sounds/beep.mp3".into());
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn list() {
        let instr = SoundInstruction::List(vec![
            SoundInstruction::PlayFile("a.mp3".into()),
            SoundInstruction::PlayFile("b.mp3".into()),
        ]);
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn list_empty() {
        let instr = SoundInstruction::List(vec![]);
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn simultaneous() {
        let instr = SoundInstruction::Simultaneous(vec![
            SoundInstruction::PlayFile("a.mp3".into()),
            SoundInstruction::PlayFile("b.mp3".into()),
        ]);
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn repeat_with_times() {
        let instr = SoundInstruction::Repeat(
            Box::new(SoundInstruction::PlayFile("loop.mp3".into())),
            Some(5),
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn repeat_indefinite() {
        let instr = SoundInstruction::Repeat(
            Box::new(SoundInstruction::PlayFile("loop.mp3".into())),
            None,
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn controller_all_params() {
        let instr = SoundInstruction::Controller(
            Box::new(SoundInstruction::PlayFile("music.mp3".into())),
            ControllerParams {
                id: 42,
                speed: Some(1.5),
                volume: Some(0.8),
                paused: Some(true),
            },
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn controller_minimal() {
        let instr = SoundInstruction::Controller(
            Box::new(SoundInstruction::PlayFile("music.mp3".into())),
            ControllerParams {
                id: 1,
                speed: None,
                volume: None,
                paused: None,
            },
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn silence() {
        let instr = SoundInstruction::Silence(Duration::from_millis(1500));
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn speak_number() {
        let instr = SoundInstruction::SpeakNumber(-42);
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn error_sound() {
        let instr = SoundInstruction::ErrorSound;
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn empty_sound() {
        let instr = SoundInstruction::EmptySound;
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn timed_commands_minimal() {
        let instr = SoundInstruction::TimedCommands(
            None,
            Box::new(SoundInstruction::PlayFile("bg.mp3".into())),
            vec![],
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn timed_commands_with_path() {
        let instr = SoundInstruction::TimedCommands(
            Some("commands/level1.txt".into()),
            Box::new(SoundInstruction::PlayFile("bg.mp3".into())),
            vec![],
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn timed_commands_with_commands() {
        let instr = SoundInstruction::TimedCommands(
            None,
            Box::new(SoundInstruction::PlayFile("bg.mp3".into())),
            vec!["0 START".into(), "5000 STOP".into()],
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn timed_commands_full() {
        let instr = SoundInstruction::TimedCommands(
            Some("commands/level1.txt".into()),
            Box::new(SoundInstruction::PlayFile("bg.mp3".into())),
            vec!["0 START".into(), "5000 STOP".into()],
        );
        assert_eq!(round_trip(&instr), instr);
    }

    #[test]
    fn nested_controller_in_repeat() {
        let instr = SoundInstruction::Repeat(
            Box::new(SoundInstruction::Controller(
                Box::new(SoundInstruction::PlayFile("nested.mp3".into())),
                ControllerParams {
                    id: 7,
                    speed: Some(2.0),
                    volume: None,
                    paused: Some(false),
                },
            )),
            Some(3),
        );
        assert_eq!(round_trip(&instr), instr);
    }
}

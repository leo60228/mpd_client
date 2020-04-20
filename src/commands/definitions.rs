//! Definitions of commands.

use mpd_protocol::Command as RawCommand;

use std::borrow::Cow;
use std::cmp::min;
use std::time::Duration;

use super::{
    responses::{self as res, SingleMode, SongId},
    Command,
};

macro_rules! argless_command {
    // Utility branch to generate struct with doc expression
    (#[doc = $doc:expr],
     $item:item) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $item
    };
    ($name:ident, $command:literal, $response:ty) => {
        argless_command!(
            #[doc = concat!("`", $command, "` command")],
            pub struct $name;
        );

        impl Command for $name {
            type Response = $response;

            fn to_command(self) -> RawCommand {
                RawCommand::new($command)
            }
        }
    };
}

macro_rules! single_arg_command {
    // Utility branch to generate struct with doc expression
    (#[doc = $doc:expr],
     $item:item) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[allow(missing_copy_implementations)]
        $item
    };
    ($name:ident, $argtype:ty, $command:literal, $response:ty) => {
        single_arg_command!(
            #[doc = concat!("`", $command, "` command")],
            pub struct $name(pub $argtype);
        );

        impl Command for $name {
            type Response = $response;

            fn to_command(self) -> RawCommand {
                RawCommand::new($command)
                    .argument(self.0.render())
            }
        }
    };
}

macro_rules! impl_display_argument {
    ($($type:ty),+) => {
        $(
            impl Argument for $type {
                fn render(self) -> Cow<'static, str> {
                    Cow::Owned(self.to_string())
                }
            }
        )+
    };
}

trait Argument {
    fn render(self) -> Cow<'static, str>;
}

impl_display_argument!(u8);

impl Argument for bool {
    fn render(self) -> Cow<'static, str> {
        Cow::Borrowed(if self { "1" } else { "0" })
    }
}

argless_command!(Next, "next", res::Empty);
argless_command!(Previous, "previous", res::Empty);
argless_command!(Stop, "stop", res::Empty);
argless_command!(ClearQueue, "clear", res::Empty);

argless_command!(Status, "status", res::Status);
argless_command!(Stats, "stats", res::Stats);

argless_command!(Queue, "playlistinfo", Vec<res::Song>);
argless_command!(CurrentSong, "currentsong", Option<res::Song>);

single_arg_command!(SetRandom, bool, "random", res::Empty);
single_arg_command!(SetConsume, bool, "consume", res::Empty);
single_arg_command!(SetRepeat, bool, "repeat", res::Empty);
single_arg_command!(SetPause, bool, "pause", res::Empty);

/// `crossfade` command.
///
/// The given duration is truncated to the seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crossfade(pub Duration);

impl Command for Crossfade {
    type Response = res::Empty;

    fn to_command(self) -> RawCommand {
        let seconds = self.0.as_secs();
        RawCommand::new("crossfade").argument(seconds.to_string())
    }
}

/// `setvol` command.
///
/// Set the volume. The value is truncated to fit in the range `0..=100`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetVolume(pub u8);

impl Command for SetVolume {
    type Response = res::Empty;

    fn to_command(self) -> RawCommand {
        let volume = min(self.0, 100);
        RawCommand::new("setvol").argument(volume.to_string())
    }
}

/// `single` command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetSingle(pub SingleMode);

impl Command for SetSingle {
    type Response = res::Empty;

    fn to_command(self) -> RawCommand {
        let single = match self.0 {
            SingleMode::Disabled => "0",
            SingleMode::Enabled => "1",
            SingleMode::Oneshot => "oneshot",
        };

        RawCommand::new("single").argument(single)
    }
}

/// Modes to target a song with a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Song {
    /// By ID
    Id(SongId),
    /// By position in the queue.
    Position(usize),
}

/// `seek` and `seekid` commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeekTo(pub Song, pub Duration);

impl Command for SeekTo {
    type Response = res::Empty;

    fn to_command(self) -> RawCommand {
        let (command, song_arg) = match self.0 {
            Song::Id(id) => ("seekid", id.to_string()),
            Song::Position(pos) => ("seek", pos.to_string()),
        };

        RawCommand::new(command)
            .argument(song_arg)
            .argument(format!("{:.3}", self.1.as_secs_f64()))
    }
}

/// Possible ways to seek in the current song.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekMode {
    /// Forwards from current position.
    Forward(Duration),
    /// Backwards from current position.
    Backward(Duration),
    /// To the absolute position in the current song.
    Absolute(Duration),
}

/// `seekcur` command.
///
/// Seek in the current song.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seek(pub SeekMode);

impl Command for Seek {
    type Response = res::Empty;

    fn to_command(self) -> RawCommand {
        let time = match self.0 {
            SeekMode::Absolute(pos) => format!("{:.3}", pos.as_secs_f64()),
            SeekMode::Forward(time) => format!("+{:.3}", time.as_secs_f64()),
            SeekMode::Backward(time) => format!("-{:.3}", time.as_secs_f64()),
        };

        RawCommand::new("seekcur").argument(time)
    }
}

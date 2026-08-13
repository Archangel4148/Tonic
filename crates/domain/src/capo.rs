//! Capo position versus concert/sounding pitch.

use std::fmt;

use crate::key::Key;
use crate::note::{Note, Spelling};
use crate::pitch::Semitones;

/// Guitar capo fret. `0` means no capo.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Capo {
    fret: u8,
}

/// Invalid capo fret.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapoError {
    FretOutOfRange { fret: u8 },
}

impl Capo {
    pub const MAX_FRET: u8 = 12;

    #[must_use]
    pub const fn none() -> Self {
        Self { fret: 0 }
    }

    /// Creates a capo at `fret` (0–12).
    ///
    /// # Errors
    ///
    /// Returns [`CapoError::FretOutOfRange`] when `fret` is greater than 12.
    pub fn new(fret: u8) -> Result<Self, CapoError> {
        if fret > Self::MAX_FRET {
            return Err(CapoError::FretOutOfRange { fret });
        }
        Ok(Self { fret })
    }

    #[must_use]
    pub fn fret(self) -> u8 {
        self.fret
    }

    #[must_use]
    pub fn is_none(self) -> bool {
        self.fret == 0
    }
}

impl fmt::Display for CapoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FretOutOfRange { fret } => {
                write!(f, "capo fret {fret} is outside 0–{}", Capo::MAX_FRET)
            }
        }
    }
}

impl std::error::Error for CapoError {}

/// Concert/sounding pitch produced by a played shape under a capo.
#[must_use]
pub fn concert_pitch(played: Note, capo: Capo) -> Note {
    played.transpose(
        Semitones::new(i32::from(capo.fret())),
        Spelling::PreserveAccidentalFamily,
    )
}

/// Chord shape the player fingers to produce a concert pitch under a capo.
#[must_use]
pub fn played_shape(concert: Note, capo: Capo) -> Note {
    concert.transpose(
        Semitones::new(-i32::from(capo.fret())),
        Spelling::PreserveAccidentalFamily,
    )
}

/// Sounding key implied by played shapes under a capo.
#[must_use]
pub fn concert_key(played: Key, capo: Capo) -> Key {
    let tonic = concert_pitch(played.tonic(), capo);
    match played.mode() {
        crate::key::Mode::Major => Key::major(tonic),
        crate::key::Mode::Minor => Key::minor(tonic),
    }
}

/// Fret that makes `played` shapes sound in `concert` (same mode). `0` is unison.
#[must_use]
pub fn fret_between(played: Key, concert: Key) -> Option<u8> {
    if played.mode() != concert.mode() {
        return None;
    }
    let delta = (i32::from(concert.pitch_class().value()) - i32::from(played.pitch_class().value()))
        .rem_euclid(12) as u8;
    Capo::new(delta).ok().map(Capo::fret)
}

/// Played key/shapes implied by a sounding key under a capo.
#[must_use]
pub fn played_key(concert: Key, capo: Capo) -> Key {
    let tonic = played_shape(concert.tonic(), capo);
    match concert.mode() {
        crate::key::Mode::Major => Key::major(tonic),
        crate::key::Mode::Minor => Key::minor(tonic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::Key;

    #[test]
    fn sounding_a_capo_2_played_g() {
        let capo = Capo::new(2).unwrap();
        let sounding = Note::parse("A").unwrap();
        let played = played_shape(sounding, capo);
        assert_eq!(played.symbol(), "G");
        assert_eq!(concert_pitch(played, capo).symbol(), "A");

        let sounding_key = Key::parse("A").unwrap();
        assert_eq!(played_key(sounding_key, capo).symbol(), "G");
        assert_eq!(concert_key(Key::parse("G").unwrap(), capo).symbol(), "A");
    }

    #[test]
    fn fret_between_g_shapes_sounding_a_is_2() {
        let g = Key::parse("G").unwrap();
        let a = Key::parse("A").unwrap();
        assert_eq!(fret_between(g, a), Some(2));
        assert_eq!(fret_between(g, g), Some(0));
        assert_eq!(fret_between(g, Key::parse("Am").unwrap()), None);
    }

    #[test]
    fn no_capo_is_identity() {
        let note = Note::parse("F#").unwrap();
        assert_eq!(concert_pitch(note, Capo::none()), note);
        assert_eq!(played_shape(note, Capo::none()), note);
    }

    #[test]
    fn rejects_out_of_range_fret() {
        assert_eq!(Capo::new(13), Err(CapoError::FretOutOfRange { fret: 13 }));
    }
}

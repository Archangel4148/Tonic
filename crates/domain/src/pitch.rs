//! Pitch classes and semitone intervals, independent of note spelling.

use std::fmt;

/// Chromatic pitch class, with C = 0 and B = 11.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PitchClass(u8);

impl PitchClass {
    pub const C: Self = Self(0);
    pub const CS: Self = Self(1);
    pub const D: Self = Self(2);
    pub const DS: Self = Self(3);
    pub const E: Self = Self(4);
    pub const F: Self = Self(5);
    pub const FS: Self = Self(6);
    pub const G: Self = Self(7);
    pub const GS: Self = Self(8);
    pub const A: Self = Self(9);
    pub const AS: Self = Self(10);
    pub const B: Self = Self(11);

    /// Wraps any integer into `0..12`.
    #[must_use]
    pub fn new(value: impl Into<i32>) -> Self {
        Self(value.into().rem_euclid(12) as u8)
    }

    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn wrapping_add(self, semitones: Semitones) -> Self {
        Self::new(i32::from(self.0) + semitones.value())
    }

    /// Ascending distance in `0..12` semitones from `self` to `other`.
    #[must_use]
    pub fn ascending_to(self, other: Self) -> Semitones {
        Semitones::new(i32::from(other.0) - i32::from(self.0)).wrap_unsigned()
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Signed semitone offset. Spelling is intentionally not represented here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Semitones(i32);

impl Semitones {
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }

    /// Reduces to `0..12` while keeping direction as an ascending chromatic step.
    #[must_use]
    pub fn wrap_unsigned(self) -> Self {
        Self(self.0.rem_euclid(12))
    }
}

impl From<i32> for Semitones {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Semitones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_class_wraps() {
        assert_eq!(PitchClass::new(12), PitchClass::C);
        assert_eq!(PitchClass::new(-1), PitchClass::B);
        assert_eq!(PitchClass::new(13), PitchClass::CS);
    }

    #[test]
    fn wrapping_add_supports_negative_offsets() {
        assert_eq!(PitchClass::C.wrapping_add(Semitones::new(2)), PitchClass::D);
        assert_eq!(
            PitchClass::C.wrapping_add(Semitones::new(-1)),
            PitchClass::B
        );
        assert_eq!(
            PitchClass::F.wrapping_add(Semitones::new(14)),
            PitchClass::G
        );
    }

    #[test]
    fn ascending_interval_is_independent_of_spelling() {
        assert_eq!(PitchClass::C.ascending_to(PitchClass::GS).value(), 8);
        assert_eq!(PitchClass::A.ascending_to(PitchClass::A).value(), 0);
    }
}

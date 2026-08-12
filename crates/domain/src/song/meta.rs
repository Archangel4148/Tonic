//! Tempo, meter, and timestamps.

use serde::{Deserialize, Serialize};

/// Beats per minute.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Tempo {
    bpm: u16,
}

impl Tempo {
    #[must_use]
    pub fn new(bpm: u16) -> Option<Self> {
        (1..=400).contains(&bpm).then_some(Self { bpm })
    }

    #[must_use]
    pub fn bpm(self) -> u16 {
        self.bpm
    }
}

/// Time signature such as 4/4 or 6/8.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TimeSignature {
    numerator: u8,
    denominator: u8,
}

impl TimeSignature {
    #[must_use]
    pub fn new(numerator: u8, denominator: u8) -> Option<Self> {
        if numerator == 0 || denominator == 0 || !denominator.is_power_of_two() {
            return None;
        }
        Some(Self {
            numerator,
            denominator,
        })
    }

    #[must_use]
    pub fn numerator(self) -> u8 {
        self.numerator
    }

    #[must_use]
    pub fn denominator(self) -> u8 {
        self.denominator
    }

    #[must_use]
    pub fn symbol(self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}

/// Unix timestamp in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub fn now() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0);
        Self(secs)
    }

    #[must_use]
    pub fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    #[must_use]
    pub fn as_secs(self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo_rejects_zero_and_extreme_values() {
        assert!(Tempo::new(0).is_none());
        assert!(Tempo::new(72).is_some());
        assert!(Tempo::new(401).is_none());
    }

    #[test]
    fn time_signature_requires_power_of_two_denominator() {
        assert_eq!(TimeSignature::new(4, 4).unwrap().symbol(), "4/4");
        assert_eq!(TimeSignature::new(6, 8).unwrap().symbol(), "6/8");
        assert!(TimeSignature::new(4, 0).is_none());
        assert!(TimeSignature::new(4, 3).is_none());
    }
}

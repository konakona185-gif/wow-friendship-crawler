use bevy::prelude::{Component, Vec3};
use std::fmt::{Display, Formatter};

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct PendingDisplacement {
    translation: Vec3,
}

impl PendingDisplacement {
    pub fn accumulate(&mut self, translation: Vec3) -> Result<(), DisplacementError> {
        if !translation.is_finite() {
            return Err(DisplacementError::InvalidTranslation);
        }

        let updated = self.translation + translation;

        if !updated.is_finite() {
            return Err(DisplacementError::AccumulationOverflow);
        }

        self.translation = updated;

        Ok(())
    }

    pub fn pending_translation(&self) -> Vec3 {
        self.translation
    }

    pub fn take(&mut self) -> Vec3 {
        std::mem::take(&mut self.translation)
    }

    pub fn is_empty(&self) -> bool {
        self.translation == Vec3::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplacementError {
    InvalidTranslation,
    AccumulationOverflow,
}

impl Display for DisplacementError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTranslation => {
                formatter.write_str("Lo spostamento deve contenere coordinate finite")
            }
            Self::AccumulationOverflow => {
                formatter.write_str("L'accumulo degli spostamenti ha prodotto un overflow")
            }
        }
    }
}

impl std::error::Error for DisplacementError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_displacement_starts_empty() {
        let displacement = PendingDisplacement::default();

        assert_eq!(displacement.pending_translation(), Vec3::ZERO);

        assert!(displacement.is_empty());
    }

    #[test]
    fn pending_displacement_accumulates_translations() {
        let mut displacement = PendingDisplacement::default();

        displacement.accumulate(Vec3::new(2.0, 0.0, 1.0)).unwrap();

        displacement.accumulate(Vec3::new(3.0, 1.0, -2.0)).unwrap();

        assert_eq!(
            displacement.pending_translation(),
            Vec3::new(5.0, 1.0, -1.0)
        );

        assert!(!displacement.is_empty());
    }

    #[test]
    fn pending_displacement_rejects_non_finite_values() {
        let mut displacement = PendingDisplacement::default();

        for translation in [
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::new(f32::INFINITY, 0.0, 0.0),
            Vec3::new(0.0, f32::NEG_INFINITY, 0.0),
        ] {
            assert_eq!(
                displacement.accumulate(translation),
                Err(DisplacementError::InvalidTranslation)
            );
        }

        assert!(displacement.is_empty());
    }

    #[test]
    fn pending_displacement_rejects_accumulation_overflow() {
        let mut displacement = PendingDisplacement::default();

        displacement.accumulate(Vec3::splat(f32::MAX)).unwrap();

        let previous = displacement.pending_translation();

        assert_eq!(
            displacement.accumulate(Vec3::splat(f32::MAX)),
            Err(DisplacementError::AccumulationOverflow)
        );

        assert_eq!(displacement.pending_translation(), previous);
    }

    #[test]
    fn pending_displacement_take_returns_and_clears_translation() {
        let mut displacement = PendingDisplacement::default();

        displacement.accumulate(Vec3::new(4.0, 2.0, -3.0)).unwrap();

        let taken = displacement.take();

        assert_eq!(taken, Vec3::new(4.0, 2.0, -3.0));

        assert!(displacement.is_empty());
        assert_eq!(displacement.pending_translation(), Vec3::ZERO);
    }

    #[test]
    fn pending_displacement_can_accumulate_again_after_take() {
        let mut displacement = PendingDisplacement::default();

        displacement.accumulate(Vec3::X * 5.0).unwrap();

        displacement.take();

        displacement.accumulate(Vec3::Z * 3.0).unwrap();

        assert_eq!(displacement.pending_translation(), Vec3::Z * 3.0);
    }
}

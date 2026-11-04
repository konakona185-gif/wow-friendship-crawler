
use crate::content::AuraRefreshPolicy;

use super::AuraRuntimeError;

const PANDEMIC_CARRYOVER_CAP: f64 = 0.30;

pub(super) fn refreshed_duration(
    previous_duration: Option<f32>,
    new_duration: Option<f32>,
    policy: AuraRefreshPolicy,
) -> Result<Option<f32>, AuraRuntimeError> {
    match policy {
        AuraRefreshPolicy::ResetDuration => {
            Ok(new_duration)
        }
        AuraRefreshPolicy::Pandemic => {
            match (previous_duration, new_duration) {
                (Some(previous), Some(base)) => {
                    if !previous.is_finite() {
                        return Err(AuraRuntimeError::InvalidDuration);
                    }

                    let carryover = (previous as f64)
                        .max(0.0)
                        .min(base as f64 * PANDEMIC_CARRYOVER_CAP);

                    let refreshed = base as f64 + carryover;

                    if !refreshed.is_finite()
                        || refreshed > f32::MAX as f64
                    {
                        return Err(AuraRuntimeError::InvalidDuration);
                    }

                    Ok(Some(refreshed as f32))
                }
                (_, duration) => Ok(duration),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_duration_discards_previous_remaining_duration() {
        assert_eq!(
            refreshed_duration(
                Some(8.0),
                Some(10.0),
                AuraRefreshPolicy::ResetDuration,
            ),
            Ok(Some(10.0))
        );
    }

    #[test]
    fn pandemic_preserves_remaining_duration_below_thirty_percent() {
        assert_eq!(
            refreshed_duration(
                Some(2.0),
                Some(10.0),
                AuraRefreshPolicy::Pandemic,
            ),
            Ok(Some(12.0))
        );
    }

    #[test]
    fn pandemic_caps_remaining_duration_at_thirty_percent() {
        assert_eq!(
            refreshed_duration(
                Some(8.0),
                Some(10.0),
                AuraRefreshPolicy::Pandemic,
            ),
            Ok(Some(13.0))
        );
    }

    #[test]
    fn pandemic_uses_new_duration_without_existing_timed_aura() {
        assert_eq!(
            refreshed_duration(
                None,
                Some(10.0),
                AuraRefreshPolicy::Pandemic,
            ),
            Ok(Some(10.0))
        );
    }

    #[test]
    fn pandemic_preserves_permanent_new_duration() {
        assert_eq!(
            refreshed_duration(
                Some(5.0),
                None,
                AuraRefreshPolicy::Pandemic,
            ),
            Ok(None)
        );
    }

    #[test]
    fn pandemic_rejects_overflowing_refreshed_duration() {
        assert_eq!(
            refreshed_duration(
                Some(f32::MAX),
                Some(f32::MAX),
                AuraRefreshPolicy::Pandemic,
            ),
            Err(AuraRuntimeError::InvalidDuration)
        );
    }
}

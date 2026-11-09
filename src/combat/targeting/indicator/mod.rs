use bevy::prelude::*;

use crate::combat::{Character, Enemy, Target, TargetIndicator};

pub(in crate::combat) fn target_indicator_system(
    target_query: Query<&Target, With<Character>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut indicator_query: Query<(&TargetIndicator, &mut Transform, &mut Visibility), Without<Enemy>>,
) {
    for (indicator, mut indicator_transform, mut visibility) in &mut indicator_query {
        let selected = target_query
            .iter()
            .any(|target| target.entity == Some(indicator.enemy));

        if !selected {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Ok(enemy_transform) = enemy_query.get(indicator.enemy) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        indicator_transform.translation = enemy_transform.translation + Vec3::new(0.0, -0.47, 0.0);

        *visibility = Visibility::Visible;
    }
}

#[cfg(test)]
mod tests;

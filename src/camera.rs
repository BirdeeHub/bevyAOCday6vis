use bevy::prelude::*;
use crate::types::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Guard>)>,
    stateinfo: Res<StateInfo>,
    guards: Query<(&Transform, &Guard), Without<Camera2d>>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let mut guard = None;

    for (e, g) in &guards {
        if g.display_index == stateinfo.camera_target {
            guard = Some(e);
        }
    };

    if let Some(g) = guard {
        let Vec3{ x, y, .. } = g.translation;
        let direction = Vec3::new(x, y, camera.translation.z);

        // Applies a smooth effect to camera movement using stable interpolation
        // between the camera position and the player position on the x and y axes.
        camera
            .translation
            .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
    }
}

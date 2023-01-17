use bevy::prelude::*;

use crate::systems::apply_decode;
pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_decode);
    }
}

use bevy::{asset::Handle, ecs::system::Commands};
use bevy_vello::{Origin, VelloVector, VelloVectorBundle};

pub fn spawn_vector(vector_handle: Handle<VelloVector>, commands: &mut Commands) {
    commands.spawn(VelloVectorBundle {
        origin: Origin::Center,
        vector: vector_handle,
        ..Default::default()
    });
}

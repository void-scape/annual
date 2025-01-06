use crate::annual;
use bevy::prelude::*;

pub fn leaf_emitters(
    mut commands: Commands,
    server: Res<AssetServer>,
    emitter_query: Query<Entity, Added<annual::LeafEmitter>>,
    mut materials: ResMut<Assets<bevy_enoki::prelude::SpriteParticle2dMaterial>>,
) {
    for entity in emitter_query.iter() {
        let sprite_material =
            materials.add(bevy_enoki::prelude::SpriteParticle2dMaterial::from_texture(
                server.load("sprites/leaf1.png"),
            ));

        commands.entity(entity).with_child((
            bevy_enoki::ParticleSpawner(sprite_material),
            bevy_enoki::ParticleEffectHandle(server.load("particles/leaves.ron")),
            Transform::from_xyz(0., 0., 1.),
        ));
    }
}

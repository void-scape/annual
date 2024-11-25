use super::DialogueBoxRegistry;
use crate::dialogue_box::DialogueBoxDescriptor;
use bevy::prelude::*;
use rand::Rng;
use std::path::Path;

pub struct BoxPlugin {
    box_atlas_path: String,
    tile_size: UVec2,
}

impl BoxPlugin {
    pub fn new<P: AsRef<Path>>(box_atlas_path: &P, tile_size: UVec2) -> Self {
        Self {
            box_atlas_path: String::from(
                box_atlas_path
                    .as_ref()
                    .to_str()
                    .expect("invalid atlas path"),
            ),
            tile_size,
        }
    }
}

impl Plugin for BoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowDialogueBox>()
            .add_event::<HideDialogueBox>()
            .insert_resource(DialogueBoxAtlasPath(self.box_atlas_path.clone()))
            .insert_resource(DialogueBoxAtlasTileSize(self.tile_size))
            .add_systems(Startup, setup_atlas)
            .add_systems(Update, (handle_show_dialogue_box, handle_hide_dialogue_box));
    }
}

#[derive(Resource)]
struct DialogueBoxAtlas {
    texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    tile_size: UVec2,
}

#[derive(Resource)]
pub struct DialogueBoxAtlasPath(String);

#[derive(Resource)]
pub struct DialogueBoxAtlasTileSize(UVec2);

fn setup_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    atlas_path: Res<DialogueBoxAtlasPath>,
    atlas_tile_size: Res<DialogueBoxAtlasTileSize>,
) {
    let texture_atlas = TextureAtlasLayout::from_grid(atlas_tile_size.0, 3, 3, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let atlas = DialogueBoxAtlas {
        tile_size: atlas_tile_size.0,
        texture: asset_server.load(&atlas_path.0),
        atlas_layout: texture_atlas_handle.clone(),
    };

    commands.insert_resource(atlas);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogueBoxId(pub(super) u64);

impl DialogueBoxId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

#[derive(Event)]
pub struct ShowDialogueBox {
    pub id: DialogueBoxId,
    pub transform: Transform,
    pub inner_width: usize,
    pub inner_height: usize,
}

#[derive(Event)]
pub struct HideDialogueBox {
    pub id: DialogueBoxId,
}

pub struct DialogueBoxDimensions {
    pub inner_width: usize,
    pub inner_height: usize,
}

impl DialogueBoxDimensions {
    pub fn new(inner_width: usize, inner_height: usize) -> Self {
        Self {
            inner_width,
            inner_height,
        }
    }
}

pub fn show_dialogue_box(
    box_id: DialogueBoxId,
    transform: Transform,
    dimensions: DialogueBoxDimensions,
) -> impl Fn(EventWriter<ShowDialogueBox>) {
    move |mut writer: EventWriter<ShowDialogueBox>| {
        writer.send(ShowDialogueBox {
            id: box_id,
            transform,
            inner_width: dimensions.inner_width,
            inner_height: dimensions.inner_height,
        });
    }
}

pub fn hide_dialogue_box(box_id: DialogueBoxId) -> impl Fn(EventWriter<HideDialogueBox>) {
    move |mut writer: EventWriter<HideDialogueBox>| {
        writer.send(HideDialogueBox { id: box_id });
    }
}

fn handle_show_dialogue_box(
    mut commands: Commands,
    mut reader: EventReader<ShowDialogueBox>,
    atlas: Res<DialogueBoxAtlas>,
    mut registry: ResMut<DialogueBoxRegistry>,
) {
    for event in reader.read() {
        // info!("showing dialogue box: {:?}", event.id);

        registry.register(
            event.id,
            DialogueBoxDescriptor {
                dimensions: DialogueBoxDimensions::new(event.inner_width, event.inner_height),
                tile_size: atlas.tile_size,
                transform: event.transform,
            },
        );

        let width = 2 + event.inner_width;
        let height = 2 + event.inner_height;

        for y in 0..height {
            for x in 0..width {
                #[allow(clippy::collapsible_else_if)]
                let current_component = if y == 0 {
                    if x == 0 {
                        DialogueBoxComponent::TopLeft
                    } else if x < width - 1 {
                        DialogueBoxComponent::Top
                    } else {
                        DialogueBoxComponent::TopRight
                    }
                } else if y > 0 && y < height - 1 {
                    if x == 0 {
                        DialogueBoxComponent::MiddleLeft
                    } else if x < width - 1 {
                        DialogueBoxComponent::Middle
                    } else {
                        DialogueBoxComponent::MiddleRight
                    }
                } else {
                    if x == 0 {
                        DialogueBoxComponent::BottomLeft
                    } else if x < width - 1 {
                        DialogueBoxComponent::Bottom
                    } else {
                        DialogueBoxComponent::BottomRight
                    }
                };

                let mut transform = event.transform;
                transform.translation += Vec3::new(
                    x as f32 * atlas.tile_size.x as f32 * event.transform.scale.x,
                    -(y as i32) as f32 * atlas.tile_size.y as f32 * event.transform.scale.y,
                    0.0,
                );

                commands.spawn((
                    SpriteBundle {
                        texture: atlas.texture.clone(),
                        transform,
                        ..Default::default()
                    },
                    TextureAtlas {
                        layout: atlas.atlas_layout.clone(),
                        index: current_component.atlas_index(),
                    },
                    event.id,
                ));
            }
        }
    }
}

fn handle_hide_dialogue_box(
    mut commands: Commands,
    mut reader: EventReader<HideDialogueBox>,
    components: Query<(Entity, &DialogueBoxId)>,
    mut registry: ResMut<DialogueBoxRegistry>,
) {
    for event in reader.read() {
        info!("hiding dialogue box: {:?}", event.id);

        registry.remove(&event.id);

        for (entity, id) in components.iter() {
            if *id == event.id {
                commands.entity(entity).despawn();
            }
        }
    }
}

enum DialogueBoxComponent {
    TopLeft,
    Top,
    TopRight,
    MiddleLeft,
    Middle,
    MiddleRight,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl DialogueBoxComponent {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::TopLeft => 0,
            Self::Top => 1,
            Self::TopRight => 2,
            Self::MiddleLeft => 3,
            Self::Middle => 4,
            Self::MiddleRight => 5,
            Self::BottomLeft => 6,
            Self::Bottom => 7,
            Self::BottomRight => 8,
        }
    }
}

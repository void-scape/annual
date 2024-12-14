use super::text::WaveMaterial;
use crate::dialogue_box::text::UvRect;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::storage::ShaderStorageBuffer,
    sprite::Anchor,
    text::{PositionedGlyph, TextLayoutInfo},
    window::PrimaryWindow,
};
use bevy_bits::text::{TextMod, TypeWriterSection};

/// Generated for a an entity that contains a [`TypeWriterSection`] and a non empty [`TextLayoutInfo`].
///
/// Generation occurs after every change to the [`TextLayoutInfo`].
#[derive(Debug, Component)]
pub struct TextEffectInfo {
    pub atlas: Handle<Image>,
    pub extracted_glyphs: Vec<ExtractedGlyphs>,
}

/// ['PositionedGlyph'](bevy::text::PositionedGlyph) extracted from [`TextLayoutInfo`].
///
/// Disposed of after creating a [`TextMeshMaterial2d`](super::material::TextMeshMaterial2d) mesh.
#[derive(Debug)]
pub struct ExtractedGlyphs {
    pub glyphs: Vec<PositionedGlyph>,
    pub text_mod: TextMod,
}

pub fn compute_info(
    mut commands: Commands,
    mut sections: Query<(Entity, &TypeWriterSection, &mut TextLayoutInfo), Changed<TextLayoutInfo>>,
) {
    for (entity, section, mut text_layout_info) in sections.iter_mut() {
        let Some(atlas) = text_layout_info
            .glyphs
            .iter()
            .map(|g| g.atlas_info.texture.clone())
            .next()
        else {
            continue;
        };

        let mut extracted_glyphs = Vec::with_capacity(section.text.modifiers.len());
        let mut ranges = Vec::with_capacity(section.text.modifiers.len());

        for tm in section.text.modifiers.iter() {
            let start = tm.start;
            let end = tm.end.min(text_layout_info.glyphs.len());
            ranges.push(start..end);

            if text_layout_info.glyphs.len() > tm.start {
                extracted_glyphs.push(ExtractedGlyphs {
                    glyphs: text_layout_info.glyphs[start..end].to_vec(),
                    text_mod: tm.text_mod,
                });
            }
        }

        let mut index = 0;
        text_layout_info.glyphs.retain(|_| {
            let keep = !ranges.iter().any(|r| r.contains(&index));
            index += 1;
            keep
        });

        commands.entity(entity).insert(TextEffectInfo {
            atlas,
            extracted_glyphs,
        });
    }
}

/// Points to the glyph's root entity for change detection.
#[derive(Component)]
pub struct EffectGlyph {
    root: Entity,
    glyph: PositionedGlyph,
}

pub fn extract_effect_glyphs(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut text2d_query: Query<
        (
            Entity,
            &mut TextEffectInfo,
            &TextLayoutInfo,
            &TextColor,
            &Anchor,
            &Transform,
        ),
        With<TypeWriterSection>,
    >,
    mut wave_materials: ResMut<Assets<WaveMaterial>>,
    mut storage: ResMut<Assets<ShaderStorageBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if text2d_query
        .iter()
        .filter(|q| !q.1.extracted_glyphs.is_empty())
        .count()
        == 0
    {
        return;
    }

    // TODO: Support window-independent scaling: https://github.com/bevyengine/bevy/issues/5621
    let scale_factor = windows
        .get_single()
        .map(|window| window.resolution.scale_factor())
        .unwrap_or(1.0);
    let scaling = Transform::from_scale(Vec2::splat(scale_factor.recip()).extend(1.));

    let atlas_uvs = storage.add(ShaderStorageBuffer::with_size(
        std::mem::size_of::<UvRect>() * 128,
        RenderAssetUsages::all(),
    ));

    for (entity, mut effect_info, text_layout_info, color, anchor, transform) in
        text2d_query.iter_mut()
    {
        if effect_info.extracted_glyphs.is_empty() {
            continue;
        }

        let texture = effect_info.atlas.clone();
        let text_anchor = -(anchor.as_vec() + 0.5);
        let alignment_translation = text_layout_info.size * text_anchor;
        let transform =
            *transform * Transform::from_translation(alignment_translation.extend(0.)) * scaling;
        // let mut color = LinearRgba::WHITE;

        // let colors = storage.add(ShaderStorageBuffer::with_size(
        //     std::mem::size_of::<LinearRgba>() * 128,
        //     RenderAssetUsages::all(),
        // ));

        let mat = wave_materials.add(WaveMaterial {
            texture: texture.clone(),
            atlas_uvs: atlas_uvs.clone(),
            color: color.to_linear(),
        });

        for extracted_glyphs in effect_info.extracted_glyphs.drain(..) {
            match extracted_glyphs.text_mod {
                TextMod::Wave => {
                    commands.spawn_batch(
                        extracted_glyphs
                            .glyphs
                            .iter()
                            .map(|glyph| {
                                (
                                    Mesh2d(meshes.add(Rectangle::new(glyph.size.x, glyph.size.y))),
                                    MeshMaterial2d(mat.clone()),
                                    transform
                                        * Transform::from_translation(glyph.position.extend(0.)),
                                    EffectGlyph {
                                        root: entity,
                                        glyph: glyph.clone(),
                                    },
                                )
                            })
                            .collect::<Vec<_>>(),
                    );
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn order_effect_glyphs(mut glyphs: Query<&mut Transform, With<EffectGlyph>>) {
    for (z, mut t) in glyphs.iter_mut().enumerate() {
        t.translation.z = z as f32;
    }
}

pub fn flush_outdated_effect_glyphs(
    mut commands: Commands,
    effect_glyphs: Query<(Entity, &EffectGlyph)>,
    sections: Query<Entity, (Changed<TextLayoutInfo>, With<TypeWriterSection>)>,
) {
    for section in sections.iter() {
        println!("extracting section effect glyphs");
        effect_glyphs
            .iter()
            .filter(|(_, g)| g.root == section)
            .for_each(|(e, _)| commands.entity(e).despawn());
    }
}

pub fn update_buffers(
    meshes: Query<(&Transform, &MeshMaterial2d<WaveMaterial>, &EffectGlyph), Added<EffectGlyph>>,
    material_assets: Res<Assets<WaveMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    texture_atlases: Res<Assets<TextureAtlasLayout>>,
) {
    if meshes.is_empty() {
        return;
    }

    let material = material_assets
        .get(&meshes.iter().map(|(_, m, _)| m.0.clone()).next().unwrap())
        .unwrap();
    let buffer = buffers.get_mut(&material.atlas_uvs).unwrap();

    let mut meshes = meshes.iter().collect::<Vec<_>>();
    meshes.sort_by_key(|(t, _, _)| t.translation.z as i32);
    let atlas_uvs = meshes
        .iter()
        .map(|(_, _, g)| {
            let atlas = texture_atlases
                .get(&g.glyph.atlas_info.texture_atlas)
                .unwrap();
            let rect: UvRect = atlas.textures[g.glyph.atlas_info.location.glyph_index]
                .as_rect()
                .into();
            [
                rect.min[0] / atlas.size.x as f32,
                rect.max[1] / atlas.size.y as f32,
                rect.max[0] / atlas.size.x as f32,
                rect.min[1] / atlas.size.y as f32,
            ]
        })
        .collect::<Vec<_>>();
    buffer.set_data(&atlas_uvs);
}

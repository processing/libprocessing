#![cfg(target_arch = "wasm32")]

use bevy::prelude::Entity;
use processing_render::{
    config::Config, exit, geometry_box, geometry_sphere, graphics_begin_draw, graphics_end_draw,
    graphics_flush, graphics_record_command, image_create, image_destroy, image_load,
    image_readback, image_resize, init, material, material_create_pbr, material_destroy,
    material_set, render::command::DrawCommand, surface_create_from_canvas, surface_destroy,
    surface_resize, transform_look_at, transform_reset, transform_rotate_axis, transform_rotate_x,
    transform_rotate_y, transform_rotate_z, transform_scale, transform_set_position,
    transform_set_rotation, transform_set_scale, transform_translate,
};
use wasm_bindgen::prelude::*;

fn check<T, E: std::fmt::Display>(result: Result<T, E>) -> Result<T, JsValue> {
    result.map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(start)]
fn wasm_start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = "init")]
pub async fn js_init() -> Result<(), JsValue> {
    let config = Config::new();
    check(init(config).await)
}

#[wasm_bindgen(js_name = "createSurface")]
pub fn js_surface_create(canvas_id: &str, width: u32, height: u32) -> Result<u64, JsValue> {
    check(surface_create_from_canvas(canvas_id, width, height).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "destroySurface")]
pub fn js_surface_destroy(surface_id: u64) -> Result<(), JsValue> {
    check(surface_destroy(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "resizeSurface")]
pub fn js_surface_resize(surface_id: u64, width: u32, height: u32) -> Result<(), JsValue> {
    check(surface_resize(Entity::from_bits(surface_id), width, height))
}

#[wasm_bindgen(js_name = "background")]
pub fn js_background_color(surface_id: u64, r: f32, g: f32, b: f32, a: f32) -> Result<(), JsValue> {
    let color = bevy::color::Color::srgba(r, g, b, a);
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::BackgroundColor(color),
    ))
}

#[wasm_bindgen(js_name = "backgroundImage")]
pub fn js_background_image(surface_id: u64, image_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::BackgroundImage(Entity::from_bits(image_id)),
    ))
}

#[wasm_bindgen(js_name = "beginDraw")]
pub fn js_begin_draw(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_begin_draw(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "flush")]
pub fn js_flush(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_flush(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "endDraw")]
pub fn js_end_draw(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_end_draw(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "exit")]
pub fn js_exit(exit_code: u8) -> Result<(), JsValue> {
    check(exit(exit_code))
}

#[wasm_bindgen(js_name = "fill")]
pub fn js_fill(surface_id: u64, r: f32, g: f32, b: f32, a: f32) -> Result<(), JsValue> {
    let color = bevy::color::Color::srgba(r, g, b, a);
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Fill(color),
    ))
}

#[wasm_bindgen(js_name = "stroke")]
pub fn js_stroke(surface_id: u64, r: f32, g: f32, b: f32, a: f32) -> Result<(), JsValue> {
    let color = bevy::color::Color::srgba(r, g, b, a);
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::StrokeColor(color),
    ))
}

#[wasm_bindgen(js_name = "strokeWeight")]
pub fn js_stroke_weight(surface_id: u64, weight: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::StrokeWeight(weight),
    ))
}

#[wasm_bindgen(js_name = "noFill")]
pub fn js_no_fill(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::NoFill,
    ))
}

#[wasm_bindgen(js_name = "noStroke")]
pub fn js_no_stroke(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::NoStroke,
    ))
}

#[wasm_bindgen(js_name = "rect")]
pub fn js_rect(
    surface_id: u64,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tl: f32,
    tr: f32,
    br: f32,
    bl: f32,
) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Rect {
            x,
            y,
            w,
            h,
            radii: [tl, tr, br, bl],
        },
    ))
}

#[wasm_bindgen(js_name = "pushMatrix")]
pub fn js_push_matrix(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::PushMatrix,
    ))
}

#[wasm_bindgen(js_name = "popMatrix")]
pub fn js_pop_matrix(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::PopMatrix,
    ))
}

#[wasm_bindgen(js_name = "resetMatrix")]
pub fn js_reset_matrix(surface_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::ResetMatrix,
    ))
}

#[wasm_bindgen(js_name = "translate")]
pub fn js_translate(surface_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Translate { x, y },
    ))
}

#[wasm_bindgen(js_name = "rotate")]
pub fn js_rotate(surface_id: u64, angle: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Rotate { angle },
    ))
}

#[wasm_bindgen(js_name = "scale")]
pub fn js_scale(surface_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Scale { x, y },
    ))
}

#[wasm_bindgen(js_name = "shearX")]
pub fn js_shear_x(surface_id: u64, angle: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::ShearX { angle },
    ))
}

#[wasm_bindgen(js_name = "shearY")]
pub fn js_shear_y(surface_id: u64, angle: f32) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::ShearY { angle },
    ))
}

#[wasm_bindgen(js_name = "createImage")]
pub fn js_image_create(width: u32, height: u32, data: &[u8]) -> Result<u64, JsValue> {
    use bevy::render::render_resource::{Extent3d, TextureFormat};

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    check(image_create(size, data.to_vec(), TextureFormat::Rgba8UnormSrgb).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "loadImage")]
pub async fn js_image_load(url: &str) -> Result<u64, JsValue> {
    check(image_load(url).await.map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "resizeImage")]
pub fn js_image_resize(image_id: u64, new_width: u32, new_height: u32) -> Result<(), JsValue> {
    use bevy::render::render_resource::Extent3d;

    let new_size = Extent3d {
        width: new_width,
        height: new_height,
        depth_or_array_layers: 1,
    };
    check(image_resize(Entity::from_bits(image_id), new_size))
}

#[wasm_bindgen(js_name = "loadPixels")]
pub fn js_image_readback(image_id: u64) -> Result<Vec<f32>, JsValue> {
    let colors = check(image_readback(Entity::from_bits(image_id)))?;

    let mut result = Vec::with_capacity(colors.len() * 4);
    for color in colors {
        result.push(color.red);
        result.push(color.green);
        result.push(color.blue);
        result.push(color.alpha);
    }
    Ok(result)
}

#[wasm_bindgen(js_name = "destroyImage")]
pub fn js_image_destroy(image_id: u64) -> Result<(), JsValue> {
    check(image_destroy(Entity::from_bits(image_id)))
}

#[wasm_bindgen(js_name = "transformSetPosition")]
pub fn js_transform_set_position(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    check(transform_set_position(
        Entity::from_bits(entity_id),
        x,
        y,
        z,
    ))
}

#[wasm_bindgen(js_name = "transformTranslate")]
pub fn js_transform_translate(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    check(transform_translate(Entity::from_bits(entity_id), x, y, z))
}

#[wasm_bindgen(js_name = "transformSetRotation")]
pub fn js_transform_set_rotation(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    check(transform_set_rotation(
        Entity::from_bits(entity_id),
        x,
        y,
        z,
    ))
}

#[wasm_bindgen(js_name = "transformRotateX")]
pub fn js_transform_rotate_x(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    check(transform_rotate_x(Entity::from_bits(entity_id), angle))
}

#[wasm_bindgen(js_name = "transformRotateY")]
pub fn js_transform_rotate_y(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    check(transform_rotate_y(Entity::from_bits(entity_id), angle))
}

#[wasm_bindgen(js_name = "transformRotateZ")]
pub fn js_transform_rotate_z(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    check(transform_rotate_z(Entity::from_bits(entity_id), angle))
}

#[wasm_bindgen(js_name = "transformRotateAxis")]
pub fn js_transform_rotate_axis(
    entity_id: u64,
    angle: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) -> Result<(), JsValue> {
    check(transform_rotate_axis(
        Entity::from_bits(entity_id),
        angle,
        axis_x,
        axis_y,
        axis_z,
    ))
}

#[wasm_bindgen(js_name = "transformSetScale")]
pub fn js_transform_set_scale(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    check(transform_set_scale(Entity::from_bits(entity_id), x, y, z))
}

#[wasm_bindgen(js_name = "transformScale")]
pub fn js_transform_scale(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    check(transform_scale(Entity::from_bits(entity_id), x, y, z))
}

#[wasm_bindgen(js_name = "transformLookAt")]
pub fn js_transform_look_at(
    entity_id: u64,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) -> Result<(), JsValue> {
    check(transform_look_at(
        Entity::from_bits(entity_id),
        target_x,
        target_y,
        target_z,
    ))
}

#[wasm_bindgen(js_name = "transformReset")]
pub fn js_transform_reset(entity_id: u64) -> Result<(), JsValue> {
    check(transform_reset(Entity::from_bits(entity_id)))
}

#[wasm_bindgen(js_name = "materialCreatePbr")]
pub fn js_material_create_pbr() -> Result<u64, JsValue> {
    check(material_create_pbr().map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "materialSetFloat")]
pub fn js_material_set_float(mat_id: u64, name: &str, value: f32) -> Result<(), JsValue> {
    check(material_set(
        Entity::from_bits(mat_id),
        name,
        material::MaterialValue::Float(value),
    ))
}

#[wasm_bindgen(js_name = "materialSetFloat4")]
pub fn js_material_set_float4(
    mat_id: u64,
    name: &str,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), JsValue> {
    check(material_set(
        Entity::from_bits(mat_id),
        name,
        material::MaterialValue::Float4([r, g, b, a]),
    ))
}

#[wasm_bindgen(js_name = "materialDestroy")]
pub fn js_material_destroy(mat_id: u64) -> Result<(), JsValue> {
    check(material_destroy(Entity::from_bits(mat_id)))
}

#[wasm_bindgen(js_name = "material")]
pub fn js_material(surface_id: u64, mat_id: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(surface_id),
        DrawCommand::Material(Entity::from_bits(mat_id)),
    ))
}

#[wasm_bindgen(js_name = "geometryBox")]
pub fn js_geometry_box(width: f32, height: f32, depth: f32) -> Result<u64, JsValue> {
    check(geometry_box(width, height, depth).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "geometrySphere")]
pub fn js_geometry_sphere(radius: f32, sectors: u32, stacks: u32) -> Result<u64, JsValue> {
    check(geometry_sphere(radius, sectors, stacks).map(|e| e.to_bits()))
}

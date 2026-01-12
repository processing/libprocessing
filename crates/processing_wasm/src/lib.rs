#![cfg(target_arch = "wasm32")]

use bevy::prelude::Entity;
use processing_render::{
    config::Config, exit, graphics_begin_draw, graphics_end_draw, graphics_flush,
    graphics_record_command, image_create, image_destroy, image_load, image_readback, image_resize,
    init, render::command::DrawCommand, surface_create_from_canvas, surface_destroy,
    surface_resize,
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

use nannou::noise::{Fbm, MultiFractal, NoiseFn};
use nannou::prelude::*;
use rand::prelude::*;
use std::env;

fn main() {
    nannou::app(model).update(update).run();
}

fn get_dimensions() -> (u32, u32) {
    let w = env::var("WIDTH").unwrap_or_else(|_| "600".to_string()).parse().unwrap();
    let h = env::var("HEIGHT").unwrap_or_else(|_| "500".to_string()).parse().unwrap();
    (w, h)
}

struct Model {
    dimensions: (u32, u32),
    inv_dimensions: (f64, f64),
    texture: wgpu::Texture,
    noise: Fbm,
    time: f32,
    rng: rand::rngs::SmallRng,
}

fn model(app: &App) -> Model {
    let dimensions = get_dimensions();
    let window_id = app.new_window().title("ruidos")
        .size(dimensions.0, dimensions.1)
        .view(view).build().unwrap();

    let window = app.window(window_id).unwrap();
    let texture = wgpu::TextureBuilder::new()
        .size([dimensions.0, dimensions.1])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    let fbm = Fbm::new().set_octaves(7);
    Model {
        dimensions,
        inv_dimensions: (1.0 / dimensions.0 as f64, 1.0 / dimensions.1 as f64),
        texture,
        noise: fbm,
        time: 0.0,
        rng: SmallRng::from_thread_rng(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let base: u8 = if model.rng.gen_bool(16.0 / 17.0) { 0 } else { 16 };
    let variation: u8 = model.rng.gen_range(0..35);
    let flicker = model.rng.gen_bool(1.0 / 7.0);

    let (w, h) = (model.dimensions.0 as usize, model.dimensions.1 as usize);
    let mut data = vec![0u8; w * h * 4];
    let nz = model.time as f64 * 0.08;

    for y in 0..h {
        let ny = y as f64 * model.inv_dimensions.1;
        for x in 0..w {
            let nx = x as f64 * model.inv_dimensions.0;

            // carísimo el Perlin tridimencional, ver flamegraph.svg
            let noise_val = if !flicker {
                model.noise.get([nx, ny, nz])
            } else {
                model.noise.get([nz, nx])
//      model.noise.get([ny, nz])
            };

            let color = if noise_val < 0.3 {
                [base + 2, base + 1, base + 1, 40] // Water
            } else if noise_val < 0.7 {
                [40 + variation, 46, 34 + variation, 20] // Mountains
            } else {
                [20, 25, 255, 0] // Peaks
            };

            let index = (y * w + x) * 4;
            data[index..index + 4].copy_from_slice(&color);
        }
    }

    window.queue().write_texture(
        wgpu::ImageCopyTexture {
            texture: &model.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(model.dimensions.0 * 4),
            rows_per_image: Some(model.dimensions.1),
        },
        wgpu::Extent3d { width: model.dimensions.0, height: model.dimensions.1, depth_or_array_layers: 1, },
    );

    model.time += 0.22;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}

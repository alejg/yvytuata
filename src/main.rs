use nannou::noise::{Fbm, MultiFractal, NoiseFn};
use nannou::prelude::*;
use rand::Rng;

fn main() {
    nannou::app(model).update(update).run();
}

const W : u32 = 500;
const H : u32 = 400;
const WF : f64 = 500.0;
const HF : f64 = 400.0;

struct Model {
    texture: wgpu::Texture,
    noise: Fbm,
    time: f32,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window()
        .title("ruidos")
        .size(W, H)
        .view(view)
        .build().unwrap();

    let window = app.window(window_id).unwrap();
    let texture = wgpu::TextureBuilder::new()
        .size([W, H])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    let fbm = Fbm::new().set_octaves(8);
    Model {
        texture,
        noise: fbm,
        time: 0.0,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let mut rng = rand::thread_rng();
    let mut base:u8 = rng.gen_range(0, 17);
    if base < 16 {
        base = 0;
    }
    let variation_a:u8 = rng.gen_range(0, 35);
    let variation_b:u8 = rng.gen_range(0, 35);
    let flicker = rng.gen_bool(1.0 / 7.0);

    let w = W as usize;
    let h = H as usize;
    let mut data = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let nx = x as f64 / WF;
            let ny = y as f64 / HF;
            let nz = model.time as f64 * 0.08;

            // carísimo el Perlin tridimencional, ver flamegraph.svg
            let noise_val = if !flicker {
                model.noise.get([nx, ny, nz])
            } else {
                model.noise.get([nz, nx])
            };

            let color = if noise_val < 0.3 {
                // Water
                [base + 2, base + 1, base +1, 40] // Dark ocher
            } else if noise_val < 0.7 {
                // Mountains
                [40 + variation_a, 46, 34 + variation_b, 20] // Medium ocher
            } else {
                // Peaks
                [20, 25, 255, 0] // Light ocher
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
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );

    model.time += 0.22;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}
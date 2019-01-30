use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::ops::{Add, Mul, Sub};

mod geometry;
use geometry::*;

const SPHERE_RADIUS: f32 = 1.5; // all the explosion fits in a sphere with this radius. The center lies in the origin.
const NOISE_AMPLITUDE: f32 = 1.0; // amount of noise applied to the sphere (towards the center)

fn lerp<T>(v0: T, v1: T, t: f32) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
{
    v0 + (v1 - v0) * t.min(1.0).max(0.0)
}

fn hash(n: f32) -> f32 {
    let x = n.sin() * 43758.5453;
    x - x.floor()
}

fn noise(x: Vec3f) -> f32 {
    let p = vec3!(x.x.floor(), x.y.floor(), x.z.floor());
    let mut f = vec3!(x.x - p.x, x.y - p.y, x.z - p.z);
    f = f * (f * (vec3!(3.0, 3.0, 3.0) - f * 2.0));
    let n = p * vec3!(1.0, 57.0, 113.0);

    lerp(
        lerp(
            lerp(hash(n + 0.0), hash(n + 1.0), f.x),
            lerp(hash(n + 57.0), hash(n + 58.0), f.x),
            f.y,
        ),
        lerp(
            lerp(hash(n + 113.0), hash(n + 114.0), f.x),
            lerp(hash(n + 170.0), hash(n + 171.0), f.x),
            f.y,
        ),
        f.z,
    )
}

fn rotate(v: Vec3f) -> Vec3f {
    vec3!(
        vec3!(0.0, 0.8, 0.6) * v,
        vec3!(-0.8, 0.36, -0.48) * v,
        vec3!(-0.6, -0.48, 0.64) * v
    )
}

// this is a bad noise function with lots of artifacts. TODO: find a better one
fn fractal_brownian_motion(v: Vec3f) -> f32 {
    let mut p = rotate(v);
    let mut f = 0.0;
    f += 0.50 * noise(p);
    p = p * 2.32;
    f += 0.25 * noise(p);
    p = p * 3.03;
    f += 0.125 * noise(p);
    p = p * 2.61;
    f += 0.0625 * noise(p);

    f / 0.9375
}

// simple linear gradent yellow-orange-red-darkgray-gray. d is supposed to vary from 0 to 1
fn palette_fire(d: f32) -> Vec3f {
    let yellow = vec3!(66.0 / 255.0, 122.0 / 255.0, 169.0 / 255.0);
    let orange = vec3!(100.0 / 255.0, 143.0 / 255.0, 185.0 / 255.0);
    let red = vec3!(131.0 / 255.0, 157.0 / 255.0, 190.0 / 255.0);
    let darkgray = vec3!(209.0 / 255.0, 209.0 / 255.0, 211.0 / 255.0);
    let gray = vec3!(248.0 / 255.0, 243.0 / 255.0, 239.0 / 255.0);

    let x = d.min(1.0).max(0.0);
    if x < 0.25 {
        lerp(gray, darkgray, x * 4.0)
    } else if x < 0.5 {
        lerp(darkgray, red, x * 4.0 - 1.0)
    } else if x < 0.75 {
        lerp(red, orange, x * 4.0 - 2.0)
    } else {
        lerp(orange, yellow, x * 4.0 - 3.0)
    }
}

// this function defines the implicit surface we render
fn signed_distance(p: Vec3f) -> f32 {
    let displacement = -fractal_brownian_motion(p * 3.4) * NOISE_AMPLITUDE;
    return p.norm() - (SPHERE_RADIUS + displacement);
}

// Notice the early discard; in fact I know that the noise() function produces non-negative values,
// thus all the explosion fits in the sphere. Thus this early discard is a conservative check.
// It is not necessary, just a small speed-up
fn sphere_trace(orig: Vec3f, dir: Vec3f) -> Option<Vec3f> {
    if orig * orig - (orig * dir).powf(2.0) > SPHERE_RADIUS.powf(2.0) {
        return None;
    }

    let mut pos = orig;
    for _ in 0..128 {
        let d = signed_distance(pos);
        if d < 0.0 {
            return Some(pos);
        }

        pos = pos + dir * (d * 0.1).max(0.01); // note that the step depends on the current distance, if we are far from the surface, we can do big steps
    }
    None
}

// simple finite differences, very sensitive to the choice of the eps constant
fn distance_field_normal(pos: Vec3f) -> Vec3f {
    let eps = 0.1;
    let d = signed_distance(pos);
    let nx = signed_distance(pos + vec3!(eps, 0.0, 0.0)) - d;
    let ny = signed_distance(pos + vec3!(0.0, eps, 0.0)) - d;
    let nz = signed_distance(pos + vec3!(0.0, 0.0, eps)) - d;
    vec3!(nx, ny, nz).normalize()
}

fn main() {
    let width: usize = 640 * 2; // image width
    let height: usize = 480 * 2; // image height
    let fov: f32 = std::f32::consts::PI / 3.0; // field of view angle

    let mut framebuffer = vec![vec3!(0.0, 0.0, 0.0); width * height];

    // actual rendering loop
    framebuffer
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(j, line)| {
            for (i, pixel) in line.iter_mut().enumerate() {
                let dir_x = (i as f32 + 0.5) - width as f32 / 2.0;
                let dir_y = -(j as f32 + 0.5) + height as f32 / 2.0; // this flips the image at the same time
                let dir_z = -(height as f32) / (2.0 * (fov / 2.0).tan());

                // the camera is placed to (0,0,3) and it looks along the -z axis
                if let Some(hit) =
                    sphere_trace(vec3!(0.0, 0.0, 3.0), vec3!(dir_x, dir_y, dir_z).normalize())
                {
                    let noise_level = (SPHERE_RADIUS - hit.norm()) / NOISE_AMPLITUDE;
                    let light_dir = (vec3!(10.0, 10.0, 10.0) - hit).normalize(); // one light is placed to (10,10,10)
                    let light_intensity = (light_dir * distance_field_normal(hit)).max(0.4);

                    *pixel = palette_fire((-0.2 + noise_level) * 2.0) * light_intensity;
                } else {
                    *pixel = vec3!(0.2_f32.powf(2.2), 0.7_f32.powf(2.2), 0.8_f32.powf(2.2)); // background color
                }
            }
        });

    // save the framebuffer to file
    let file = File::create("./out.ppm").unwrap();
    let mut writer = BufWriter::new(file);
    write!(writer, "P6\n{} {}\n255\n", width, height).unwrap();
    for pixel in framebuffer {
        let p = vec3!(
            pixel.x.powf(1.0 / 2.2),
            pixel.y.powf(1.0 / 2.2),
            pixel.z.powf(1.0 / 2.2)
        ) * 255.0;
        let x = (p.x as i32).min(255).max(0) as u8;
        let y = (p.y as i32).min(255).max(0) as u8;
        let z = (p.z as i32).min(255).max(0) as u8;
        writer.write_all(&[x, y, z]).unwrap();
    }
}

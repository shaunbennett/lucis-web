use crate::geometry::volume::VolumetricSolid;
use crate::geometry::Ray;
use crate::scene::{Color, Intersect, Light, SceneNode};
use nalgebra::{convert, Affine3, Isometry, Point3, Rotation3, Vector3, U3};
use rand::{thread_rng, Rng};

use wasm_bindgen::Clamped;
use wasm_bindgen::prelude::*;

type Isometry3<N> = Isometry<N, U3, Rotation3<f32>>;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub struct Raytracer {
    pub root_node: SceneNode,

    // Viewing
    pub eye: Point3<f32>,
    pub view: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov_y: f32,

    // Lighting
    pub ambient: Color,
    pub lights: Vec<Light>,
    pub volumes: Vec<VolumetricSolid>,
}

impl Default for Raytracer {
    fn default() -> Raytracer {
        let root = SceneNode::new(0, String::from("root node"));
        Raytracer {
            root_node: root,
            eye: Point3::new(0.0, 0.0, 0.0),
            view: Point3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 0.0, 0.0),
            fov_y: 30.,
            ambient: Color::new(0.0, 0.0, 0.0),
            lights: Vec::new(),
            volumes: Vec::new(),
        }
    }
}

const Z_NEAR: f32 = -1.0;

impl Raytracer {
    // Ray trace and save a specific image
    pub fn render(
        &self,
        width: u32,
        height: u32
    ) -> Vec<u8> {
        console_log!("Start!");
        let view_matrix: Affine3<f32> =
            convert(Isometry3::look_at_rh(&self.eye, &self.view, &self.up));

        let side = -2.0f32 * (self.fov_y.to_radians() / 2.0f32).tan();
        let fw = width as f32;
        let fh = height as f32;

        let pixel_count = width * height;
        let mut u8Pixels: Vec<u8> = Vec::with_capacity((4 * pixel_count) as usize);

        (0..pixel_count)
            .into_iter()
            .map(|i| {
                let x = i % width;
                let y = i / height;
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;
                let pixel_vec = view_matrix
                    * Vector3::new(
                        Z_NEAR * ((fx / fw) - 0.5) * side * fw / fh,
                        Z_NEAR * -((fy / fh) - 0.5) * side,
                        Z_NEAR,
                    );
                let ray = Ray::new(self.eye, pixel_vec);
                self.trace_ray(width, height, &ray, x, y)
            })
            .for_each(|c| {
                u8Pixels.push((c.r * 255.0).round() as u8);
                u8Pixels.push((c.g * 255.0).round() as u8);
                u8Pixels.push((c.b * 255.0).round() as u8);
                u8Pixels.push(255u8);
            });

        u8Pixels
    }

    fn trace_ray(&self, width: u32, height: u32, ray: &Ray, x: u32, y: u32) -> Color {
        let collision = self.root_node.intersects(ray);
        match collision {
            Some(c) => {
                let mut color = c.node.material.get_color(ray, self, &c);
                for volume in self.volumes.iter() {
                    // TODO: don't do this
                    color = volume.apply(ray, &collision, color)
                }
                color
            }
            None => {
                let mut color = get_background_color(x, y, width, height);
                for volume in self.volumes.iter() {
                    // TODO: don't do this
                    color = volume.apply(ray, &collision, color)
                }
                color
            }
        }
    }
}

fn get_background_color(_x: u32, y: u32, _width: u32, height: u32) -> Color {
    // let fw = width as f32;
    let fh = height as f32;
    let r_rate = 67.0f32 / 255.;
    let g_rate = 133.0f32 / 255.;
    let b_rate = 1.0f32;
    let height_rate = f32::max(0.0f32, (y as f32 / fh) - 0.2f32);

    if height_rate <= 0.35 {
        let rand_chance = if height_rate >= 0.05 {
            let reverse_height = 0.4f32 - height_rate;
            let percent = reverse_height / 0.35f32;
            percent * 0.003f32
        } else {
            0.005f32
        };

        let mut rng = thread_rng();
        let render_star: f32 = rng.gen();
        if render_star <= rand_chance {
            // Render a star instead
            let gray_rand: f32 = rng.gen();
            let gray_range = 200.0f32;
            let gray = 55 + (gray_rand * gray_range) as i32;
            let value = gray as f32 / 255.0f32;
            return Color::new(value, value, value);
        }
    }

    Color::new(
        r_rate * height_rate,
        g_rate * height_rate,
        b_rate * height_rate,
    )
}

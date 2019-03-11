use crate::geometry::volume::VolumetricSolid;
use crate::geometry::Ray;
use crate::scene::{Color, Intersect, Light, SceneNode, Scene};
use nalgebra::{convert, Affine3, Isometry, Point3, Rotation3, Vector3, U3};
use rand::{thread_rng, Rng};

use web_sys::ImageData;
use wasm_bindgen::{JsCast, Clamped};
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

#[wasm_bindgen]
pub struct Raytracer {
    pub(crate) scene: Scene,

    // Viewing
    pub(crate) eye: Point3<f32>,
    pub(crate) view: Point3<f32>,
    pub(crate) up: Vector3<f32>,
    pub(crate) fov_y: f32,

    pub(crate) ambient: Color,
    pub(crate) lights: Vec<Light>,
    pub(crate) volumes: Vec<VolumetricSolid>,
}

const Z_NEAR: f32 = -1.0;

#[wasm_bindgen]
impl Raytracer {
    #[wasm_bindgen(constructor)]
    pub fn new(scene: Scene) -> Raytracer {
        Raytracer {
            scene: scene,
            eye: Point3::new(0.0, 0.0, 0.0),
            view: Point3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fov_y: 30.,
            ambient: Color::new(0.4, 0.4, 0.4),
            lights: Vec::new(),
            volumes: Vec::new(),
        }
    }

    // Ray trace and save a specific image
    pub fn render(
        &self,
        width: u32,
        height: u32
    ) {
        console_log!("Start!");
        let view_matrix: Affine3<f32> =
            convert(Isometry3::look_at_rh(&self.eye, &self.view, &self.up));
        console_log!("Start!1");

        let side = -2.0f32 * (self.fov_y.to_radians() / 2.0f32).tan();
        let fw = width as f32;
        let fh = height as f32;

        let pixel_count = width * height;
        let mut u8_pixels: Vec<u8> = Vec::with_capacity((4 * pixel_count) as usize);

        console_log!("Done!");
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
                u8_pixels.push((c.r * 255.0).round() as u8);
                u8_pixels.push((c.g * 255.0).round() as u8);
                u8_pixels.push((c.b * 255.0).round() as u8);
                u8_pixels.push(255u8);
            });
        console_log!("Done2!");

        // let mutArr: &mut [u8] = u8_pixels.as_mut();
        // let clamped: Clamped<&mut [u8]> = Clamped(mutArr);

        // console_log!("Done3!");
        // ImageData::new_with_u8_clamped_array_and_sh(clamped, 200, 200).unwrap()

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        canvas.set_width(width);
        canvas.set_height(height);

        context.fill_rect(0.0, 0.0, 500.0, 500.0);

        let mutArr: &mut [u8] = u8_pixels.as_mut();
        let clamped: Clamped<&mut [u8]> = Clamped(mutArr);

        let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(clamped, width, height).unwrap();
        context.put_image_data(&image_data, 0.0, 0.0).unwrap();
        // context.put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(&image_data, 0.0, 0.0, 0.0, 0.0, width as f64, height as f64).unwrap();
    }

    fn trace_ray(&self, width: u32, height: u32, ray: &Ray, x: u32, y: u32) -> Color {
        let collision = self.scene.intersects(ray);
        match collision {
            Some(c) => {
                // let mut color = c.node.material.get_color(ray, self, &c);
                let mut color = Color::new(1.0, 0.0, 0.0);
                // for volume in self.volumes.iter() {
                //     // TODO: don't do this
                //     color = volume.apply(ray, &collision, color)
                // }
                color
            }
            None => {
                let mut color = get_background_color(x, y, width, height);
                // for volume in self.volumes.iter() {
                //     // TODO: don't do this
                //     color = volume.apply(ray, &collision, color)
                // }
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

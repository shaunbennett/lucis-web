use crate::geometry::{Primitive, Ray};
use crate::scene::{Color, Intersection};
use crate::Raytracer;
use nalgebra::{clamp, distance_squared, Affine3, Matrix4, Vector3};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub enum Material {
    PhongMaterial {
        kd: Color,
        ks: Color,
        shininess: f32,
    },
    None,
}

fn calculate_phong_lighting(
    kd: &Color,
    ks: &Color,
    shininess: f32,
    _ray: &Ray,
    raytracer: &Raytracer,
    intersect: &Intersection,
) -> Color {
    let intersect_point = intersect.point;
    let n = intersect.normal.normalize();
    let v = (raytracer.eye - intersect_point).normalize();

    let mut final_color = *kd * raytracer.ambient;

    for light in raytracer.lights.iter() {
        let total_shadow_rays = light.num_samples;
        let mut shadow_rays_hit = 0;
        for p in light.light_samples.iter() {
            let shadow_ray = Ray::new_from_points(intersect_point, *p);
            if raytracer.scene.intersects(&shadow_ray).is_none() {
                shadow_rays_hit += 1;
            }
        }
        if shadow_rays_hit == 0 {
            continue;
        }

        let shadow_multiplier = shadow_rays_hit as f32 / total_shadow_rays as f32;

        let mut l = light.position - intersect_point;
        let l_norm = l.norm();
        l = l.normalize();

        let ldotn = clamp(l.dot(&n), 0.0f32, 1.0f32);
        let r = ((2.0f32 * ldotn * n) - l).normalize();
        let rdotv = clamp(r.dot(&v), 0.0f32, 1.0f32);
        let attenuation =
            light.falloff[0] + (light.falloff[1] * l_norm) + (light.falloff[2] * l_norm * l_norm);
        let light_sum = (kd * ldotn * light.color) + (ks * rdotv.powf(shininess) * light.color);
        final_color = final_color + (shadow_multiplier * (light_sum / attenuation));
    }

    final_color
}

impl Material {
    pub fn phong(kd: Color, ks: Color, shininess: f32) -> Material {
        Material::PhongMaterial { kd, ks, shininess }
    }

    pub fn get_color(&self, ray: &Ray, raytracer: &Raytracer, intersect: &Intersection) -> Color {
        match self {
            Material::PhongMaterial { kd, ks, shininess } => {
                calculate_phong_lighting(kd, ks, *shininess, ray, raytracer, intersect)
            }
            Material::None => Color::new(0.0, 0.0, 0.0),
        }
    }
}

#[wasm_bindgen]
pub struct Scene {
    pub(crate) nodes: Rc<RefCell<Vec<SceneNode>>>,
    // Index in the nodes vec of the root node
    pub(crate) root_node: usize,
}

#[wasm_bindgen]
pub struct SceneNodeRef {
    id: usize,
    parent: Rc<RefCell<Vec<SceneNode>>>,
}

#[wasm_bindgen]
impl SceneNodeRef {
    pub fn add_child(&mut self, child: &SceneNodeRef) {
        self.parent.borrow_mut()[self.id].add_child_id(child.id);
    }
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.parent.borrow_mut()[self.id].scale(x, y, z);
    }
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.parent.borrow_mut()[self.id].translate(x, y, z);
    }
    pub fn rotate(&mut self, axis: &str, angle: f32) {
        self.parent.borrow_mut()[self.id].rotate(axis, angle);
    }
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Scene {
        Scene {
            nodes: Rc::new(RefCell::new(Vec::new())),
            root_node: 0,
        }
    }

    pub fn create_node(&mut self, name: String) -> SceneNodeRef {
        let id = self.nodes.borrow().len();
        let mut node = SceneNode::new(id, name);
        node.primitive = Primitive::Sphere;
        // rt.material({0.9, 0.8, 0.4}, {0.8, 0.8, 0.4}, 25)
        node.material = Material::phong(Color::new(0.96, 0.37, 0.1), Color::new(0.7, 0.7, 0.7), 6.0);
        self.nodes.borrow_mut().push(node);
        return SceneNodeRef {
            id: id,
            parent: Rc::clone(&self.nodes),
        };
    }

    pub(crate) fn intersects(&self, ray: &Ray) -> Option<Intersection> {
        Scene::intersects_recursive(self.nodes.borrow(), self.root_node, ray)
    }

    fn intersects_recursive(nodes: Ref<Vec<SceneNode>>, current_node: usize, ray: &Ray) -> Option<Intersection> {
        let n = &nodes[current_node];
        let transformed_ray = n.inv_transform * *ray;

        let self_intersects = n.intersects(&transformed_ray);

        let min = n
            .children
            .iter()
            .map(|c_id| Scene::intersects_recursive(Ref::clone(&nodes), *c_id, &transformed_ray))
            .filter(|child| child.is_some())
            .map(|child| child.unwrap())
            .fold(None, |min, child| match min {
                None => Some(child),
                Some(cmin) => Some(
                    if distance_squared(&cmin.point, &transformed_ray.src)
                        < distance_squared(&child.point, &transformed_ray.src)
                    {
                        cmin
                    } else {
                        child
                    },
                ),
            });

        match (self_intersects, min) {
            (None, None) => None,
            (Some(a), None) => Some(a.apply_transform(&n.transform, &n.inv_transform)),
            (None, Some(a)) => Some(a.apply_transform(&n.transform, &n.inv_transform)),
            (Some(a), Some(b)) => Some(
                (if distance_squared(&a.point, &transformed_ray.src)
                    < distance_squared(&b.point, &transformed_ray.src)
                {
                    a
                } else {
                    b
                })
                .apply_transform(&n.transform, &n.inv_transform),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: usize,
    pub children: Vec<usize>,
    pub transform: Affine3<f32>,
    pub inv_transform: Affine3<f32>,
    pub name: String,

    // Material and Primitive
    pub material: Material,
    pub primitive: Primitive,
}

impl SceneNode {
    pub fn new(id: usize, name: String) -> SceneNode {
        SceneNode {
            id,
            children: Vec::new(),
            transform: Affine3::identity(),
            inv_transform: Affine3::identity(),
            name,
            material: Material::None,
            primitive: Primitive::None,
        }
    }
}

impl Intersect for SceneNode {
    fn intersects(&self, ray: &Ray) -> Option<Intersection> {
        let mut t_value: f32 = 0.0;
        let mut normal = Vector3::new(0.0f32, 0.0, 0.0);
        let mut uv = [0.0, 0.0];
        if self
            .primitive
            .collides(&ray, &mut t_value, &mut normal, &mut uv)
        {
            Some(Intersection::new(
                t_value,
                ray.src + (t_value * ray.dir.normalize()),
                self.id,
                normal,
                uv[0],
                uv[1],
            ))
        } else {
            None
        }
    }
}

impl SceneNode {
    pub fn add_child(&mut self, child: SceneNode) {
        self.children.push(child.id);
    }
    pub fn add_child_id(&mut self, child: usize) {
        self.children.push(child);
    }
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        println!("Applying scaling to {} of ({}, {}, {})", self.name, x, y, z);
        self.apply_transform(Matrix4::new_nonuniform_scaling(&Vector3::new(x, y, z)));
    }
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        println!(
            "Applying translation to {} of ({}, {}, {})",
            self.name, x, y, z
        );
        self.apply_transform(Matrix4::new_translation(&Vector3::new(x, y, z)));
    }
    pub fn rotate(&mut self, axis: &str, angle: f32) {
        println!(
            "Applying rotation to {} of ({}, {})",
            self.name, axis, angle
        );
        let axis = match axis {
            "x" | "X" => Vector3::x_axis(),
            "y" | "Y" => Vector3::y_axis(),
            "z" | "Z" => Vector3::z_axis(),
            _ => panic!(
                "Got unexpected axis: \'{}\' while trying to apply rotation to node \'{}\'",
                axis, self.name
            ),
        };
        self.apply_transform(Matrix4::from_axis_angle(&axis, angle.to_radians()));
    }
    fn apply_transform(&mut self, t: Matrix4<f32>) {
        let ta: Affine3<f32> = Affine3::from_matrix_unchecked(t);
        self.transform = ta * self.transform;
        self.inv_transform = self.transform.inverse();
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Intersection>;
}

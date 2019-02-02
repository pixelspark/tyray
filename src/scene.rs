extern crate image;
use super::geometry::{Vector, Ray};
use std::sync::{Arc};
use image::{DynamicImage, GenericImageView};

pub struct Scene {
	pub objects: Vec<Arc<Object>>,
	pub lights: Vec<Light>,
	pub environment_color: Vector,
	pub environment_map: Option<DynamicImage>
}

pub struct Light {
	pub position: Vector,
	pub intensity: f64
}

#[derive(Clone)]
pub struct Material {
	pub diffuse_color: Vector,
	pub specular_exponent: f64,
	pub albedo_diffuse: f64,
	pub albedo_reflect: f64,
	pub albedo_specular: f64,
	pub albedo_refract: f64,
	pub refractive_index: f64
}

pub trait Object: Send + Sync {
	fn intersect(&self, ray: &Ray) -> Option<f64>;
	fn material(&self) -> Arc<Material>;
	fn normal_at(&self, point: &Vector) -> Vector;
}

impl Scene {
	fn intersect(self: &Scene, ray: &Ray) -> (f64, Option<Arc<Object>>) {
		let mut min_dist: f64 = std::f64::MAX;
		let mut hit_object: Option<Arc<Object>> = None;

		// Find the first object hit by this ray
		for object in &self.objects {
			if let Some(distance) = object.intersect(ray) {
				if distance < min_dist {
					min_dist = distance;
					hit_object = Some(object.clone());
				}
			}
		}

		(min_dist, hit_object)
	}

	pub fn cast_ray(self: &Scene, ray: &Ray, depth: i32) -> Vector {
		if depth > 0 {
			let (min_dist, hit_object) = self.intersect(ray);

			// Render pixel
			if let Some(object) = hit_object {
				let material = object.material();
				let point = ray.extend(min_dist);
				let normal =  object.normal_at(&point).normalize();
				let mut diffuse_intensity = 0.0;
				let mut specular_intensity = 0.0;

				// Determine total light intensity
				for light in &self.lights {
					let light_direction = (light.position - point).normalize();

					// Shadow
					let light_distance = (light.position - point).norm();
					let shadow_origin = match light_direction.dot(&normal) {
						d if d < 0.0 => point - (normal * 0.001),
						_ => point + (normal * 0.001)
					};

					let (shadow_distance, shadow_obstacle) = self.intersect(&Ray { origin: shadow_origin, direction: light_direction });
					if shadow_obstacle.is_none() || shadow_distance > light_distance {
						// Light is not occluded
						diffuse_intensity += light.intensity * light_direction.dot(&normal).max(0.0);
						let specularity = (((light_direction * -1.0).reflect(normal) * -1.0).dot(&ray.direction)).max(0.0).powf(material.specular_exponent);
						specular_intensity += specularity * light.intensity;
					}
				}
				let diffuse_color = material.diffuse_color * diffuse_intensity * material.albedo_diffuse;
				let specular_color = Vector {x: 1.0, y: 1.0, z: 1.0} * specular_intensity * material.albedo_specular;

				// Reflection
				let reflect_direction = ray.direction.reflect(normal).normalize();
				let reflect_origin = match reflect_direction.dot(&normal) {
						d if d < 0.0 => point - (normal * 0.001),
						_ => point + (normal * 0.001)
					};
				let reflect_color = self.cast_ray(&Ray { origin: reflect_origin, direction: reflect_direction }, depth - 1) * material.albedo_reflect;

				// Refraction
				let refract_direction = ray.direction.refract(normal, material.refractive_index).normalize();
				let refract_origin = match refract_direction.dot(&normal) {
						d if d < 0.0 => point - (normal * 0.001),
						_ => point + (normal * 0.001)
					};
				let refract_color = self.cast_ray(&Ray { origin: refract_origin, direction: refract_direction }, depth - 1) * material.albedo_refract;

				// Determine lit pixel color
				return diffuse_color + specular_color + reflect_color + refract_color;
			}
		}

		// Environment
		let env_dir = ray.extend(1.0).normalize();
		match &self.environment_map {
			Some(image) => { 
				let ew = f64::from(image.width());
				let eh = f64::from(image.height());

				// https://stackoverflow.com/questions/39283698/direction-to-environment-map-uv-coordinates
				let m = (env_dir.x * env_dir.x + env_dir.y * env_dir.y + env_dir.z * env_dir.z).sqrt() * 2.0;
				let ex = ((env_dir.x / m  + 0.5) * ew) as u32;
				let ey = ((-env_dir.y / m + 0.5) * eh) as u32;
				let color = image.get_pixel(ex, ey);
				Vector { x: f64::from(color[0]) / 255.0, y: f64::from(color[1]) / 255.0, z: f64::from(color[2]) / 255.0 }
			},
			None => self.environment_color
		}
	}
}
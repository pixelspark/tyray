use super::geometry::{Ray, Vector};
use image::{DynamicImage, GenericImageView};
use std::sync::Arc;

pub struct Scene {
	pub objects: Vec<Arc<dyn Traceable>>,
	pub lights: Vec<Light>,
	pub environment_color: Vector,
	pub environment_map: Option<DynamicImage>,
}

pub struct Light {
	pub position: Vector,
	pub intensity: f64,
}

#[derive(Clone)]
pub struct Material {
	pub diffuse_color: Vector,
	pub specular_exponent: f64,
	pub albedo_diffuse: f64,
	pub albedo_reflect: f64,
	pub albedo_specular: f64,
	pub albedo_refract: f64,
	pub refractive_index: f64,
}

pub trait Traceable: Send + Sync {
	fn intersect(&self, ray: &Ray) -> Option<f64>;
	fn material(&self) -> Arc<Material>;
	fn normal_at(&self, point: &Vector) -> Vector;
}

impl Scene {
	fn intersect(self: &Scene, ray: &Ray) -> (f64, Option<Arc<dyn Traceable>>) {
		let mut min_dist: f64 = std::f64::MAX;
		let mut hit_object: Option<Arc<dyn Traceable>> = None;

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

	fn offset_orig(dir: Vector, point: Vector, n: Vector) -> Vector {
		if (dir ^ n) < 0.0 {
			point - (n * 1e-3)
		} else {
			point + (n * 1e-3)
		}
	}

	pub fn cast_ray(self: &Scene, ray: &Ray, depth: i32) -> Vector {
		if depth > 0 {
			let (min_dist, hit_object) = self.intersect(ray);

			// Render pixel
			if let Some(object) = hit_object {
				let material = object.material();
				let point = ray.extend(min_dist);
				let normal = object.normal_at(&point).normalize();
				let mut diffuse_intensity = 0.0;
				let mut specular_intensity = 0.0;

				// Determine total light intensity
				for light in &self.lights {
					let light_direction = (light.position - point).normalize();

					// Shadow
					let light_distance = (light.position - point).norm();
					let shadow_origin = Scene::offset_orig(light_direction, point, normal);

					let (shadow_distance, shadow_obstacle) =
						self.intersect(&Ray::new(shadow_origin, light_direction));
					if shadow_obstacle.is_none() || shadow_distance > light_distance {
						// Light is not occluded
						diffuse_intensity += light.intensity * (light_direction ^ normal).max(0.0);
						let specularity = (((light_direction * -1.0).reflect(normal) * -1.0)
							^ ray.direction())
						.max(0.0)
						.powf(material.specular_exponent);
						specular_intensity += specularity * light.intensity;
					}
				}
				let diffuse_color =
					material.diffuse_color * diffuse_intensity * material.albedo_diffuse;
				let specular_color = Vector {
					x: 1.0,
					y: 1.0,
					z: 1.0,
				} * specular_intensity
					* material.albedo_specular;

				// Reflection
				let reflect_direction = ray.direction().reflect(normal).normalize();
				let reflect_origin = Scene::offset_orig(reflect_direction, point, normal);
				let reflect_color = self
					.cast_ray(&Ray::new(reflect_origin, reflect_direction), depth - 1)
					* material.albedo_reflect;

				// Refraction
				let refract_direction = ray
					.direction()
					.refract(normal, material.refractive_index)
					.normalize();
				let refract_origin = Scene::offset_orig(refract_direction, point, normal);
				let refract_color = self
					.cast_ray(&Ray::new(refract_origin, refract_direction), depth - 1)
					* material.albedo_refract;

				// Determine lit pixel color
				return diffuse_color + specular_color + reflect_color + refract_color;
			}
		}

		// Environment
		let env_dir = ray.direction();
		match &self.environment_map {
			Some(image) => {
				let ew = f64::from(image.width());
				let eh = f64::from(image.height());

				// Spherical
				/*let m = env_dir.x.powf(2.0) + env_dir.y.powf(2.0) + (env_dir.z + 1.0).powf(2.0);
				let ex = (((env_dir.x / m) / 2.0 + 0.5) * ew) as u32;
				let ey = (((-env_dir.y / m) / 2.0 + 0.5) * eh) as u32;*/

				// https://stackoverflow.com/questions/39283698/direction-to-environment-map-uv-coordinates
				let m = env_dir.norm() * 2.0;
				let ex = ((-env_dir.z / m + 0.5) * ew) as u32;
				let ey = ((-env_dir.y / m + 0.5) * eh) as u32;
				let color = image.get_pixel(
					ex.min(image.width() - 1).max(0),
					ey.min(image.height() - 1).max(0),
				);
				Vector {
					x: f64::from(color[0]) / 255.0,
					y: f64::from(color[1]) / 255.0,
					z: f64::from(color[2]) / 255.0,
				}
			}
			None => self.environment_color,
		}
	}
}

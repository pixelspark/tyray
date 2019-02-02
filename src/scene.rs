use super::geometry::{Vector, Ray};
use std::sync::{Arc};

pub struct Scene {
	pub objects: Vec<Arc<Object>>,
	pub lights: Vec<Light>,
	pub environment_color: Vector
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

#[derive(Clone)]
pub struct Sphere {
	pub center: Vector,
	pub radius: f64,
	pub material: Arc<Material>
}

pub struct Plane {
	pub y: f64,
	pub x_min: f64,
	pub x_max: f64,
	pub z_min: f64,
	pub z_max: f64,
	pub material: Arc<Material>
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
				let normal =  object.normal_at(&point);
				let mut diffuse_intensity = 0.0;
				let mut specular_intensity = 0.0;

				// Determine total light intensity
				for light in &self.lights {
					let light_direction = light.position.sub(&point).normalize();

					// Shadow
					let light_distance = light.position.sub(&point).norm();
					let shadow_origin = match light_direction.dot(&normal) {
						d if d < 0.0 => point.sub(&normal.scale(0.001)),
						_ => point.add(&normal.scale(0.001))
					};

					let (shadow_distance, shadow_obstacle) = self.intersect(&Ray { origin: shadow_origin, direction: light_direction });
					if shadow_obstacle.is_none() || shadow_distance > light_distance {
						// Light is not occluded
						diffuse_intensity += light.intensity * light_direction.dot(&normal).max(0.0);
						let specularity = (&light_direction.scale(-1.0).reflect(&normal).scale(-1.0).dot(&ray.direction)).max(0.0).powf(material.specular_exponent);
						specular_intensity += specularity * light.intensity;
					}
				}
				let diffuse_color = material.diffuse_color.scale(diffuse_intensity).scale(material.albedo_diffuse);
				let specular_color = Vector {x: 1.0, y: 1.0, z: 1.0}.scale(specular_intensity).scale(material.albedo_specular);

				// Reflection
				let reflect_direction = ray.direction.reflect(&normal).normalize();
				let reflect_origin = match reflect_direction.dot(&normal) {
						d if d < 0.0 => point.sub(&normal.scale(0.001)),
						_ => point.add(&normal.scale(0.001))
					};
				let reflect_color = self.cast_ray(&Ray { origin: reflect_origin, direction: reflect_direction }, depth - 1).scale(material.albedo_reflect);

				// Refraction
				let refract_direction = ray.direction.refract(&normal, material.refractive_index).normalize();
				let refract_origin = match refract_direction.dot(&normal) {
						d if d < 0.0 => point.sub(&normal.scale(0.001)),
						_ => point.add(&normal.scale(0.001))
					};
				let refract_color = self.cast_ray(&Ray { origin: refract_origin, direction: refract_direction }, depth - 1).scale(material.albedo_refract);

				// Determine lit pixel color
				return diffuse_color.add(&specular_color).add(&reflect_color).add(&refract_color);
			}
		}

		// Environment
		self.environment_color
	}
}

impl Object for Plane {
	fn intersect(&self, ray: &Ray) -> Option<f64> {
		let d = -(ray.origin.y - self.y) / ray.direction.y;

		if d <= 0.0 {
			return None;
		}

		let pt = ray.extend(d);
		if pt.x >= self.x_min && pt.x <= self.x_max && pt.z >= self.z_min && pt.z <= self.z_max {
			return Some(d);
		}

		None
	}

	fn material(&self) -> Arc<Material> {
		self.material.clone()
	}

	fn normal_at(&self, _point: &Vector) -> Vector {
		Vector { x: 0.0, y: 1.0, z: 0.0 }
	}
}

impl Object for Sphere {
	fn intersect(&self, ray: &Ray) -> Option<f64> {
		let l = self.center.sub(&ray.origin);
		let tca = l.dot(&ray.direction);
		let d2 = l.dot(&l) - tca * tca;

		if d2 > self.radius {
			None
		}
		else {
			let thc = ((self.radius * self.radius) - d2).sqrt();
			let mut t0 = tca - thc;
			let t1 = tca + thc;
			
			if t0 < 0.0 {
				t0 = t1
			}
			if t0 < 0.0 {
				return None
			}

			Some(t0)
		}
	}

	fn material(&self) -> Arc<Material> {
		self.material.clone()
	}

	fn normal_at(&self, point: &Vector) -> Vector {
		point.sub(&self.center).normalize()
	}
}
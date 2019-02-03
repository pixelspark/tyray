use super::geometry::{Ray, Vector};
use super::scene::{Material, Traceable};
use std::sync::Arc;

#[derive(Clone)]
pub struct Sphere {
	pub center: Vector,
	pub radius: f64,
	pub material: Arc<Material>,
}

pub struct Plane {
	pub y: f64,
	pub x_min: f64,
	pub x_max: f64,
	pub z_min: f64,
	pub z_max: f64,
	pub material: Arc<Material>,
}

impl Traceable for Plane {
	fn intersect(&self, ray: &Ray) -> Option<f64> {
		let d = -(ray.origin().y - self.y) / ray.direction().y;

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
		Vector {
			x: 0.0,
			y: 1.0,
			z: 0.0,
		}
	}
}

impl Traceable for Sphere {
	fn intersect(&self, ray: &Ray) -> Option<f64> {
		let l = self.center - ray.origin();
		let tca = l ^ ray.direction();
		let d2 = l.dot(&l) - tca * tca;

		if d2 > self.radius {
			None
		} else {
			let thc = ((self.radius * self.radius) - d2).sqrt();
			let mut t0 = tca - thc;
			let t1 = tca + thc;

			if t0 < 0.0 {
				t0 = t1
			}
			if t0 < 0.0 {
				return None;
			}

			Some(t0)
		}
	}

	fn material(&self) -> Arc<Material> {
		self.material.clone()
	}

	fn normal_at(&self, point: &Vector) -> Vector {
		(*point - self.center).normalize()
	}
}

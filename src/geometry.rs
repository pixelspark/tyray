use std::ops::{Add, Sub, Mul};

#[derive(Clone, Copy)]
pub struct Vector {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

pub struct Ray {
	pub origin: Vector,
	pub direction: Vector
}

impl Ray {
	pub fn extend(self: &Ray, distance: f64) -> Vector {
		self.origin + (self.direction * distance)
	}
}

impl Vector {
	pub fn dot(self: &Vector, other: &Vector) -> f64 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn norm(self: &Vector) -> f64 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn normalize(self: &Vector) -> Vector {
		let norm = self.norm();
		Vector {
			x: self.x / norm,
			y: self.y / norm,
			z: self.z / norm
		}
	}

	pub fn reflect(self: Vector, normal: Vector) -> Vector {
		self - (normal * 2.0 * self.dot(&normal))
	}

	pub fn refract(self, normal: Vector, refractive_index: f64) -> Vector {
		let mut cosi = self.dot(&normal).min(1.0).max(-1.0);
		let mut etai = 1.0;
		let mut etat = refractive_index;
		let mut n = normal;
		if cosi < 0.0 { // if the ray is inside the object, swap the indices and invert the normal to get the correct result
			cosi = -cosi;
			std::mem::swap(&mut etat, &mut etai);
			n = n * -1.0;
		}
		let eta = etai / etat;
		let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

		if k < 0.0 {
			Vector { x: 1.0, y: 0.0, z: 0.0 }
		}
		else {
			(self * eta) + (n * (eta * cosi - k.sqrt()))
		}
	}
}

impl Add for Vector {
	type Output = Vector;

	fn add(self: Vector, other: Vector) -> Vector {
		Vector {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z
		}
	}
}

impl Sub for Vector {
	type Output = Vector;
	
	fn sub(self: Vector, other: Vector) -> Vector {
		Vector {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z
		}
	}
}

impl Mul<f64> for Vector {
	type Output = Vector;

	fn mul(self: Vector, scalar: f64) -> Vector {
		Vector {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar
		}
	}
}
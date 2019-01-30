extern crate image;
use std::rc::Rc;
use image::{ImageBuffer};

#[derive(Clone, Copy)]
struct Vector {
	x: f64,
	y: f64,
	z: f64,
}

#[derive(Clone)]
struct Material {
	diffuse_color: Vector,
	specular_exponent: f64,
	albedo_diffuse: f64,
	albedo_reflect: f64,
	albedo_specular: f64,
	albedo_refract: f64,
	refractive_index: f64
}

#[derive(Clone)]
struct Sphere {
	center: Vector,
	radius: f64,
	material: Rc<Material>
}

struct Plane {
	y: f64,
	x_min: f64,
	x_max: f64,
	z_min: f64,
	z_max: f64,
	material: Rc<Material>
}

trait Object {
	fn intersect(&self, ray: &Ray) -> Option<f64>;
	fn material(&self) -> Rc<Material>;
	fn normal_at(&self, point: &Vector) -> Vector;
}

struct Light {
	position: Vector,
	intensity: f64
}

struct Ray {
	origin: Vector,
	direction: Vector
}

impl Vector {
	fn sub(self: &Vector, other: &Vector) -> Vector {
		Vector {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z
		}
	}

	fn scale(self: &Vector, scalar: f64) -> Vector {
		Vector {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar
		}
	}

	fn add(self: &Vector, other: &Vector) -> Vector {
		Vector {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z
		}
	}

	fn dot(self: &Vector, other: &Vector) -> f64 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	fn norm(self: &Vector) -> f64 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	fn normalize(self: &Vector) -> Vector {
		let norm = self.norm();
		Vector {
			x: self.x / norm,
			y: self.y / norm,
			z: self.z / norm
		}
	}

	fn reflect(self: &Vector, normal: &Vector) -> Vector {
		self.sub(&normal.scale(2.0).scale(self.dot(normal)))
	}

	fn refract(&self, normal: &Vector, refractive_index: f64) -> Vector {
		let mut cosi = self.dot(normal).min(1.0).max(-1.0);
		let mut etai = 1.0;
		let mut etat = refractive_index;
		let mut n = *normal;
		if cosi < 0.0 { // if the ray is inside the object, swap the indices and invert the normal to get the correct result
			cosi = -cosi;
			let temp = etat;
			etat = etai;
			etai = temp;
			n = n.scale(-1.0);
		}
		let eta = etai / etat;
		let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

		if k < 0.0 {
			return Vector { x: 1.0, y: 0.0, z: 0.0 };
		}
		else {
			return self.scale(eta).add(&n.scale(eta * cosi - k.sqrt()));
		}
	}
}

impl Ray {
	fn extend(self: &Ray, distance: f64) -> Vector {
		self.origin.add(&self.direction.scale(distance))
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

	fn material(&self) -> Rc<Material> {
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

	fn material(&self) -> Rc<Material> {
		self.material.clone()
	}

	fn normal_at(&self, point: &Vector) -> Vector {
		point.sub(&self.center).normalize()
	}
}

struct Scene {
	objects: Vec<Rc<Box<Object>>>,
	lights: Vec<Light>,
	environment_color: Vector
}

impl Scene {
	fn intersect(self: &Scene, ray: &Ray) -> (f64, Option<Rc<Box<Object>>>) {
		let mut min_dist: f64 = std::f64::MAX;
		let mut hit_object: Option<Rc<Box<Object>>> = None;

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

	fn cast_ray(self: &Scene, ray: &Ray, depth: i32) -> Vector {
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
						diffuse_intensity += light.intensity * &light_direction.dot(&normal).max(0.0);
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

fn main() {
	let width = 2048;
	let height = 2048;

	let fov: f64 = 3.14159 / 2.0; // 90 degrees to radians

	let ivory = Rc::new(Material {
		albedo_diffuse: 0.6,
		albedo_specular: 0.3,
		albedo_reflect: 0.1,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.4, y: 0.4, z: 0.3 },
		specular_exponent: 50.0,
		refractive_index: 1.0
	});

	let red_rubber = Rc::new(Material {
		albedo_diffuse: 0.9,
		albedo_specular: 0.1,
		albedo_reflect: 0.0,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.3, y: 0.1, z: 0.1 },
		specular_exponent: 10.0,
		refractive_index: 1.0
	});

	let mirror = Rc::new(Material {
		albedo_diffuse: 0.0,
		albedo_specular: 10.0,
		albedo_reflect: 0.8,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 1.0, y: 1.0, z: 1.0 },
		specular_exponent: 1425.0,
		refractive_index: 1.0
	});

	let glass = Rc::new(Material {
		albedo_diffuse: 0.0,
		albedo_specular: 0.5,
		albedo_reflect: 0.1,
		albedo_refract: 0.9,
		diffuse_color: Vector { x: 0.6, y: 0.7, z: 0.8 },
		specular_exponent: 125.0,
		refractive_index: 1.3
	});

	let floor = Rc::new(Material {
		albedo_diffuse: 0.6,
		albedo_specular: 0.3,
		albedo_reflect: 0.2,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.2, y: 0.2, z: 0.2 },
		specular_exponent: 100.0,
		refractive_index: 1.0
	});

	let scene = Scene {
		environment_color: Vector { x: 0.2, y: 0.7, z: 0.8 },
		objects: vec![
			Rc::new(Box::new(Sphere {
				center: Vector { x: -3.0, y: 0.0, z: -16.0 }, radius: 6.0, material: ivory.clone()
			})),
			Rc::new(Box::new(Sphere {
				center: Vector { x: -1.0, y: -1.5, z: -8.0 }, radius: 2.0, material: glass.clone()
			})),
			Rc::new(Box::new(Sphere {
				center: Vector { x: 5.0, y: -3.0, z: -8.0 }, radius: 2.0, material: glass.clone()
			})),
			Rc::new(Box::new(Sphere {
				center: Vector { x: 1.5, y: -0.5, z: -18.0 }, radius: 3.0, material: red_rubber.clone()
			})),
			Rc::new(Box::new(Sphere {
				center: Vector { x: 7.0, y: 5.0, z: -18.0 }, radius: 4.0, material: ivory.clone()
			})),
			Rc::new(Box::new(Plane {
				x_min: -10.0,
				x_max: 10.0,
				z_min: -100.0,
				z_max: -5.0,
				y: -3.0,
				material: floor.clone()
			}))
		],
		lights: vec![
			Light { position: Vector { x: -20.0, y: 20.0, z: 20.0 }, intensity: 1.5 },
			Light { position: Vector { x: 30.0, y: 50.0, z: -25.0 }, intensity: 1.8 },
			Light { position: Vector { x: 30.0, y: 20.0, z: 30.0 }, intensity: 1.7 }
		]
	};

	// Construct a new by repeated calls to the supplied closure.
	let img = ImageBuffer::from_fn(width, height, |x, y| {
		let w = width as f64;
		let h = height as f64;
		let fx = (2.0 * ((x as f64) + 0.5) / w - 1.0) * ((fov / 2.0) * w / h).tan();
		let fy = (2.0 * (((height - y) as f64) + 0.5) / h - 1.0) * (fov / 2.0).tan();
		let dir = Vector { x: fx, y: fy, z: -1.0 }.normalize();

		let mut color = scene.cast_ray(&Ray { origin: Vector { x: 0.0, y: 0.0, z: 0.0}, direction: dir }, 6);

		// Scale color
		let max = color.x.max(color.y.max(color.z));
		if max > 1.0 {
			color = color.scale(1.0 / max);
		}

		image::Rgb([
			(color.x * 255.0).min(255.0).max(0.0) as u8,
			(color.y * 255.0).min(255.0).max(0.0) as u8,
			(color.z * 255.0).min(255.0).max(0.0) as u8
		])
	});

	img.save("test.png").unwrap();
}

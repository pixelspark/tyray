extern crate image;
extern crate rayon;
mod geometry;
mod scene;

use std::sync::{Arc};
use image::{ImageBuffer};
use rayon::prelude::*;
use geometry::{Vector, Ray};
use scene::{Light, Material, Scene, Sphere, Plane};

fn main() {
	let width = 512;
	let height = 512;

	let fov: f64 = std::f64::consts::PI / 2.0; // 90 degrees to radians

	println!("Configuring scene...");

	let ivory = Arc::new(Material {
		albedo_diffuse: 0.6,
		albedo_specular: 0.3,
		albedo_reflect: 0.1,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.4, y: 0.4, z: 0.3 },
		specular_exponent: 50.0,
		refractive_index: 1.0
	});

	let red_rubber = Arc::new(Material {
		albedo_diffuse: 0.9,
		albedo_specular: 0.1,
		albedo_reflect: 0.0,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.3, y: 0.1, z: 0.1 },
		specular_exponent: 10.0,
		refractive_index: 1.0
	});

	let mirror = Arc::new(Material {
		albedo_diffuse: 0.0,
		albedo_specular: 10.0,
		albedo_reflect: 0.8,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 1.0, y: 1.0, z: 1.0 },
		specular_exponent: 1425.0,
		refractive_index: 1.0
	});

	let glass = Arc::new(Material {
		albedo_diffuse: 0.0,
		albedo_specular: 0.5,
		albedo_reflect: 0.1,
		albedo_refract: 0.9,
		diffuse_color: Vector { x: 0.6, y: 0.7, z: 0.8 },
		specular_exponent: 125.0,
		refractive_index: 1.3
	});

	let floor = Arc::new(Material {
		albedo_diffuse: 0.6,
		albedo_specular: 0.3,
		albedo_reflect: 0.2,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.2, y: 0.2, z: 0.2 },
		specular_exponent: 100.0,
		refractive_index: 1.0
	});

	let scene = Arc::new(Scene {
		environment_color: Vector { x: 0.2, y: 0.7, z: 0.8 },
		environment_map: Some(image::open("./envmap.jpg").unwrap()),
		objects: vec![
			Arc::new(Sphere {
				center: Vector { x: -3.0, y: 0.0, z: -16.0 }, radius: 6.0, material: ivory.clone()
			}),
			Arc::new(Sphere {
				center: Vector { x: -1.0, y: -1.5, z: -8.0 }, radius: 2.0, material: glass.clone()
			}),
			Arc::new(Sphere {
				center: Vector { x: 5.0, y: -3.0, z: -8.0 }, radius: 2.0, material: glass.clone()
			}),
			Arc::new(Sphere {
				center: Vector { x: 1.5, y: -0.5, z: -18.0 }, radius: 3.0, material: red_rubber.clone()
			}),
			Arc::new(Sphere {
				center: Vector { x: 7.0, y: 5.0, z: -18.0 }, radius: 4.0, material: mirror.clone()
			}),
			Arc::new(Plane {
				x_min: -10.0,
				x_max: 10.0,
				z_min: -100.0,
				z_max: -5.0,
				y: -3.0,
				material: floor.clone()
			})
		],
		lights: vec![
			Light { position: Vector { x: -20.0, y: 20.0, z: 20.0 }, intensity: 1.5 },
			Light { position: Vector { x: 30.0, y: 50.0, z: -25.0 }, intensity: 1.8 },
			Light { position: Vector { x: 30.0, y: 20.0, z: 30.0 }, intensity: 1.7 }
		]
	});

	println!("Start rendering...");

	let image: Vec<Vec<_>> = (0 .. height).into_par_iter().map(move |y| {
		(0 .. width).map(|x| {
			let w = f64::from(width);
			let h = f64::from(height);
			let fx = (2.0 * (f64::from(x) + 0.5) / w - 1.0) * ((fov / 2.0) * w / h).tan();
			let fy = (2.0 * (f64::from(height - y) + 0.5) / h - 1.0) * (fov / 2.0).tan();
			let dir = Vector { x: fx, y: fy, z: -1.0 }.normalize();

			let mut color = scene.cast_ray(&Ray { origin: Vector { x: 0.0, y: 0.0, z: 0.0}, direction: dir }, 6);

			// Scale color
			let max = color.x.max(color.y.max(color.z));
			if max > 1.0 {
				color = color.scale(1.0 / max);
			}

			(x, y, image::Rgb([
				(color.x * 255.0).min(255.0).max(0.0) as u8,
				(color.y * 255.0).min(255.0).max(0.0) as u8,
				(color.z * 255.0).min(255.0).max(0.0) as u8
			]))
		}).collect()
	}).collect();

	println!("Rendered, writing to image...");

	let mut img = ImageBuffer::new(width, height);

	for row in image.iter() {
		for pixel in row {
			img.put_pixel(pixel.0, pixel.1, pixel.2)
		}
	}

	println!("Written, writing to disk...");
	img.save("test.png").unwrap();
}

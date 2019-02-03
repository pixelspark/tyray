mod geometry;
mod scene;
mod primitives;

use std::sync::{Arc};
use image::{ImageBuffer};
use rayon::prelude::*;
use geometry::{Vector, Ray};
use scene::{Light, Material, Scene};
use primitives::{Sphere, Plane};
use clap::{Arg, App};

fn main() {
	let app = App::new("tyray")
		.version("1.0")
		.author("Tommy van der Vorst <tommy@pixelspark.nl>")
		.about("Ray tracer")
		.arg(Arg::with_name("output")
			.help("Sets the output image file")
			.default_value("out.png")
			.required(true)
			.index(1)
		)
		.arg(Arg::with_name("width")
			.long("width")
			.help("Width of the output image")
			.default_value("512")
			.required(true)
		)
		.arg(Arg::with_name("height")
			.long("height")
			.help("Height of the output image")
			.default_value("512")
			.required(true)
		)
		.arg(Arg::with_name("fov")
			.long("fov")
			.help("Field of view angle")
			.default_value("90")
			.required(true)
		)
		.arg(Arg::with_name("depth")
			.long("depth")
			.help("Ray tracing depth")
			.default_value("6")
			.required(true)
		);
	
	let matches = app.get_matches();
	let output_path = matches.value_of("output").expect("no output path provided");

	// Output image width and height
	let width = matches.value_of("width").unwrap().parse().expect("invalid width");
	let height = matches.value_of("height").unwrap().parse().expect("invalid width");
	let fov_angle: f64 = matches.value_of("fov").unwrap().parse().expect("invalid fov");
	let max_depth: i32 = matches.value_of("depth").unwrap().parse().expect("invalid depth");
	assert!(width > 0);
	assert!(max_depth > 0);
	assert!(height > 0);
	assert!(fov_angle > 0.0 && fov_angle <= 360.0);

	// Field of view
	let fov: f64 = std::f64::consts::PI * 2.0 * fov_angle / 360.0;

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
		albedo_refract: 0.8,
		diffuse_color: Vector { x: 0.6, y: 0.7, z: 0.8 },
		specular_exponent: 125.0,
		refractive_index: 1.3
	});

	let floor = Arc::new(Material {
		albedo_diffuse: 0.3,
		albedo_specular: 0.3,
		albedo_reflect: 0.5,
		albedo_refract: 0.0,
		diffuse_color: Vector { x: 0.7, y: 0.7, z: 0.2 },
		specular_exponent: 100.0,
		refractive_index: 1.0
	});

	let scene = Arc::new(Scene {
		environment_color: Vector { x: 0.2, y: 0.7, z: 0.8 },
		environment_map: None, //Some(image::open("./envmap.jpg").unwrap()),
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

	// Iterate over all horizontal lines in parallel and render each line
	let image: Vec<Vec<_>> = (0 .. height).into_par_iter().map(move |y| {
		// Render each pixel on this line
		(0 .. width).map(|x| {
			let w = f64::from(width);
			let h = f64::from(height);
			let fx = (2.0 * (f64::from(x) + 0.5) / w - 1.0) * ((fov / 2.0) * w / h).tan();
			let fy = (2.0 * (f64::from(height - y) + 0.5) / h - 1.0) * (fov / 2.0).tan();
			let dir = Vector { x: fx, y: fy, z: -1.0 }.normalize();

			let mut color = scene.cast_ray(&Ray::new(Vector { x: 0.0, y: 0.0, z: 0.0 }, dir), max_depth);

			// Scale color
			let max = color.x.max(color.y.max(color.z));
			if max > 1.0 {
				color = color * (1.0 / max);
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
	img.save(output_path).unwrap();
}

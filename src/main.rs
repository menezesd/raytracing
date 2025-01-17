mod camera;
mod hitable;
mod material;
mod ray;
mod vec3;

use camera::Camera;
use hitable::{Hitable, HitableList, Sphere};
use material::Material;
use ray::Ray;
use std::fs;
use vec3::Vec3;

use rand::prelude::*;
use rayon::prelude::*;

fn color(r: &Ray, world: &dyn Hitable, depth: u32) -> Vec3 {
    match world.hit(r, 0.001, std::f64::INFINITY) {
        Some((hit_record, material)) => {
            let n = hit_record.normal();
            let p = hit_record.p();
            let (scattered, attenuation, b) = material.scatter(r, n, p);
            if depth < 50 && b {
                attenuation * color(&scattered, world, depth + 1)
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            }
        }
        None => {
            let unit_direction = r.direction().unit();
            let t = 0.5 * (unit_direction.y + 1.0);
            Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t
        }
    }
}

fn random_scene() -> HitableList {
    let mut rng = rand::thread_rng();
    let mut list: Vec<Box<dyn Hitable>> = vec![];
    list.push(Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Material::Lambertian {
            attenuation: Vec3::new(0.5, 0.5, 0.5),
        },
    )));
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(
                f64::from(a) + 0.9 * rng.gen::<f64>(),
                0.2,
                f64::from(b) + 0.9 * rng.gen::<f64>(),
            );
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    list.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Lambertian {
                            attenuation: Vec3::new(
                                rng.gen::<f64>() * rng.gen::<f64>(),
                                rng.gen::<f64>() * rng.gen::<f64>(),
                                rng.gen::<f64>() * rng.gen::<f64>(),
                            ),
                        },
                    )));
                } else if choose_mat < 0.95 {
                    list.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Metal {
                            attenuation: Vec3::new(
                                0.5 * (1.0 + rng.gen::<f64>()),
                                0.5 * (1.0 + rng.gen::<f64>()),
                                0.5 * (1.0 + rng.gen::<f64>()),
                            ),
                            fuzziness: 0.5 * rng.gen::<f64>(),
                        },
                    )));
                } else {
                    list.push(Box::new(Sphere::new(
                        center,
                        0.2,
                        Material::Dielectric { refraction: 1.5 },
                    )));
                }
            }
        }
    }
    list.push(Box::new(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Material::Dielectric { refraction: 1.5 },
    )));
    list.push(Box::new(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::Lambertian {
            attenuation: Vec3::new(0.4, 0.2, 0.1),
        },
    )));
    list.push(Box::new(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        Material::Metal {
            attenuation: Vec3::new(0.7, 0.6, 0.5),
            fuzziness: 0.0,
        },
    )));
    HitableList::new(list)
}

fn main() {
    println!(":: Generating the scene");
    let world = random_scene();

    let width = 640;
    let height = 480;
    let ray_per_pixel = 100;
    let max_color = 255;

    println!(":: Generating the image");
    let mut pic = format!("P3\n{} {}\n{}\n", width, height, max_color);

    let lookfrom = Vec3::new(16.0, 2.0, 4.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).length();
    let aperture = 0.2;

    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        15.0,
        f64::from(width) / f64::from(height),
        aperture,
        dist_to_focus,
    );

    let pixels = (0..height)
        .into_par_iter()
        .rev()
        .map(|h| {
            (0..width)
                .into_par_iter()
                .map(|w| {
                    let mut rng = rand::thread_rng();
                    let mut col = Vec3::new(0.0, 0.0, 0.0);
                    for _ in 0..ray_per_pixel {
                        let u = (f64::from(w) + rng.gen::<f64>()) / f64::from(width);
                        let v = (f64::from(h) + rng.gen::<f64>()) / f64::from(height);
                        let r = &camera.get_ray(u, v);
                        col = col + color(r, &world, 0);
                    }
                    col = col / f64::from(ray_per_pixel);
                    col = Vec3::new(col.x.sqrt(), col.y.sqrt(), col.z.sqrt());
                    let ir = (f64::from(max_color) * col.x) as usize;
                    let ig = (f64::from(max_color) * col.y) as usize;
                    let ib = (f64::from(max_color) * col.z) as usize;
                    format!("{} {} {}\n", ir, ig, ib)
                })
                .collect::<Vec<String>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join("");

    pic = format!("{}{}", &pic, pixels);

    println!(":: Writing the image");
    if fs::write("output.ppm", pic).is_err() {
        eprintln!("Could not generate the picture");
    };
}

use crate::*;

pub struct TeapotLand;

impl Scene for TeapotLand {
    fn world(&self) -> impl Hittable + Sync {
        let mut world: HitList<MaybeInstance> = HitList(vec![]);

        world.add(Sphere {
            center: Vector3(0.0, -1000.0, -0.0),
            radius: 1000.0,
            material: Arc::new(
                Lambertian {
                    albedo: Vector3(0.5, 0.5, 0.5),
                }
                .into(),
            ),
        });

        let mut rng = rng();
        for a in -11..11 {
            for b in -11..11 {
                let center = Vector3(
                    a as f64 + (rng.random::<f64>() * 0.9),
                    0.2,
                    b as f64 + (rng.random::<f64>() * 0.9),
                );
                if (center - Vector3(4.0, 0.2, 0.0)).len() > 0.9 {
                    let choose_mat = rng.random::<f64>();
                    let mat: MaterialEnum = if choose_mat < 0.8 {
                        Lambertian {
                            albedo: Vector3(
                                rng.random::<f64>(),
                                rng.random::<f64>(),
                                rng.random::<f64>(),
                            ),
                        }
                        .into()
                    } else if choose_mat < 0.95 {
                        Metalic {
                            albedo: Vector3(
                                (rng.random::<f64>() * 0.5) + 0.5,
                                (rng.random::<f64>() * 0.5) + 0.5,
                                (rng.random::<f64>() * 0.5) + 0.5,
                            ),
                            fuzz: rng.random::<f64>() * 0.5,
                        }
                        .into()
                    } else {
                        Dielectric {
                            refraction_index: 1.5,
                        }
                        .into()
                    };
                    world.add(Sphere {
                        center,
                        radius: 0.2,
                        material: Arc::new(mat),
                    });
                }
            }
        }

        // world.add(Sphere {
        //     center: Vector3(0.0, 1.0, 0.0),
        //     radius: 1.0,
        //     material: Arc::new(
        //         Dielectric {
        //             refraction_index: 1.5,
        //         }
        //         .into(),
        //     ),
        // });

        // world.add(Sphere {
        //     center: Vector3(-4.0, 1.0, 0.0),
        //     radius: 1.0,
        //     material: Arc::new(
        //         Emissive {
        //             emission: Vector3(4.0, 8.0, 8.0),
        //         }
        //         .into(),
        //     ),
        // });

        // world.add(Sphere {
        //     center: Vector3(4.0, 1.0, 0.0),
        //     radius: 1.0,
        //     material: Arc::new(
        //         Metalic {
        //             albedo: Vector3(0.7, 0.6, 0.5),
        //             fuzz: 0.0,
        //         }
        //         .into(),
        //     ),
        // });

        let instance = Instance {
            inner: TriangleMesh::from_stl_file(
                "teapot.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.8, 0.0, 0.8),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(0.0, 0.0, 0.0),
            rotation: Quaternion::from_euler(0.0, -PI / 2.0, PI / 2.0),
            scale: Vector3(0.2, 0.2, 0.2),
        };

        world.add(instance);

        Bvh::new(&mut world.0).unwrap()
    }

    fn camera(&self) -> Camera {
        Camera {
            aspect_ratio: 16. / 9.,
            image_width: 1920,
            fov: 20.0 * PI / 180.0,
            defocus_angle: 0.6 * PI / 180.0,
            lookfrom: Vector3(13.0, 2.0, 3.0),
            lookat: Vector3(0.0, 0.0, 0.0),
            vup: Vector3(0.0, 1.0, 0.0),
            samples_per_pixel: 500,
            max_bounce: 50,
        }
    }

    fn skybox(&self) -> impl Skybox + Sync {
        SkyishSkybox
    }
}

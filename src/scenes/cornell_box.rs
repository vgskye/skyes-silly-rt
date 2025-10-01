use crate::*;

pub struct CornellBox;

impl Scene for CornellBox {
    fn world(&self) -> impl Hittable + Sync {
        let mut world: HitList<MaybeInstance> = HitList(vec![]);

        world.add(Instance {
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
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Emissive {
                        emission: Vector3(15.0, 15.0, 15.0),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(213.0, 554.0, 227.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(130.0, 1.0, 105.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.73, 0.73, 0.73),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(0.0, -1.0, 0.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(555.0, 1.0, 555.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.73, 0.73, 0.73),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(0.0, 555.0, 0.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(555.0, 1.0, 555.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.73, 0.73, 0.73),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(0.0, 0.0, 555.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(555.0, 555.0, 1.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.65, 0.05, 0.05),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(-1.0, 0.0, 0.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(1.0, 555.0, 555.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.12, 0.45, 0.15),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(555.0, 0.0, 0.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 0.0),
            scale: Vector3(1.0, 555.0, 555.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.73, 0.73, 0.73),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(265.0, 0.0, 295.0),
            rotation: Quaternion::from_euler(0.0, 0.0, 15.0 * PI / 180.0),
            scale: Vector3(165.0, 330.0, 165.0),
        });

        world.add(Instance {
            inner: TriangleMesh::from_stl_file(
                "cube.stl",
                Arc::new(
                    Lambertian {
                        albedo: Vector3(0.73, 0.73, 0.73),
                    }
                    .into(),
                ),
            )
            .unwrap()
            .into(),
            translation: Vector3(130.0, 0.0, 65.0),
            rotation: Quaternion::from_euler(0.0, 0.0, -18.0 * PI / 180.0),
            scale: Vector3(165.0, 165.0, 165.0),
        });

        Bvh::new(&mut world.0).unwrap()
    }

    fn camera(&self) -> Camera {
        Camera {
            aspect_ratio: 1.,
            image_width: 640,
            fov: 40.0 * PI / 180.0,
            defocus_angle: 0.0 * PI / 180.0,
            lookfrom: Vector3(278.0, 278.0, -800.0),
            lookat: Vector3(278.0, 278.0, 0.0),
            vup: Vector3(0.0, 1.0, 0.0),
            samples_per_pixel: 500,
            max_bounce: 50,
        }
    }

    fn skybox(&self) -> impl Skybox + Sync {
        BlackSkybox
    }
}

use std::{
    f64::consts::PI,
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::OptionExt;
use knus::{Decode, DecodeScalar};

trait TryParse<T> {
    fn try_parse(&self) -> eyre::Result<T>;
}

#[derive(Decode, Clone, Debug)]
struct Scene {
    #[knus(child)]
    camera: Camera,
    #[knus(children)]
    instances: Vec<Instance>,
}

pub struct ParsedScene {
    skybox: Skybox,
    camera: crate::Camera,
    scene: crate::Bvh<crate::MaybeInstance>,
}

impl ParsedScene {
    pub fn from_path(path: &str) -> eyre::Result<Self> {
        let file = std::fs::read_to_string(path)?;
        let scene: Scene = knus::parse(path, &file)?;
        let skybox = scene.camera.skybox;
        let camera: crate::Camera = scene.camera.into();
        let mut instances: Vec<crate::MaybeInstance> = Vec::with_capacity(scene.instances.len());
        for instance in scene.instances {
            instances.push((&instance).try_into()?)
        }
        let bvh = crate::Bvh::new(&mut instances).ok_or_eyre("No instances in scene?")?;
        Ok(Self {
            skybox,
            camera,
            scene: bvh,
        })
    }
}

impl crate::Scene for ParsedScene {
    fn camera(&self) -> crate::Camera {
        self.camera
    }

    fn skybox(&self) -> impl crate::Skybox + Sync {
        self.skybox
    }

    fn world(&self) -> impl crate::Hittable + Sync {
        self.scene.clone()
    }
}

#[derive(Decode, Clone, Debug)]
enum Instance {
    Sphere(Sphere),
    Mesh(Mesh),
}
impl TryFrom<&Instance> for crate::MaybeInstance {
    type Error = std::io::Error;

    fn try_from(value: &Instance) -> Result<Self, Self::Error> {
        match value {
            Instance::Mesh(mesh) => mesh.try_into(),
            Instance::Sphere(sphere) => Ok((*sphere).into()),
        }
    }
}

#[derive(Decode, Clone, Copy, Debug)]
struct Sphere {
    #[knus(child)]
    center: Vector3,
    #[knus(child, unwrap(argument))]
    radius: f64,
    #[knus(child)]
    material: Material,
}

impl From<Sphere> for crate::MaybeInstance {
    fn from(value: Sphere) -> Self {
        crate::Sphere {
            center: value.center.into(),
            radius: value.radius,
            material: Arc::new(value.material.into()),
        }
        .into()
    }
}

#[derive(Decode, Clone, Debug)]
struct Mesh {
    #[knus(child, unwrap(argument))]
    stl: PathBuf,
    #[knus(child, default = Vector3(0.0, 0.0, 0.0))]
    translation: Vector3,
    #[knus(child, default = Vector3(1.0, 1.0, 1.0))]
    scale: Vector3,
    #[knus(child, default = Vector3(0.0, 0.0, 0.0))]
    rotation: Vector3,
    #[knus(child)]
    material: Material,
}

impl TryFrom<&Mesh> for crate::MaybeInstance {
    type Error = std::io::Error;

    fn try_from(value: &Mesh) -> Result<Self, Self::Error> {
        let mesh = crate::TriangleMesh::from_stl_file(&value.stl, Arc::new(value.material.into()))?;
        Ok(crate::Instance {
            inner: mesh.into(),
            translation: value.translation.into(),
            scale: value.scale.into(),
            rotation: crate::quat::Quaternion::from_euler(
                value.rotation.0 * PI / 180.0,
                value.rotation.1 * PI / 180.0,
                value.rotation.2 * PI / 180.0,
            ),
        }
        .into())
    }
}

#[derive(Decode, Clone, Copy, Debug)]
struct Material {
    #[knus(argument)]
    kind: MaterialKind,
    #[knus(child, default = Vector3(1.0, 1.0, 1.0))]
    albedo: Vector3,
    #[knus(child, unwrap(argument), default = 0.0)]
    fuzz: f64,
    #[knus(child, unwrap(argument), default = 1.0)]
    refraction_index: f64,
}

impl From<Material> for crate::MaterialEnum {
    fn from(value: Material) -> Self {
        match value.kind {
            MaterialKind::Lambertian => crate::Lambertian {
                albedo: value.albedo.into(),
            }
            .into(),
            MaterialKind::Metalic => crate::Metalic {
                albedo: value.albedo.into(),
                fuzz: value.fuzz,
            }
            .into(),
            MaterialKind::Emissive => crate::Emissive {
                emission: value.albedo.into(),
            }
            .into(),
            MaterialKind::Dielectric => crate::Dielectric {
                refraction_index: value.refraction_index,
            }
            .into(),
        }
    }
}

#[derive(DecodeScalar, Clone, Copy, Debug)]
enum MaterialKind {
    Lambertian,
    Metalic,
    Emissive,
    Dielectric,
}

#[derive(Decode, Clone, Copy, Debug)]
struct Vector3(
    #[knus(argument)] f64,
    #[knus(argument)] f64,
    #[knus(argument)] f64,
);

impl From<Vector3> for crate::Vector3 {
    fn from(value: Vector3) -> Self {
        crate::Vector3(value.0, value.1, value.2)
    }
}

#[derive(Decode, Clone, Copy, Debug)]
struct AspectRatio {
    #[knus(argument)]
    width: f64,
    #[knus(argument)]
    height: f64,
}

#[derive(Decode, Clone, Copy, Debug)]
struct Camera {
    #[knus(child, unwrap(argument), default = Skybox::Black)]
    pub skybox: Skybox,
    #[knus(child, default = AspectRatio { width: 16.0, height: 9.0 })]
    pub aspect_ratio: AspectRatio,
    #[knus(child, unwrap(argument), default = 60.0)]
    pub fov: f64,
    #[knus(child, unwrap(argument), default = 0.0)]
    pub defocus_angle: f64,
    #[knus(child, unwrap(argument), default = 1920)]
    pub image_width: u32,
    #[knus(child)]
    pub lookfrom: Vector3,
    #[knus(child)]
    pub lookat: Vector3,
    #[knus(child, default = Vector3(0.0, 1.0, 0.0))]
    pub vup: Vector3,
    #[knus(child, unwrap(argument), default = 500)]
    pub samples_per_pixel: usize,
    #[knus(child, unwrap(argument), default = 32)]
    pub max_bounce: usize,
}

impl From<Camera> for crate::Camera {
    fn from(value: Camera) -> Self {
        Self {
            aspect_ratio: value.aspect_ratio.width / value.aspect_ratio.height,
            fov: value.fov * PI / 180.0,
            defocus_angle: value.defocus_angle * PI / 180.0,
            image_width: value.image_width,
            lookfrom: value.lookfrom.into(),
            lookat: value.lookat.into(),
            vup: value.vup.into(),
            samples_per_pixel: value.samples_per_pixel,
            max_bounce: value.max_bounce,
        }
    }
}

#[derive(DecodeScalar, Clone, Copy, Debug)]
enum Skybox {
    Skyish,
    Black,
}

impl crate::Skybox for Skybox {
    fn environ(&self, ray: crate::Ray) -> crate::Vector3 {
        match self {
            Skybox::Skyish => crate::SkyishSkybox.environ(ray),
            Skybox::Black => crate::BlackSkybox.environ(ray),
        }
    }
}

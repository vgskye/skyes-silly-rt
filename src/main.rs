mod quat;
mod rng_utils;
mod vec3;

use std::{
    f64::{self, consts::PI},
    fs::File,
    ops::{Add, Mul, RangeBounds},
    path::Path,
    sync::Arc,
};

use image::RgbImage;
use indicatif::ProgressBar;
use rand::Rng;
use rng_utils::rng;
pub use vec3::Vector3;

use crate::{
    quat::Quaternion,
    rng_utils::{UniformDisc, UniformSphere},
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct RangeExclusive(pub f64, pub f64);

impl RangeBounds<f64> for RangeExclusive {
    fn start_bound(&self) -> std::ops::Bound<&f64> {
        std::ops::Bound::Excluded(&self.0)
    }

    fn end_bound(&self) -> std::ops::Bound<&f64> {
        std::ops::Bound::Excluded(&self.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    fn at(self, t: f64) -> Vector3 {
        self.origin + self.direction * t
    }
}

pub fn lerp<T: Mul<f64, Output = U>, U: Add>(lhs: T, rhs: T, factor: f64) -> U::Output {
    lhs * (1.0 - factor) + rhs * factor
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct ScatterResult {
    pub scattered: Option<Ray>,
    pub attenuation: Vector3,
}

trait Material {
    fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Lambertian {
    pub albedo: Vector3,
}

impl Material for Lambertian {
    fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult {
        let mut bounce = rng.sample(UniformSphere) + hit.normal;
        if bounce.is_small() {
            bounce = hit.normal;
        }
        ScatterResult {
            scattered: Some(Ray {
                origin: hit.point,
                direction: bounce.normalize(),
            }),
            attenuation: self.albedo,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Metalic {
    pub albedo: Vector3,
    pub fuzz: f64,
}

impl Material for Metalic {
    fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult {
        let mut bounce = ray.direction - hit.normal * ((ray.direction * hit.normal) * 2.0);
        if self.fuzz > 0.0 {
            bounce += rng.sample(UniformSphere) * self.fuzz;
            if bounce * hit.normal <= 0.0 {
                return ScatterResult {
                    scattered: None,
                    attenuation: Vector3(0.0, 0.0, 0.0),
                };
            }
        }
        ScatterResult {
            scattered: Some(Ray {
                origin: hit.point,
                direction: bounce.normalize(),
            }),
            attenuation: self.albedo,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Dielectric {
    pub refraction_index: f64,
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult {
        let ri = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let cos_theta = ((ray.direction * -1.0) * hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let r0 = (1.0 - ri) / (1.0 + ri);
        let r0 = r0 * r0;
        let reflectance = r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5);

        let direction = if ri * sin_theta > 1.0 || reflectance > rng.random::<f64>() {
            ray.direction - hit.normal * ((ray.direction * hit.normal) * 2.0)
        } else {
            let r_out_perp = (ray.direction + hit.normal * cos_theta) * ri;
            let r_out_parallel = hit.normal * -(1.0 - r_out_perp * r_out_perp).abs().sqrt();
            r_out_perp + r_out_parallel
        };

        ScatterResult {
            scattered: Some(Ray {
                origin: hit.point,
                direction,
            }),
            attenuation: Vector3(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Emissive {
    pub emission: Vector3,
}

impl Material for Emissive {
    fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult {
        ScatterResult {
            scattered: None,
            attenuation: self.emission,
        }
    }
}

macro_rules! materials_enum {
    ($($ty:ident),+) => {
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
        enum MaterialEnum {
            $($ty($ty)),+
        }

        impl Material for MaterialEnum {
            fn scatter(&self, ray: Ray, hit: Hit, rng: &mut impl Rng) -> ScatterResult {
                match &self {
                    $(Self::$ty(val) => val.scatter(ray, hit, rng)),+
                }
            }
        }


        $(impl From<$ty> for MaterialEnum {
            fn from(value: $ty) -> MaterialEnum {
                MaterialEnum::$ty(value)
            }
        })+
    };
}

materials_enum!(Lambertian, Metalic, Emissive, Dielectric);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Hit {
    pub point: Vector3,
    pub normal: Vector3,
    pub t: f64,
    pub front_face: bool,
}

impl Hit {
    fn new(ray: Ray, point: Vector3, outward_normal: Vector3, t: f64) -> Self {
        let front_face = ray.direction * outward_normal < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            outward_normal * -1.0
        };
        Self {
            point,
            normal,
            t,
            front_face,
        }
    }
}

trait Hittable {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)>;
    fn aabb(&self) -> Aabb;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Aabb(pub (f64, f64), pub (f64, f64), pub (f64, f64));

impl Aabb {
    fn new(a: Vector3, b: Vector3) -> Self {
        Self(
            (a.0.min(b.0), a.0.max(b.0)),
            (a.1.min(b.1), a.1.max(b.1)),
            (a.2.min(b.2), a.2.max(b.2)),
        )
    }

    fn combine(a: Aabb, b: Aabb) -> Self {
        Self(
            (a.0.0.min(b.0.0), a.0.1.max(b.0.1)),
            (a.1.0.min(b.1.0), a.1.1.max(b.1.1)),
            (a.2.0.min(b.2.0), a.2.1.max(b.2.1)),
        )
    }
    fn hit_cached(&self, ray: Ray, t_range: RangeExclusive, dir_inv: Vector3) -> bool {
        let min = t_range.0;
        let max = t_range.1;

        let t0 = (self.0.0 - ray.origin.0) * dir_inv.0;
        let t1 = (self.0.1 - ray.origin.0) * dir_inv.0;

        let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
        let min = if min <= t0 { t0 } else { min };
        let max = if max >= t1 { t1 } else { max };

        if min >= max {
            return false;
        }

        let t0 = (self.1.0 - ray.origin.1) * dir_inv.1;
        let t1 = (self.1.1 - ray.origin.1) * dir_inv.1;

        let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
        let min = if min <= t0 { t0 } else { min };
        let max = if max >= t1 { t1 } else { max };

        if min >= max {
            return false;
        }

        let t0 = (self.2.0 - ray.origin.2) * dir_inv.2;
        let t1 = (self.2.1 - ray.origin.2) * dir_inv.2;

        let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
        let min = if min <= t0 { t0 } else { min };
        let max = if max >= t1 { t1 } else { max };

        min < max
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Sphere {
    pub center: Vector3,
    pub radius: f64,
    pub material: Arc<MaterialEnum>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
        let oc = self.center - ray.origin;
        let h = ray.direction * oc;
        let c = oc * oc - self.radius * self.radius;

        let discriminant = h * h - c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = h - sqrtd;
        if !t_range.contains(&root) {
            root = h + sqrtd;
            if !t_range.contains(&root) {
                return None;
            }
        }
        let point = ray.at(root);
        Some((
            Hit::new(ray, point, (point - self.center) / self.radius, root),
            self.material.clone(),
        ))
    }

    fn aabb(&self) -> Aabb {
        let rvec = Vector3(self.radius, self.radius, self.radius);
        Aabb::new(self.center - rvec, self.center + rvec)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Triangle {
    pub a: Vector3,
    pub b: Vector3,
    pub c: Vector3,
    pub normal: Vector3,
    pub material: Arc<MaterialEnum>,
}

impl Hittable for Triangle {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;

        let ray_cross_e2 = ray.direction / e2;
        let det = e1 * ray_cross_e2;

        if det > -f64::EPSILON && det < f64::EPSILON {
            return None; // This ray is parallel to this triangle.
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.a;
        let u = s * ray_cross_e2 * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let s_cross_e1 = s / e1;
        let v = ray.direction * s_cross_e1 * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let t = e2 * s_cross_e1 * inv_det;

        if t_range.contains(&t) {
            Some((
                Hit::new(ray, ray.at(t), self.normal, t),
                self.material.clone(),
            ))
        } else {
            None
        }
    }

    fn aabb(&self) -> Aabb {
        let mut xmin = self.a.0.min(self.b.0).min(self.c.0);
        let mut ymin = self.a.1.min(self.b.1).min(self.c.1);
        let mut zmin = self.a.2.min(self.b.2).min(self.c.2);
        let mut xmax = self.a.0.max(self.b.0).max(self.c.0);
        let mut ymax = self.a.1.max(self.b.1).max(self.c.1);
        let mut zmax = self.a.2.max(self.b.2).max(self.c.2);
        if xmax - xmin < 0.0001 {
            xmax += 0.0001;
            xmin -= 0.0001;
        }
        if ymax - ymin < 0.0001 {
            ymax += 0.0001;
            ymin -= 0.0001;
        }
        if zmax - zmin < 0.0001 {
            zmax += 0.0001;
            zmin -= 0.0001;
        }
        Aabb::new(Vector3(xmin, ymin, zmin), Vector3(xmax, ymax, zmax))
    }
}

type TriangleMesh = Bvh<Triangle>;

impl TriangleMesh {
    pub fn from_stl_file(
        path: impl AsRef<Path>,
        material: Arc<MaterialEnum>,
    ) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let stl = stl_io::read_stl(&mut file)?;
        let mut triangles = Vec::with_capacity(stl.faces.len());
        for triangle in stl.faces {
            let [a, b, c] = triangle.vertices;
            let a = stl.vertices[a];
            let b = stl.vertices[b];
            let c = stl.vertices[c];
            triangles.push(Triangle {
                a: a.into(),
                b: b.into(),
                c: c.into(),
                normal: Vector3::from(triangle.normal).normalize(),
                material: material.clone(),
            });
        }
        Self::new(&mut triangles).ok_or(std::io::ErrorKind::InvalidData.into())
    }
}

macro_rules! hittables_enum {
    ($outerty:ident: $($ty:ident),+) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd)]
        enum $outerty {
            $($ty($ty)),+
        }

        impl Hittable for $outerty {
            fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
                match &self {
                    $(Self::$ty(val) => val.hit(ray, t_range)),+
                }
            }

            fn aabb(&self) -> Aabb {
                match &self {
                    $(Self::$ty(val) => val.aabb()),+
                }
            }
        }


        $(impl From<$ty> for $outerty {
            fn from(value: $ty) -> $outerty {
                $outerty::$ty(value)
            }
        })+
    };
}

hittables_enum!(HittableEnum: Sphere, TriangleMesh);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Instance {
    pub inner: HittableEnum,
    pub translation: Vector3,
    pub scale: Vector3,
    pub rotation: Quaternion,
}

impl Instance {
    fn transform_pos(&self, value: Vector3) -> Vector3 {
        self.rotation.rotate(value.attune(Vector3(
            1.0 / self.scale.0,
            1.0 / self.scale.1,
            1.0 / self.scale.2,
        ))) + self.translation
    }
    fn transform_dir(&self, value: Vector3) -> Vector3 {
        self.rotation.rotate(value.attune(Vector3(
            1.0 / self.scale.0,
            1.0 / self.scale.1,
            1.0 / self.scale.2,
        )))
    }
    fn transform_ray(&self, value: Ray) -> Ray {
        Ray {
            origin: self.transform_pos(value.origin),
            direction: self.transform_dir(value.direction),
        }
    }
    fn untransform_pos(&self, value: Vector3) -> Vector3 {
        self.rotation
            .conj()
            .rotate(value - self.translation)
            .attune(self.scale)
    }
    fn untransform_dir(&self, value: Vector3) -> Vector3 {
        self.rotation.conj().rotate(value).attune(self.scale)
    }
    fn untransform_ray(&self, value: Ray) -> Ray {
        Ray {
            origin: self.untransform_pos(value.origin),
            direction: self.untransform_dir(value.direction),
        }
    }
}

impl Hittable for Instance {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
        let new_ray = self.transform_ray(ray);
        let new_dir_len = new_ray.direction.len();
        let new_t_range = RangeExclusive(t_range.0 * new_dir_len, t_range.1 * new_dir_len);
        let new_ray = Ray {
            origin: new_ray.origin,
            direction: new_ray.direction.normalize(),
        };
        let (hit, mat) = self.inner.hit(new_ray, new_t_range)?;
        let hit = Hit {
            point: self.untransform_pos(hit.point),
            normal: self.untransform_dir(hit.normal).normalize(),
            t: hit.t / new_dir_len,
            front_face: hit.front_face,
        };
        Some((hit, mat))
    }

    fn aabb(&self) -> Aabb {
        let mut xmin = f64::INFINITY;
        let mut ymin = f64::INFINITY;
        let mut zmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        let mut zmax = f64::NEG_INFINITY;

        let aabb = self.inner.aabb();
        for x in [aabb.0.0, aabb.0.1] {
            for y in [aabb.1.0, aabb.1.1] {
                for z in [aabb.2.0, aabb.2.1] {
                    let transformed = self.untransform_pos(Vector3(x, y, z));
                    xmin = xmin.min(transformed.0);
                    ymin = ymin.min(transformed.1);
                    zmin = zmin.min(transformed.2);
                    xmax = xmax.max(transformed.0);
                    ymax = ymax.max(transformed.1);
                    zmax = zmax.max(transformed.2);
                }
            }
        }
        Aabb::new(Vector3(xmin, ymin, zmin), Vector3(xmax, ymax, zmax))
    }
}

hittables_enum!(MaybeInstance: Sphere, TriangleMesh, Instance);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum Bvh<T: Hittable + Clone> {
    Leaf(T),
    Tree {
        children: Box<(Bvh<T>, Bvh<T>)>,
        aabb: Aabb,
    },
}

impl<T: Hittable + Clone> Hittable for Bvh<T> {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
        self.hit_cached(
            ray,
            t_range,
            Vector3(
                1.0 / ray.direction.0,
                1.0 / ray.direction.1,
                1.0 / ray.direction.2,
            ),
        )
    }

    fn aabb(&self) -> Aabb {
        match self {
            Self::Leaf(entry) => entry.aabb(),
            Self::Tree { aabb, .. } => *aabb,
        }
    }
}

impl<T: Hittable + Clone> Bvh<T> {
    fn new(world: &mut [T]) -> Option<Self> {
        if world.len() == 1 {
            return Some(Self::Leaf(world[0].clone()));
        }
        let aabb = world.iter().map(T::aabb).reduce(Aabb::combine)?;
        let xsize = aabb.0.1 - aabb.0.0;
        let ysize = aabb.1.1 - aabb.1.0;
        let zsize = aabb.2.1 - aabb.2.0;
        let midpoint = if xsize >= ysize && xsize >= zsize {
            world.sort_by(|a, b| {
                (a.aabb().0.0 + a.aabb().0.1).total_cmp(&(b.aabb().0.0 + b.aabb().0.1))
            });
            world.partition_point(|e| (e.aabb().0.0 + e.aabb().0.1) < (aabb.0.0 + aabb.0.1))
        } else if ysize >= xsize && ysize >= zsize {
            world.sort_by(|a, b| {
                (a.aabb().1.0 + a.aabb().1.1).total_cmp(&(b.aabb().1.0 + b.aabb().1.1))
            });
            world.partition_point(|e| (e.aabb().1.0 + e.aabb().1.1) < (aabb.1.0 + aabb.1.1))
        } else {
            world.sort_by(|a, b| {
                (a.aabb().2.0 + a.aabb().2.1).total_cmp(&(b.aabb().2.0 + b.aabb().2.1))
            });
            world.partition_point(|e| (e.aabb().2.0 + e.aabb().2.1) < (aabb.2.0 + aabb.2.1))
        };

        let (left, right) = world.split_at_mut(midpoint.clamp(1, world.len() - 1));
        let left = Self::new(left)?;
        let right = Self::new(right)?;

        Some(Self::Tree {
            children: Box::new((left, right)),
            aabb,
        })
    }

    fn hit_cached(
        &self,
        ray: Ray,
        t_range: RangeExclusive,
        dir_inv: Vector3,
    ) -> Option<(Hit, Arc<MaterialEnum>)> {
        match self {
            Self::Leaf(entry) => entry.hit(ray, t_range),
            Self::Tree { children, aabb } => {
                if aabb.hit_cached(ray, t_range, dir_inv) {
                    let left = children.0.hit_cached(ray, t_range, dir_inv);
                    let t_max = if let Some((hit, _)) = &left {
                        hit.t
                    } else {
                        t_range.1
                    };
                    let right =
                        children
                            .1
                            .hit_cached(ray, RangeExclusive(t_range.0, t_max), dir_inv);
                    right.or(left)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct HitList<T: Hittable>(pub Vec<T>);

impl<T: Hittable> Hittable for HitList<T> {
    fn hit(&self, ray: Ray, t_range: RangeExclusive) -> Option<(Hit, Arc<MaterialEnum>)> {
        self.0
            .iter()
            .filter_map(|e| e.hit(ray, t_range))
            .min_by(|x, y| x.0.t.total_cmp(&y.0.t))
    }

    fn aabb(&self) -> Aabb {
        self.0.iter().map(T::aabb).reduce(Aabb::combine).unwrap()
    }
}

impl<T: Hittable> HitList<T> {
    fn add(&mut self, item: impl Into<T>) {
        self.0.push(item.into());
    }
}

struct Camera {
    pub aspect_ratio: f64,
    pub fov: f64,
    pub defocus_angle: f64,
    pub image_width: u32,
    pub lookfrom: Vector3,
    pub lookat: Vector3,
    pub vup: Vector3,
    pub samples_per_pixel: usize,
    pub max_bounce: usize,
}

impl Camera {
    pub fn render(&self, world: &(impl Hittable + Sync)) -> RgbImage {
        let image_height = ((self.image_width as f64 / self.aspect_ratio) as u32).max(1);

        let focal_length = (self.lookfrom - self.lookat).len();
        let viewport_height = 2.0 * (self.fov / 2.0).tan() * focal_length;
        let viewport_width = viewport_height * (self.image_width as f64 / image_height as f64);

        let w = (self.lookfrom - self.lookat).normalize();
        let u = (self.vup / w).normalize();
        let v = w / u;

        let viewport_u = u * viewport_width;
        let viewport_v = v * -viewport_height;

        let pixel_delta_u = viewport_u / self.image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            self.lookfrom - (w * focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        let defocus_radius = focal_length * (self.defocus_angle / 2.0).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let bar = ProgressBar::new(self.image_width as u64 * image_height as u64);
        RgbImage::from_par_fn(self.image_width, image_height, |x, y| {
            bar.inc(1);
            let mut rng = rng();
            let mut color = Vector3(0.0, 0.0, 0.0);
            for _ in 0..self.samples_per_pixel {
                let origin = if self.defocus_angle <= 0.0 {
                    self.lookfrom
                } else {
                    let (x_defocus, y_defocus) = rng.sample(UniformDisc);
                    self.lookfrom + (defocus_disk_u * x_defocus) + (defocus_disk_v * y_defocus)
                };
                let x_nudge = rng.random::<f64>() - 0.5;
                let y_nudge = rng.random::<f64>() - 0.5;
                let pixel_center = pixel00_loc
                    + (pixel_delta_u * (x as f64 + x_nudge))
                    + (pixel_delta_v * (y as f64 + y_nudge));
                let direction = pixel_center - origin;
                let ray = Ray {
                    origin,
                    direction: direction.normalize(),
                };
                color += Self::color(ray, world, &mut rng, self.max_bounce);
            }
            (color / self.samples_per_pixel as f64).into()
        })
    }

    fn color(
        mut ray: Ray,
        world: &impl Hittable,
        rng: &mut impl Rng,
        max_bounce: usize,
    ) -> Vector3 {
        let mut remaining_bounces = max_bounce;
        let mut attenuation = Vector3(1.0, 1.0, 1.0);
        loop {
            return if remaining_bounces == 0 || attenuation.is_small() {
                Vector3(0.0, 0.0, 0.0)
            } else if let Some((hit, mat)) = world.hit(ray, RangeExclusive(0.001, f64::INFINITY)) {
                let scatter = mat.scatter(ray, hit, rng);
                if let Some(next) = scatter.scattered {
                    ray = next;
                    remaining_bounces -= 1;
                    attenuation = attenuation.attune(scatter.attenuation);
                    continue;
                } else {
                    scatter.attenuation.attune(attenuation)
                }
            } else {
                lerp(
                    Vector3(1.0, 1.0, 1.0),
                    Vector3(0.5, 0.7, 1.0),
                    (ray.direction.1 + 1.0) * 0.5,
                )
                .attune(attenuation)
            };
        }
    }
}

fn main() {
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
        rotation: Quaternion::from_euler(-PI / 2.0, PI / 2.0, 0.0),
        scale: Vector3(0.2, 0.2, 0.2),
    };

    world.add(instance);

    let world = Bvh::new(&mut world.0).unwrap();

    let camera = Camera {
        aspect_ratio: 16. / 9.,
        image_width: 1920,
        fov: 20.0 * PI / 180.0,
        defocus_angle: 0.6 * PI / 180.0,
        lookfrom: Vector3(13.0, 2.0, 3.0),
        lookat: Vector3(0.0, 0.0, 0.0),
        vup: Vector3(0.0, 1.0, 0.0),
        samples_per_pixel: 500,
        max_bounce: 50,
    };

    let image = camera.render(&world);
    image.save("output.png").unwrap();
}

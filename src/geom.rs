use std::ops;

/// A two-dimensional vector.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Constructs a vector with the given coordinates.
    #[inline]
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x: x, y: y }
    }

    /// Computes the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Computes the two-dimensional cross product of two vectors
    /// (which yields a scalar quantity).
    #[inline]
    pub fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Computes the Euclidean distance between two vectros.
    #[inline]
    pub fn distance(self, other: Vec2) -> f32 {
        (other - self).length()
    }

    /// Computes the Euclidean length of a vector.
    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Returns the given vector normalized to unit length.
    #[inline]
    pub fn normalized(self) -> Vec2 {
        (1.0 / self.length()) * self
    }

    /// Linearly interpolates between two vectors by the parameter *t*.
    #[inline]
    pub fn lerp(t: f32, a: Vec2, b: Vec2) -> Vec2 {
        (1.0 - t) * a + t * b
    }

    /// Finds the pointwise min of two vectors.
    #[inline]
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    /// Finds the pointwise max of two vectors.
    #[inline]
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other;
    }
}

impl ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, other: Vec2) {
        *self = *self - other;
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl ops::MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

/// A three-dimensional vector.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Constructs a vector with the given coordinates.
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }

    /// Computes the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Computes the three-dimensional cross product of two vectors.
    #[inline]
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Computes the Euclidean distance between two vectors.
    #[inline]
    pub fn distance(self, other: Vec3) -> f32 {
        (other - self).length()
    }

    /// Computes the Euclidean length of a vector.
    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Returns the given vector normalized to unit length.
    #[inline]
    pub fn normalized(self) -> Vec3 {
        (1.0 / self.length()) * self
    }

    /// Linearly interpolates between two vectors by the parameter *t*.
    #[inline]
    pub fn lerp(t: f32, a: Vec3, b: Vec3) -> Vec3 {
        (1.0 - t) * a + t * b
    }

    /// Finds the pointwise min of two vectors.
    #[inline]
    pub fn min(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    /// Finds the pointwise max of two vectors.
    #[inline]
    pub fn max(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, other: Vec3) {
        *self = *self + other;
    }
}

impl ops::Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, other: Vec3) {
        *self = *self - other;
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl ops::MulAssign<f32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

/// A four-by-four matrix, stored in row-major order.
///
/// Represents a three-dimensional projective transformation.
#[derive(Copy, Clone)]
pub struct Mat4x4(pub [f32; 16]);

impl Mat4x4 {
    /// Constructs the identity matrix.
    #[inline]
    pub fn id() -> Mat4x4 {
        Mat4x4([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    /// Constructs a matrix which scales a three-dimensional vector
    /// by the given factor.
    #[inline]
    pub fn scale(scale: f32) -> Mat4x4 {
        Mat4x4([
            scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    /// Constructs a matrix which translates a three-dimensional vector
    /// by the given amounts in each dimension.
    #[inline]
    pub fn translate(x: f32, y: f32, z: f32) -> Mat4x4 {
        Mat4x4([
            1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    /// Constructs an orthographic projection matrix which maps the given
    /// input ranges to the [-1,1] by [-1,1] by [-1,1] cube.
    ///
    /// Equivalent to the OpenGL function `glOrtho`.
    #[inline]
    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4x4 {
        Mat4x4([
            2.0 / (right - left),
            0.0,
            0.0,
            -(right + left) / (right - left),
            0.0,
            2.0 / (top - bottom),
            0.0,
            -(top + bottom) / (top - bottom),
            0.0,
            0.0,
            -2.0 / (far - near),
            -(far + near) / (far - near),
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// Constructs a perspective projection matrix with the given field of view
    /// angle, aspect ratio, and near and far planes.
    ///
    /// Equivalent to the OpenGL function `gluPerspective`.
    #[inline]
    pub fn perspective(fovy: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4x4 {
        let f = 1.0 / (fovy / 2.0).tan();
        Mat4x4([
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (z_far + z_near) / (z_near - z_far),
            (2.0 * z_far * z_near) / (z_near - z_far),
            0.0,
            0.0,
            -1.0,
            0.0,
        ])
    }

    /// Constructs a viewing transformation matrix from an eye point, a
    /// reference point, and an up vector.
    ///
    /// Equivalent to the OpenGL function `gluLookAt`.
    #[inline]
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4x4 {
        let f = (center - eye).normalized();
        let s = f.cross(up.normalized());
        let u = s.normalized().cross(f);
        Mat4x4([
            s.x, s.y, s.z, 0.0, u.x, u.y, u.z, 0.0, -f.x, -f.y, -f.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    /// Constructs a matrix that rotates by the given angle in the xy-plane.
    #[inline]
    pub fn rotate_xy(angle: f32) -> Mat4x4 {
        Mat4x4([
            angle.cos(),
            -angle.sin(),
            0.0,
            0.0,
            angle.sin(),
            angle.cos(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// Constructs a matrix that rotates by the given angle in the yz-plane.
    #[inline]
    pub fn rotate_yz(angle: f32) -> Mat4x4 {
        Mat4x4([
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            angle.cos(),
            -angle.sin(),
            0.0,
            0.0,
            angle.sin(),
            angle.cos(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// Constructs a matrix that rotates by the given angle in the zx-plane.
    #[inline]
    pub fn rotate_zx(angle: f32) -> Mat4x4 {
        Mat4x4([
            angle.cos(),
            0.0,
            angle.sin(),
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            -angle.sin(),
            0.0,
            angle.cos(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }
}

impl ops::Mul<Mat4x4> for Mat4x4 {
    type Output = Mat4x4;
    #[inline]
    fn mul(self, rhs: Mat4x4) -> Mat4x4 {
        let mut out = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                let mut x = 0.0;
                for k in 0..4 {
                    x += self.0[4 * i + k] * rhs.0[j + 4 * k];
                }
                out[4 * i + j] = x;
            }
        }
        Mat4x4(out)
    }
}

impl ops::MulAssign<Mat4x4> for Mat4x4 {
    #[inline]
    fn mul_assign(&mut self, other: Mat4x4) {
        *self = other * *self;
    }
}

impl ops::Mul<Mat4x4> for f32 {
    type Output = Mat4x4;
    #[inline]
    fn mul(self, rhs: Mat4x4) -> Mat4x4 {
        let mut out = rhs.0;
        for x in out.iter_mut() {
            *x *= self;
        }
        Mat4x4(out)
    }
}

impl ops::Mul<f32> for Mat4x4 {
    type Output = Mat4x4;
    #[inline]
    fn mul(self, rhs: f32) -> Mat4x4 {
        rhs * self
    }
}

impl ops::MulAssign<f32> for Mat4x4 {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

/// Lifts a three-dimensional vector to homogeneous coordinates, applies
/// the transformation, then divides out the *w* factor.
impl ops::Mul<Vec3> for Mat4x4 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.0[0] * rhs.x + self.0[1] * rhs.y + self.0[2] * rhs.z + self.0[3],
            y: self.0[4] * rhs.x + self.0[5] * rhs.y + self.0[6] * rhs.z + self.0[7],
            z: self.0[8] * rhs.x + self.0[9] * rhs.y + self.0[10] * rhs.z + self.0[11],
        } * (1.0 / (self.0[12] * rhs.x + self.0[13] * rhs.y + self.0[14] * rhs.z + self.0[15]))
    }
}

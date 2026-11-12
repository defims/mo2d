//! Vec3 + 4×4 matrix + camera math (f64, right-handed, y-up world).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    pub fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    pub fn scale(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
        a.add(b.sub(a).scale(t))
    }
}

/// Column-major 4×4 matrix (m[col][row]).
#[derive(Debug, Clone, Copy)]
pub struct Mat4(pub [[f64; 4]; 4]);

impl Mat4 {
    pub fn identity() -> Mat4 {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        Mat4(m)
    }

    pub fn mul(&self, o: &Mat4) -> Mat4 {
        let mut r = [[0.0; 4]; 4];
        for c in 0..4 {
            for row in 0..4 {
                let mut s = 0.0;
                for k in 0..4 {
                    s += self.0[k][row] * o.0[c][k];
                }
                r[c][row] = s;
            }
        }
        Mat4(r)
    }

    /// Transform a point (w=1). Returns (xyz, w).
    pub fn project(&self, p: Vec3) -> (Vec3, f64) {
        let x = self.0[0][0] * p.x + self.0[1][0] * p.y + self.0[2][0] * p.z + self.0[3][0];
        let y = self.0[0][1] * p.x + self.0[1][1] * p.y + self.0[2][1] * p.z + self.0[3][1];
        let z = self.0[0][2] * p.x + self.0[1][2] * p.y + self.0[2][2] * p.z + self.0[3][2];
        let w = self.0[0][3] * p.x + self.0[1][3] * p.y + self.0[2][3] * p.z + self.0[3][3];
        (Vec3::new(x, y, z), w)
    }
}

/// Right-handed look-at matrix (camera at `eye`, looking at `target`, `up` hint).
pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let z = eye.sub(target).normalize(); // forward is -z in view space
    let x = up.cross(z).normalize();
    let y = z.cross(x);
    let mut m = [[0.0; 4]; 4];
    m[0] = [x.x, y.x, z.x, 0.0];
    m[1] = [x.y, y.y, z.y, 0.0];
    m[2] = [x.z, y.z, z.z, 0.0];
    m[3] = [-x.dot(eye), -y.dot(eye), -z.dot(eye), 1.0];
    Mat4(m)
}

/// Perspective projection. NDC z ∈ [0,1] (near→0, far→1); smaller depth = closer.
pub fn perspective(fov_y_deg: f64, aspect: f64, near: f64, far: f64) -> Mat4 {
    let f = 1.0 / (fov_y_deg.to_radians() / 2.0).tan();
    let mut m = [[0.0; 4]; 4];
    m[0][0] = f / aspect;
    m[1][1] = f;
    m[2][2] = far / (near - far);
    m[3][2] = far * near / (near - far);
    m[2][3] = -1.0;
    Mat4(m)
}

/// Orthographic projection with NDC z ∈ [0,1].
pub fn ortho(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    m[0][0] = 2.0 / (right - left);
    m[1][1] = 2.0 / (top - bottom);
    m[2][2] = 1.0 / (near - far); // note: maps near→0? see below
    m[3][0] = -(right + left) / (right - left);
    m[3][1] = -(top + bottom) / (top - bottom);
    m[3][2] = near / (near - far);
    m[3][3] = 1.0;
    Mat4(m)
}

impl Vec3 {
    pub fn normalize(self) -> Vec3 {
        let l = self.len();
        if l == 0.0 {
            Vec3::ZERO
        } else {
            self.scale(1.0 / l)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64, eps: f64, what: &str) {
        assert!((a - b).abs() < eps, "{}: {} vs {}", what, a, b);
    }

    #[test]
    fn look_at_direction() {
        let v = look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        let (p, w) = v.project(Vec3::new(0.0, 0.0, 0.0));
        assert_close(p.x, 0.0, 1e-12, "x");
        assert_close(p.y, 0.0, 1e-12, "y");
        assert_close(p.z, -5.0, 1e-12, "z (camera space)");
        assert_close(w, 1.0, 1e-12, "w");
    }

    #[test]
    fn perspective_maps_near_far() {
        let p = perspective(90.0, 1.0, 1.0, 100.0);
        // Point straight ahead at view z = -near → NDC z = 0; at -far → NDC z = 1.
        let (n, w) = p.project(Vec3::new(0.0, 0.0, -1.0));
        assert_close(w, 1.0, 1e-9, "near w");
        assert_close(n.z / w, 0.0, 1e-9, "near ndc z");
        let (f, w) = p.project(Vec3::new(0.0, 0.0, -100.0));
        assert_close(w, 100.0, 1e-9, "far w");
        assert_close(f.z / w, 1.0, 1e-9, "far ndc z");
    }

    #[test]
    fn ortho_maps_corners() {
        let o = ortho(0.0, 100.0, 0.0, 50.0, -10.0, 10.0);
        let (p, w) = o.project(Vec3::new(0.0, 0.0, 0.0));
        assert_close(p.x, -1.0, 1e-12, "left");
        assert_close(p.y, -1.0, 1e-12, "bottom");
        assert_close(w, 1.0, 1e-12, "w");
    }
}

#[derive(Copy, Clone)]
pub struct Vec3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl Vec3 {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }

  pub fn dot(&self, other: Vec3) -> f32 {
    self.x * other.x + self.y * other.y + self.z * other.z
  }

  pub fn cross(&self, other: Vec3) -> Vec3 {
    Vec3::new(
      self.y * other.z - self.z * other.y,
      self.z * other.x - self.x * other.z,
      self.x * other.y - self.y * other.x,
    )
  }

  pub fn length_squared(&self) -> f32 {
    self.dot(*self)
  }

  pub fn length(&self) -> f32 {
    self.length_squared().sqrt()
  }

  pub fn normalize(&self) -> Vec3 {
    *self / self.length()
  }
}

impl std::ops::Add for Vec3 {
  type Output = Vec3;
  fn add(self, rhs: Vec3) -> Vec3 {
    Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
  }
}

impl std::ops::Sub for Vec3 {
  type Output = Vec3;
  fn sub(self, rhs: Vec3) -> Vec3 {
    Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
  }
}

impl std::ops::Mul<f32> for Vec3 {
  type Output = Vec3;
  fn mul(self, rhs: f32) -> Self::Output {
    Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
  }
}

impl std::ops::Div<f32> for Vec3 {
  type Output = Vec3;
  fn div(self, rhs: f32) -> Self::Output {
    Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
  }
}

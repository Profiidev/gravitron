use super::VertexData;

pub fn plane() -> (Vec<VertexData>, Vec<u32>) {
  let tl = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, 0.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.0, 1.0),
  };

  let tr = VertexData {
    position: glam::Vec3::new(1.0, 1.0, 0.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(1.0, 1.0),
  };

  let br = VertexData {
    position: glam::Vec3::new(1.0, -1.0, 0.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(1.0, 0.0),
  };

  let bl = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, 0.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.0, 0.0),
  };

  (vec![tl, tr, br, bl], vec![0, 1, 2, 0, 2, 3])
}

use crate::model::model::VertexData;

pub fn cube() -> (Vec<VertexData>, Vec<u32>) {
  let btl_t = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.0),
  };
  let btl_b = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, -1.0),
    normal: glam::Vec3::new(-1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.25, 1.0),
  };
  let btl_l = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
    uv: glam::Vec2::new(0.0, 0.25),
  };

  let btr_t = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.0),
  };
  let btr_b = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, 1.0),
    normal: glam::Vec3::new(-1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.5, 1.0),
  };
  let btr_r = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
    uv: glam::Vec2::new(0.75, 0.25),
  };

  let bbl_d = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, -1.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.75),
  };
  let bbl_b = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, -1.0),
    normal: glam::Vec3::new(-1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.75),
  };
  let bbl_l = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
    uv: glam::Vec2::new(0.0, 0.5),
  };

  let bbr_d = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, -1.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.75),
  };
  let bbr_b = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, 1.0),
    normal: glam::Vec3::new(-1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.75),
  };
  let bbr_r = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
    uv: glam::Vec2::new(0.75, 0.5),
  };

  let ftl_t = VertexData {
    position: glam::Vec3::new(1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.25),
  };
  let ftl_f = VertexData {
    position: glam::Vec3::new(1.0, 1.0, -1.0),
    normal: glam::Vec3::new(1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.25),
  };
  let ftl_l = VertexData {
    position: glam::Vec3::new(1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
    uv: glam::Vec2::new(0.25, 0.25),
  };

  let ftr_t = VertexData {
    position: glam::Vec3::new(1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 1.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.25),
  };
  let ftr_f = VertexData {
    position: glam::Vec3::new(1.0, 1.0, 1.0),
    normal: glam::Vec3::new(1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.25),
  };
  let ftr_r = VertexData {
    position: glam::Vec3::new(1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
    uv: glam::Vec2::new(0.5, 0.25),
  };

  let fbl_d = VertexData {
    position: glam::Vec3::new(1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, -1.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.5),
  };
  let fbl_f = VertexData {
    position: glam::Vec3::new(1.0, -1.0, -1.0),
    normal: glam::Vec3::new(1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.25, 0.5),
  };
  let fbl_l = VertexData {
    position: glam::Vec3::new(1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
    uv: glam::Vec2::new(0.25, 0.5),
  };

  let fbr_d = VertexData {
    position: glam::Vec3::new(1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, -1.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.5),
  };
  let fbr_f = VertexData {
    position: glam::Vec3::new(1.0, -1.0, 1.0),
    normal: glam::Vec3::new(1.0, 0.0, 0.0),
    uv: glam::Vec2::new(0.5, 0.5),
  };
  let fbr_r = VertexData {
    position: glam::Vec3::new(1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
    uv: glam::Vec2::new(0.5, 0.5),
  };

  (
    vec![
      bbl_d, bbr_d, fbl_d, fbr_d, //bottom
      btl_t, btr_t, ftl_t, ftr_t, //top
      ftl_f, ftr_f, fbl_f, fbr_f, //front
      btl_b, btr_b, bbl_b, bbr_b, //back
      btl_l, bbl_l, fbl_l, ftl_l, //left
      btr_r, bbr_r, ftr_r, fbr_r, //right
    ],
    vec![
      0, 1, 2, 3, 2, 1, //bottom
      4, 6, 5, 5, 6, 7, //top
      8, 10, 9, 9, 10, 11, //front
      12, 13, 14, 15, 14, 13, //back
      16, 17, 18, 19, 16, 18, //left
      20, 22, 21, 21, 22, 23, //right
    ],
  )
}

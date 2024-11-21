use nalgebra_glm::{Vec3, Vec4};
use crate::vertex::Vertex;
use crate::Uniforms;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  let position = Vec4::new(
      vertex.position.x,
      vertex.position.y,
      vertex.position.z,
      1.0
  );

  // Aplicar las matrices de transformación
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

  // Normalizar si 'w' no es 1
  let w = transformed.w;
  let transformed_position = if w != 0.0 {
      Vec3::new(
          transformed.x / w,
          transformed.y / w,
          transformed.z / w
      )
  } else {
      Vec3::new(transformed.x, transformed.y, transformed.z)
  };

  // Transformar la normal usando normal_matrix
  let transformed_normal = uniforms.normal_matrix * Vec4::new(vertex.normal.x, vertex.normal.y, vertex.normal.z, 0.0);
  let transformed_normal = Vec3::new(
      transformed_normal.x,
      transformed_normal.y,
      transformed_normal.z,
  ).normalize();

  Vertex {
      position: vertex.position,
      normal: vertex.normal,
      tex_coords: vertex.tex_coords,
      color: vertex.color,
      transformed_position,
      transformed_normal,
  }
}
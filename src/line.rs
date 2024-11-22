use crate::fragment::Fragment;
use crate::vertex::Vertex;
use nalgebra_glm::{Vec2, Vec3};
use crate::color::Color;
use crate::framebuffer::Framebuffer;


pub fn line(a: &Vertex, b: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let start = a.transformed_position;
    let end = b.transformed_position;

    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = if dx > dy { dx / 2 } else { -dy / 2 };

    let normal_vector = Vec3::new(0.0, 0.0, 1.0);
    let intensity_value = 1.0;

    loop {
        let z = start.z + (end.z - start.z) * (x0 - start.x as i32) as f32 / (end.x - start.x) as f32;
        
        fragments.push(Fragment::new(
            Vec2::new(x0 as f32, y0 as f32),
            z,
            intensity_value,
            start
        ));

        if x0 == x1 && y0 == y1 { break; }

        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x0 += sx;
        }
        if e2 < dy {
            err += dx;
            y0 += sy;
        }
    }

    fragments
}



pub fn draw_line(p1: &Vertex, p2: &Vertex, framebuffer: &mut Framebuffer, color: Color) {
    let x1 = p1.transformed_position.x as isize;
    let y1 = p1.transformed_position.y as isize;
    let x2 = p2.transformed_position.x as isize;
    let y2 = p2.transformed_position.y as isize;

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x1;
    let mut y = y1;

    // Validación de límites del framebuffer
    let width = framebuffer.width as isize;
    let height = framebuffer.height as isize;

    loop {
        if x >= 0 && y >= 0 && x < width && y < height {
            framebuffer.set_current_color(color.to_hex());
            framebuffer.point(x as usize, y as usize, 0.0); // Profundidad arbitraria
        }

        if x == x2 && y == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

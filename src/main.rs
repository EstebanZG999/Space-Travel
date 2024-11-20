use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;


use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::vertex_shader;
use crate::fragment::fragment_shader;
use fastnoise_lite::{FastNoiseLite, NoiseType, CellularDistanceFunction};



pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise_open_simplex: FastNoiseLite,
    noise_cellular: FastNoiseLite,
    noise_perlin: FastNoiseLite,
    noise_value: FastNoiseLite,
    noise_value_cubic: FastNoiseLite,
}



fn create_uniforms() -> Uniforms {
    Uniforms {
        model_matrix: Mat4::identity(),
        view_matrix: Mat4::identity(),
        projection_matrix: Mat4::identity(),
        viewport_matrix: Mat4::identity(),
        time: 0,
        noise_open_simplex: create_open_simplex_noise(),
        noise_cellular: create_cellular_noise(),
        noise_perlin: create_perlin_noise(),
        noise_value: create_value_noise(),
        noise_value_cubic: create_value_cubic_noise(),
    }
}


fn create_cellular_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_distance_function(Some(CellularDistanceFunction::Manhattan));
    noise
}

fn create_open_simplex_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_perlin_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise
}

fn create_value_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Value));
    noise
}

fn create_value_cubic_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::ValueCubic));
    noise
}


fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], shader_type: &str) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            // Llama a fragment_shader para calcular el color final del fragmento
            let shaded_color = fragment_shader(&fragment, uniforms, shader_type);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
    
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}


fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Solar System",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    let mut translation = Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 100.0f32;

    let obj = Obj::load("assets/spheresmooth.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 
    let ring_obj = Obj::load("assets/ring.obj").expect("Failed to load rings.obj");
    let ring_vertex_array = ring_obj.get_vertex_array();


    let mut time = 0;

    // Añadimos las constantes para identificar los cuerpos celestes
    const STAR: u8 = 1;
    const VOLCANIC_PLANET: u8 = 3;
    const CRYSTAL: u8 = 6;
    const VORTEX: u8 = 7;
    const RINGED_PLANET: u8 = 10;
    const ROCKY_PLANET: u8 = 11;
    const EARTH_LIKE_PLANET: u8 = 12;
    // Variable para guardar el cuerpo celeste seleccionado
    let mut selected_object: u8 = STAR;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
        time += 1;

        handle_input(&window, &mut translation, &mut rotation, &mut scale);

        // Cambiamos el objeto seleccionado con teclas
        if window.is_key_down(Key::Key1) {
            selected_object = STAR;
        } else if window.is_key_down(Key::Key2) {
            selected_object = VOLCANIC_PLANET;
        } else if window.is_key_down(Key::Key3) {
            selected_object = CRYSTAL;
        } else if window.is_key_down(Key::Key4) {
            selected_object = VORTEX;
        } else if window.is_key_down(Key::Key5) {
            selected_object = RINGED_PLANET;
        } else if window.is_key_down(Key::Key6) {
            selected_object = ROCKY_PLANET;
        } else if window.is_key_down(Key::Key7) {
            selected_object = EARTH_LIKE_PLANET;
        }
        

        framebuffer.clear();

        let model_matrix = create_model_matrix(translation, scale, rotation);
        let mut uniforms = Uniforms {
            model_matrix,
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            viewport_matrix: Mat4::identity(),
            time,
            noise_open_simplex: create_open_simplex_noise(),
            noise_cellular: create_cellular_noise(),
            noise_perlin: create_perlin_noise(),
            noise_value: create_value_noise(),
            noise_value_cubic: create_value_cubic_noise(),
        };
        


        // Renderizamos el objeto seleccionado con shaders específicos
        match selected_object {
            STAR => {
                framebuffer.set_current_color(0xFFDDDD);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "solar_surface");
            },
            VOLCANIC_PLANET => {
                framebuffer.set_current_color(0xFF4500);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "volcanic_planet_shader");
            },
            CRYSTAL => {
                framebuffer.set_current_color(0x0000FF);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "crystal_planet_shader");
            },
            VORTEX => {
                framebuffer.set_current_color(0xAAAAAA);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "vortex");
            },
            RINGED_PLANET => {
                framebuffer.set_current_color(0x00FFAA);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "ringed_planet");

                // Renderizar los anillos del gigante gaseoso
                let ring_model_matrix = create_model_matrix(translation, scale * 1.2, rotation);
                uniforms.model_matrix = ring_model_matrix;
                render(&mut framebuffer, &uniforms, &ring_vertex_array, "ring_shader");
            },
            ROCKY_PLANET => {
                framebuffer.set_current_color(0xAAAAAA);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "rocky_planet");
            
                // Generar la luna del planeta rocoso
                let moon_orbit_radius = 200.0; // Radio de la órbita de la luna, ajustable
                let moon_scale = scale * 0.3; // Tamaño de la luna en relación al planeta
            
                let moon_translation = translation + Vec3::new(
                    moon_orbit_radius * (time as f32 * 0.05).cos(),
                    moon_orbit_radius * (time as f32 * 0.05).sin(),
                    0.0,
                );
            
                let moon_model_matrix = create_model_matrix(moon_translation, moon_scale, rotation);
                uniforms.model_matrix = moon_model_matrix;
                render(&mut framebuffer, &uniforms, &vertex_arrays, "moon_shader");
            },
            EARTH_LIKE_PLANET => {
                framebuffer.set_current_color(0xFFFFFF);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "earth_like_planet_shader");
            },
            _ => {},
        }


        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, translation: &mut Vec3, rotation: &mut Vec3, scale: &mut f32) {
    let move_speed = 10.0; 
    let rotation_speed = 0.2; 
    let zoom_speed = 2.0; 

    // Movimiento de cámara
    if window.is_key_down(Key::Left) {
        translation.x -= move_speed; 
    }
    if window.is_key_down(Key::Right) {
        translation.x += move_speed; 
    }
    if window.is_key_down(Key::Up) {
        translation.y -= move_speed; 
    }
    if window.is_key_down(Key::Down) {
        translation.y += move_speed; 
    }

    // Control de rotación 
    if window.is_key_down(Key::A) {
        rotation.y += rotation_speed; 
    }
    if window.is_key_down(Key::D) {
        rotation.y -= rotation_speed; 
    }
    if window.is_key_down(Key::W) {
        rotation.x += rotation_speed; 
    }
    if window.is_key_down(Key::S) {
        rotation.x -= rotation_speed; 
    }

    // Zoom
    if window.is_key_down(Key::Q) {
        *scale += zoom_speed;  
    }
    if window.is_key_down(Key::E) {
        *scale -= zoom_speed;  
    }
}
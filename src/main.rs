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
use std::clone::Clone;
use nalgebra_glm as glm;


pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    normal_matrix: Mat4, // Campo agregado
    time: u32,
    noise_open_simplex: FastNoiseLite,
    noise_cellular: FastNoiseLite,
    noise_perlin: FastNoiseLite,
    noise_value: FastNoiseLite,
    noise_value_cubic: FastNoiseLite,
}

impl Clone for Uniforms {
    fn clone(&self) -> Self {
        Uniforms {
            model_matrix: self.model_matrix,
            view_matrix: self.view_matrix,
            projection_matrix: self.projection_matrix,
            viewport_matrix: self.viewport_matrix,
            normal_matrix: self.normal_matrix, // Incluir este campo
            time: self.time,
            noise_open_simplex: create_open_simplex_noise(),
            noise_cellular: create_cellular_noise(),
            noise_perlin: create_perlin_noise(),
            noise_value: create_value_noise(),
            noise_value_cubic: create_value_cubic_noise(),
            // clonar otros campos...
        }
    }
}

fn create_uniforms() -> Uniforms {
    Uniforms {
        model_matrix: Mat4::identity(),
        view_matrix: Mat4::identity(),
        projection_matrix: Mat4::identity(),
        viewport_matrix: Mat4::identity(),
        normal_matrix: Mat4::identity(),
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
    // Crear matrices de rotación alrededor de cada eje
    let rotation_matrix_x = glm::rotation(rotation.x, &Vec3::x_axis());
    let rotation_matrix_y = glm::rotation(rotation.y, &Vec3::y_axis());
    let rotation_matrix_z = glm::rotation(rotation.z, &Vec3::z_axis());

    // Combinar las matrices de rotación
    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    // Crear la matriz de escala
    let scaling_matrix = glm::scaling(&Vec3::new(scale, scale, scale));

    // Crear la matriz de traslación
    let translation_matrix = glm::translation(&translation);

    // Combinar las transformaciones: primero escala, luego rotación y finalmente traslación
    let model_matrix = translation_matrix * rotation_matrix * scaling_matrix;

    model_matrix
}

fn create_view_matrix(translation: Vec3, rotation: Vec3, scale: f32) -> Mat4 {
    // Invertimos las transformaciones para la cámara
    let translation_matrix = glm::translation(&-translation);
    let scaling_matrix = glm::scaling(&Vec3::new(1.0 / scale, 1.0 / scale, 1.0 / scale));

    let rotation_matrix_x = glm::rotation(-rotation.x, &Vec3::x_axis());
    let rotation_matrix_y = glm::rotation(-rotation.y, &Vec3::y_axis());
    let rotation_matrix_z = glm::rotation(-rotation.z, &Vec3::z_axis());

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    // El orden es importante: primero escala, luego rotación, luego traslación
    let view_matrix = scaling_matrix * rotation_matrix * translation_matrix;

    view_matrix
}


fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], shader_type: &str) {
    // Transformar vértices usando el vertex shader
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Triangulación de los vértices
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

    // Rasterización de los triángulos
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Aplicar el fragment shader a cada fragmento
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = fragment_shader(&fragment, uniforms, shader_type);
            framebuffer.set_current_color(shaded_color.to_hex());
            framebuffer.point(x, y, fragment.depth);
        }
    }
}


fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

pub struct Planet {
    name: &'static str,
    scale: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    shader: &'static str,
}


fn main() {
    // Define el arreglo de planetas dentro de `main`
    let planets = vec![
        Planet {
            name: "Mercury",
            scale: 0.3,
            orbit_radius: 100.0,
            orbit_speed: 0.02,
            rotation_speed: 0.1,
            shader: "molten_core_planet_shader",
        },
        Planet {
            name: "Venus",
            scale: 0.5,
            orbit_radius: 150.0,
            orbit_speed: 0.015,
            rotation_speed: 0.09,
            shader: "volcanic_planet_shader",
        },
        Planet {
            name: "Earth",
            scale: 0.7,
            orbit_radius: 200.0,
            orbit_speed: 0.01,
            rotation_speed: 0.08,
            shader: "earth_like_planet_shader",
        },
        Planet {
            name: "Mars",
            scale: 0.4,
            orbit_radius: 250.0,
            orbit_speed: 0.008,
            rotation_speed: 0.07,
            shader: "rocky_planet",
        },
        Planet {
            name: "Jupiter",
            scale: 1.5,
            orbit_radius: 300.0,
            orbit_speed: 0.005,
            rotation_speed: 0.06,
            shader: "gas_giant_shader",
        },
        Planet {
            name: "Saturn",
            scale: 1.2,
            orbit_radius: 350.0,
            orbit_speed: 0.004,
            rotation_speed: 0.05,
            shader: "ringed_planet",
        },
        Planet {
            name: "Uranus",
            scale: 1.0,
            orbit_radius: 400.0,
            orbit_speed: 0.003,
            rotation_speed: 0.04,
            shader: "crystal_planet_shader",
        },
    ];
    

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

    // Definir las variables de la cámara al inicio de `main`
    let mut camera_translation = Vec3::new(0.0, 0.0, 0.0);
    let mut camera_rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut camera_scale = 1.0f32;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
        time += 1;

        handle_input(&window, &mut camera_translation, &mut camera_rotation, &mut camera_scale);

        // Depuración de valores de la cámara
        println!("Camera Translation: {:?}", camera_translation);
        println!("Camera Rotation: {:?}", camera_rotation);
        println!("Camera Scale: {:?}", camera_scale);

        let view_matrix = create_view_matrix(camera_translation, camera_rotation, camera_scale);


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

        // Renderizar el Sol
        let sun_translation =
            Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
        let sun_rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05); // Velocidad de rotación del Sol
        let sun_scale = 50.0; // Ajusta este valor según sea necesario

        let sun_model_matrix = create_model_matrix(sun_translation, sun_scale, sun_rotation);
        let normal_matrix = sun_model_matrix.try_inverse().unwrap().transpose();


        let sun_uniforms = Uniforms {
            normal_matrix,
            model_matrix: sun_model_matrix,
            view_matrix: view_matrix,
            projection_matrix: Mat4::identity(),
            viewport_matrix: Mat4::identity(),
            time,
            noise_open_simplex: create_open_simplex_noise(),
            noise_cellular: create_cellular_noise(),
            noise_perlin: create_perlin_noise(),
            noise_value: create_value_noise(),
            noise_value_cubic: create_value_cubic_noise(),
        };

        render(
            &mut framebuffer,
            &sun_uniforms,
            &vertex_arrays,
            "solar_surface",
        );

        // Renderizar los planetas
        for planet in &planets {
            let angle = time as f32 * planet.orbit_speed;
            let orbit_x = (planet.orbit_radius * angle.cos()) + (window_width as f32 / 2.0);
            let orbit_y = (planet.orbit_radius * angle.sin()) + (window_height as f32 / 2.0);

            let model_matrix = create_model_matrix(
                Vec3::new(orbit_x, orbit_y, 0.0),
                planet.scale * 10.0, // Ajusta la escala si es necesario
                Vec3::new(0.0, 0.0, time as f32 * planet.rotation_speed),
            );

            let normal_matrix = model_matrix.try_inverse().unwrap().transpose();
            let planet_uniforms = Uniforms {
                normal_matrix,
                model_matrix,
                view_matrix: view_matrix,
                projection_matrix: Mat4::identity(),
                viewport_matrix: Mat4::identity(),
                time,
                noise_open_simplex: create_open_simplex_noise(),
                noise_cellular: create_cellular_noise(),
                noise_perlin: create_perlin_noise(),
                noise_value: create_value_noise(),
                noise_value_cubic: create_value_cubic_noise(),
            };

            render(
                &mut framebuffer,
                &planet_uniforms,
                &vertex_arrays,
                planet.shader,
            );
        }

        // Renderizar el objeto seleccionado con shaders específicos
        match selected_object {
            STAR => {
                // Ya renderizamos el Sol, podrías agregar efectos adicionales si lo deseas
            }
            VOLCANIC_PLANET => {
                let translation = Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05);
                let scale = 30.0;
                let model_matrix = create_model_matrix(translation, scale, rotation);

                let uniforms = Uniforms {
                    model_matrix,
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0xFF4500);
                render(
                    &mut framebuffer,
                    &uniforms,
                    &vertex_arrays,
                    "volcanic_planet_shader",
                );
            }
            CRYSTAL => {
                let translation = Vec3::new(
                    window_width as f32 / 2.0,
                    window_height as f32 / 2.0,
                    0.0,
                );
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05);
                let scale = 30.0;
                let model_matrix = create_model_matrix(translation, scale, rotation);

                let uniforms = Uniforms {
                    model_matrix,
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0x00FFFF);
                render(
                    &mut framebuffer,
                    &uniforms,
                    &vertex_arrays,
                    "crystal_planet_shader",
                );
            }
            VORTEX => {
                let translation = Vec3::new(
                    window_width as f32 / 2.0,
                    window_height as f32 / 2.0,
                    0.0,
                );
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.1);
                let scale = 35.0;
                let model_matrix = create_model_matrix(translation, scale, rotation);

                let uniforms = Uniforms {
                    model_matrix,
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0xFF00FF);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "vortex_shader");
            }
            RINGED_PLANET => {
                let translation = Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05);
                let scale = 40.0;

                let model_matrix = create_model_matrix(translation, scale, rotation);

                let mut uniforms = Uniforms {
                    model_matrix,
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0x00FFAA);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "ringed_planet");

                // Renderizar los anillos del gigante gaseoso
                let ring_model_matrix = create_model_matrix(translation, scale * 1.2, rotation);
                uniforms.model_matrix = ring_model_matrix;
                render(&mut framebuffer, &uniforms, &ring_vertex_array, "ring_shader");
            }
            ROCKY_PLANET => {
                let translation = Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05);
                let scale = 25.0;

                let mut uniforms = Uniforms {
                    model_matrix: create_model_matrix(translation, scale, rotation),
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0xAAAAAA);
                render(&mut framebuffer, &uniforms, &vertex_arrays, "rocky_planet");

                // Generar la luna del planeta rocoso
                let moon_orbit_radius = 50.0; // Radio de la órbita de la luna, ajustable
                let moon_scale = scale * 0.3; // Tamaño de la luna en relación al planeta

                let moon_translation = translation
                    + Vec3::new(
                        moon_orbit_radius * (time as f32 * 0.05).cos(),
                        moon_orbit_radius * (time as f32 * 0.05).sin(),
                        0.0,
                    );

                let moon_model_matrix = create_model_matrix(moon_translation, moon_scale, rotation);
                uniforms.model_matrix = moon_model_matrix;
                render(&mut framebuffer, &uniforms, &vertex_arrays, "moon_shader");
            }
            EARTH_LIKE_PLANET => {
                let translation = Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0);
                let rotation = Vec3::new(0.0, 0.0, time as f32 * 0.05);
                let scale = 35.0;

                let uniforms = Uniforms {
                    model_matrix: create_model_matrix(translation, scale, rotation),
                    view_matrix: view_matrix,
                    normal_matrix,
                    projection_matrix: Mat4::identity(),
                    viewport_matrix: Mat4::identity(),
                    time,
                    noise_open_simplex: create_open_simplex_noise(),
                    noise_cellular: create_cellular_noise(),
                    noise_perlin: create_perlin_noise(),
                    noise_value: create_value_noise(),
                    noise_value_cubic: create_value_cubic_noise(),
                };

                framebuffer.set_current_color(0xFFFFFF);
                render(
                    &mut framebuffer,
                    &uniforms,
                    &vertex_arrays,
                    "earth_like_planet_shader",
                );
            }
            _ => {}
        }

        // Actualizar la ventana una sola vez
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}


fn handle_input(window: &Window, translation: &mut Vec3, rotation: &mut Vec3, scale: &mut f32) {
    let move_speed = 5.0; // Reducir la velocidad de movimiento para un control más preciso
    let rotation_speed = 0.05; // Reducir la velocidad de rotación
    let zoom_speed = 0.05; // Reducir la velocidad de zoom

    // Movimiento de cámara
    if window.is_key_down(Key::Left) {
        translation.x -= move_speed;
    }
    if window.is_key_down(Key::Right) {
        translation.x += move_speed;
    }
    if window.is_key_down(Key::Up) {
        translation.y += move_speed; // Cambiar el signo para corregir la dirección
    }
    if window.is_key_down(Key::Down) {
        translation.y -= move_speed; // Cambiar el signo para corregir la dirección
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
        *scale *= 1.0 + zoom_speed; // Ajustar zoom multiplicativamente
    }
    if window.is_key_down(Key::E) {
        *scale *= 1.0 - zoom_speed; // Ajustar zoom multiplicativamente
        if *scale < 0.1 {
            *scale = 0.1; // Prevenir escala negativa o muy pequeña
        }
    }
}

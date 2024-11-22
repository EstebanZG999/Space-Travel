use nalgebra_glm::{Vec2, Vec3, Mat4};
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
mod skybox;



use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::vertex_shader;
use color::Color;
use crate::fragment::fragment_shader;
use fastnoise_lite::{FastNoiseLite, NoiseType, CellularDistanceFunction};
use std::clone::Clone;
use nalgebra_glm as glm;
use crate::line::draw_line;



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
            normal_matrix: self.normal_matrix, 
            time: self.time,
            noise_open_simplex: create_open_simplex_noise(),
            noise_cellular: create_cellular_noise(),
            noise_perlin: create_perlin_noise(),
            noise_value: create_value_noise(),
            noise_value_cubic: create_value_cubic_noise(),
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



fn create_orbit_points(center: Vec3, radius: f32, segments: usize) -> Vec<Vertex> {
    let mut points = Vec::new();
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        points.push(Vertex {
            position: Vec3::new(x, y, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            tex_coords: Vec2::new(0.0, 0.0), // Cambiado a Vec2
            color: Color::new(255, 255, 255), // Color blanco
            transformed_position: Vec3::zeros(),
            transformed_normal: Vec3::zeros(),
        });
    }
    points
}

fn render_orbit(
    framebuffer: &mut Framebuffer,
    points: &[Vertex],
    color: Color,
) {
    for i in 0..points.len() {
        let p1 = &points[i];
        let p2 = &points[(i + 1) % points.len()]; // Conectar el último punto con el primero
        draw_line(p1, p2, framebuffer, color);
    }
}



pub struct Planet {
    name: &'static str,
    scale: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    shader: &'static str,
    ring_shader: Option<&'static str>, // Shader para el anillo
    ring_scale: Option<f32>,          // Escala del anillo
    moon_shader: Option<&'static str>, // Shader para la luna
    moon_scale: Option<f32>,          // Escala de la luna
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
            ring_shader: None,
            ring_scale: None,
            moon_shader: None,
            moon_scale: None,
        },
        Planet {
            name: "Venus",
            scale: 0.5,
            orbit_radius: 150.0,
            orbit_speed: 0.015,
            rotation_speed: 0.09,
            shader: "volcanic_planet_shader",
            ring_shader: None,
            ring_scale: None,
            moon_shader: None,
            moon_scale: None,
        },
        Planet {
            name: "Earth",
            scale: 0.7,
            orbit_radius: 200.0,
            orbit_speed: 0.01,
            rotation_speed: 0.08,
            shader: "earth_like_planet_shader",
            ring_shader: None,
            ring_scale: None,
            moon_shader: Some("moon_shader"),
            moon_scale: Some(0.3),
        },
        Planet {
            name: "Mars",
            scale: 0.4,
            orbit_radius: 250.0,
            orbit_speed: 0.008,
            rotation_speed: 0.07,
            shader: "rocky_planet",
            ring_shader: None,
            ring_scale: None,
            moon_shader: None,
            moon_scale: None,
        },
        Planet {
            name: "Jupiter",
            scale: 1.5,
            orbit_radius: 300.0,
            orbit_speed: 0.005,
            rotation_speed: 0.06,
            shader: "gas_giant_shader",
            ring_shader: None,
            ring_scale: None,
            moon_shader: None,
            moon_scale: None,
        },
        Planet {
            name: "Saturn",
            scale: 1.2,
            orbit_radius: 350.0,
            orbit_speed: 0.004,
            rotation_speed: 0.05,
            shader: "ringed_planet",
            ring_shader: Some("ring_shader"),
            ring_scale: Some(1.3),
            moon_shader: Some("moon_shader"),
            moon_scale: Some(0.4),
        },
        Planet {
            name: "Uranus",
            scale: 1.0,
            orbit_radius: 400.0,
            orbit_speed: 0.003,
            rotation_speed: 0.04,
            shader: "crystal_planet_shader",
            ring_shader: None,
            ring_scale: None,
            moon_shader: None,
            moon_scale: None,
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

    //SKYBOX
    let skybox = skybox::Skybox::new(10000); // Ajusta el número de estrellas


    let obj = Obj::load("assets/spheresmooth.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let ring_obj = Obj::load("assets/ring.obj").expect("Failed to load rings.obj");
    let ring_vertex_array = ring_obj.get_vertex_array();
    let moon_obj = Obj::load("assets/moon.obj").expect("Failed to load moon.obj");
    let moon_vertex_array = moon_obj.get_vertex_array();

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
    let mut camera_translation = Vec3::new(0.0, 0.0, -500.0); // Aleja la cámara para ver el skybox
    let mut camera_rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut camera_scale = 1.0f32;


    //Orbitas
    let orbit_segments = 60; // Número de segmentos para suavizar las líneas
    let orbits: Vec<Vec<Vertex>> = planets
        .iter()
        .map(|planet| create_orbit_points(
            Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0),
            planet.orbit_radius,
            orbit_segments,
        ))
        .collect();


    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
        time += 1;

        framebuffer.clear();

        handle_input(&window, &mut camera_translation, &mut camera_rotation, &mut camera_scale);

        // Depuración de valores de la cámara
        println!("Camera Translation: {:?}", camera_translation);
        println!("Camera Rotation: {:?}", camera_rotation);
        println!("Camera Scale: {:?}", camera_scale);

        let view_matrix = create_view_matrix(camera_translation, camera_rotation, camera_scale);

        // Crear uniforms para el Skybox
        let skybox_uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix,
            projection_matrix: glm::perspective(
                framebuffer_width as f32 / framebuffer_height as f32, // Relación de aspecto
                45.0_f32.to_radians(),                               // Campo de visión
                0.1,                                                 // Plano cercano
                2000.0,                                              // Plano lejano ajustado
            ),
            viewport_matrix: glm::scaling(&Vec3::new(
                framebuffer_width as f32 / 2.0,
                framebuffer_height as f32 / 2.0,
                1.0,
            )),
            normal_matrix: Mat4::identity(),
            time,
            noise_open_simplex: create_open_simplex_noise(),
            noise_cellular: create_cellular_noise(),
            noise_perlin: create_perlin_noise(),
            noise_value: create_value_noise(),
            noise_value_cubic: create_value_cubic_noise(),
        };
        

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
        

        // Renderizar el Skybox
        skybox.render(&mut framebuffer, &skybox_uniforms, camera_translation);

        for orbit_points in &orbits {
            let orbit_model_matrix = Mat4::identity(); // Las órbitas no rotan ni se escalan
            let orbit_uniforms = Uniforms {
                model_matrix: orbit_model_matrix,
                view_matrix: view_matrix,
                projection_matrix: Mat4::identity(),
                viewport_matrix: Mat4::identity(),
                normal_matrix: orbit_model_matrix.try_inverse().unwrap().transpose(),
                time,
                noise_open_simplex: create_open_simplex_noise(),
                noise_cellular: create_cellular_noise(),
                noise_perlin: create_perlin_noise(),
                noise_value: create_value_noise(),
                noise_value_cubic: create_value_cubic_noise(),
            };
        
            render(&mut framebuffer, &orbit_uniforms, &orbit_points, "orbit_shader");
        }

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
            let orbit_points = create_orbit_points(
                Vec3::new(window_width as f32 / 2.0, window_height as f32 / 2.0, 0.0),
                planet.orbit_radius,
                100, // Número de segmentos para una órbita más suave
            );

            render_orbit(&mut framebuffer, &orbit_points, Color::new(255, 255, 255)); // Blanco


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
                planet.shader
            );

            if let (Some(ring_shader), Some(ring_scale)) = (planet.ring_shader, planet.ring_scale) {
                let ring_model_matrix = create_model_matrix(
                    Vec3::new(orbit_x, orbit_y, 0.0), // Asegúrate de que está en la posición del planeta
                    ring_scale * 10.0,               // Escala ajustada para que el anillo sea visible
                    Vec3::new(0.0, 0.0, 0.0),        // Anillos normalmente no rotan en este contexto
                );
                
            
                let ring_normal_matrix = ring_model_matrix.try_inverse().unwrap().transpose();
            
                let ring_uniforms = Uniforms {
                    normal_matrix: ring_normal_matrix,
                    model_matrix: ring_model_matrix,
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
            
                render(&mut framebuffer, &ring_uniforms, &ring_vertex_array, ring_shader);
            }

            if let (Some(moon_shader), Some(moon_scale)) = (planet.moon_shader, planet.moon_scale) {
                let moon_orbit_radius = planet.scale * 100.0; // Relación con el tamaño del planeta
                let moon_angle = time as f32 * 0.01;         // Ajusta la velocidad angular
                let moon_x = orbit_x + moon_orbit_radius * moon_angle.cos();
                let moon_y = orbit_y + moon_orbit_radius * moon_angle.sin();
                
            
                let moon_model_matrix = create_model_matrix(
                    Vec3::new(moon_x, moon_y, 0.0),
                    moon_scale * 10.0, // Ajusta la escala manualmente si no es visible
                    Vec3::new(0.0, 0.0, 0.0),
                );
            
                let moon_normal_matrix = moon_model_matrix.try_inverse().unwrap().transpose();
            
                let moon_uniforms = Uniforms {
                    normal_matrix: moon_normal_matrix,
                    model_matrix: moon_model_matrix,
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
            
                render(&mut framebuffer, &moon_uniforms, &moon_vertex_array, moon_shader);
            }
            
            
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

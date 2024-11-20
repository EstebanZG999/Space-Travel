use nalgebra_glm::{Vec2, Vec3};
use crate::color::Color;
use crate::Uniforms;
use fastnoise_lite::FastNoiseLite;


pub struct Fragment {
    pub position: Vec2,
    pub depth: f32,
    pub intensity: f32,
    pub vertex_position: Vec3,
}

impl Fragment {
    pub fn new(position: Vec2, depth: f32,intensity: f32, vertex_position: Vec3) -> Self {
        Fragment {
            position,
            depth,
            intensity,
            vertex_position
        }
    }
}

// Shaders para planetas
fn solar_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time_factor = (uniforms.time as f32 * 0.05).sin() * 0.4 + 0.8; 

    let zoom = 15.0;
    let noise_value = uniforms.noise_open_simplex.get_noise_2d(x * zoom, y * zoom) * 0.3 + 0.7;
    let surface_intensity = (0.9 + noise_value * 0.1) * time_factor; 

    // Color base con variaciones para simular la superficie solar
    let r = (255.0 * surface_intensity) as u8;
    let g = (200.0 * surface_intensity) as u8;
    let b = (50.0 * surface_intensity) as u8;

    let core_color = Color::new(r, g, b) * fragment.intensity;

    // Efecto de halo alrededor del Sol
    let distance_to_center = (x.powi(2) + y.powi(2)).sqrt();
    let halo_threshold = 0.0;
    let halo_intensity = if distance_to_center > halo_threshold {
        ((distance_to_center - halo_threshold) * 3.0).exp().min(1.0)
    } else {
        0.0
    };

    let halo_color = Color::new(255, 140, 0) * halo_intensity;

    // Iluminación ambiental para que toda la esfera tenga visibilidad mínima
    let ambient_intensity = 0.1;
    let ambient_color = Color::new(255, 100, 50) * ambient_intensity;

    // Mezcla del color de la superficie, halo, y luz ambiental
    core_color.blend_add(&halo_color).blend_add(&ambient_color)
}


fn volcanic_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let zoom = 600.0;

    // Usar ruido Value Cubic
    let noise_value = uniforms.noise_value_cubic.get_noise_2d(x * zoom, y * zoom);
    let normalized_noise = ((noise_value + 1.0) * 0.5).clamp(0.0, 1.0);

    // Definir colores para el patrón volcánico
    let color_roca = Color::new(139, 69, 19);        
    let color_sombra = Color::new(105, 60, 45);      
    let color_mineral = Color::new(189, 183, 107);   

    // Interpolación del patrón rocoso y simulación de grietas
    let color_intermediate = color_roca.lerp(&color_sombra, normalized_noise * 0.8);
    let base_color = color_intermediate.lerp(&color_mineral, normalized_noise * 0.5);

    // Color final con iluminación ambiental
    let ambient_intensity = 0.3;
    let ambient_color = Color::new(40, 20, 20);
    base_color * fragment.intensity + ambient_color * ambient_intensity
}



fn molten_core_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 10.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.05;

    // Patrón para la lava usando ruido en movimiento
    let lava_pattern = ((x * zoom + time).sin() * (y * zoom + time).cos()).abs();
    let noise_value = uniforms.noise_cellular.get_noise_2d(x * zoom, y * zoom);

    // Colores base para el núcleo y la lava
    let lava_color = Color::new(255, (80.0 * lava_pattern) as u8, 0);  
    let rock_color = Color::new((50.0 * (1.0 - noise_value)) as u8, 0, 0);  

    // Mezcla de colores entre el núcleo de lava y las áreas de roca
    if noise_value > 0.3 {
        lava_color.blend_add(&rock_color)
    } else {
        rock_color.blend_multiply(&lava_color)
    }
}

fn crystal_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 20.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.03;

    // Genera un patrón de cristales usando ruido
    let crystal_pattern = uniforms.noise_cellular.get_noise_2d(x * zoom, y * zoom).abs();
    let angle_variation = ((x * 0.5 + y * 0.5 + time).sin().abs() + 0.5) * 0.5;

    // Colores base para los cristales, variando en tonos de azul y púrpura
    let base_color = Color::new(
        (120.0 * angle_variation) as u8,
        (160.0 * crystal_pattern) as u8,
        (255.0 * angle_variation) as u8,
    );

    // Color de borde brillante para los cristales
    let highlight_color = Color::new(255, 255, 255) * (0.3 + angle_variation * 0.7);

    // Mezcla el color base con el brillo de los cristales
    base_color.blend_add(&highlight_color)
}

fn vortex_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.03;

    // Convertir coordenadas cartesianas a polares para crear el efecto de vórtice
    let radius = (x * x + y * y).sqrt();
    let angle = y.atan2(x) + time;  // Añadimos tiempo para rotación

    // Parámetros para ajustar el efecto de vórtice y el ruido
    let vortex_zoom = 10.0;
    let noise_zoom = 20.0;

    // Crear el patrón de vórtice en espiral
    let vortex_pattern = ((angle * vortex_zoom).sin() * (radius * vortex_zoom).cos()).abs();

    // Puedes elegir el tipo de ruido que prefieras; aquí usaremos ruido Perlin
    let noise_value = uniforms.noise_perlin.get_noise_2d(x * noise_zoom, y * noise_zoom);
    let normalized_noise = ((noise_value + 1.0) * 0.5).clamp(0.0, 1.0);

    // Combinar el patrón de vórtice con el ruido para añadir detalles
    let combined_pattern = (vortex_pattern * 0.7 + normalized_noise * 0.3).clamp(0.0, 1.0);

    // Definir colores base y aplicar variaciones según el patrón combinado
    let base_color = Color::new(
        (50.0 + 205.0 * combined_pattern) as u8,   
        (50.0 + 155.0 * (1.0 - combined_pattern)) as u8, 
        (150.0 + 105.0 * combined_pattern) as u8,  
    );

    // Añadir efectos de iluminación o brillo según el patrón
    let highlight_intensity = (normalized_noise * 0.5 + 0.5).clamp(0.0, 1.0);
    let highlight_color = Color::new(
        (255.0 * highlight_intensity) as u8,
        (200.0 * highlight_intensity) as u8,
        (150.0 * highlight_intensity) as u8,
    );

    // Mezclar el color base con el color de brillo para resaltar áreas del vórtice
    let final_color = base_color.blend_add(&highlight_color) * fragment.intensity;

    final_color
}



fn ringed_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 4.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.03;

    // Crear un patrón de bandas horizontales para simular el aspecto de Júpiter
    let band_pattern = ((y * zoom).sin() * (y * zoom * 0.5 + time).cos()).abs();
    let secondary_pattern = ((y * zoom * 0.8 - time).cos() * (y * zoom * 0.3 + time).sin()).abs();
    
    // Mezcla de patrones para obtener variaciones en las bandas
    let combined_pattern = (band_pattern * 0.7 + secondary_pattern * 0.3).min(1.0);

    // Colores base en tonos cálidos para simular el color de Júpiter
    let r = (combined_pattern * 230.0) as u8;      
    let g = ((1.0 - combined_pattern) * 150.0) as u8; 
    let b = ((0.5 - combined_pattern) * 50.0) as u8 + 20; 

    let base_color = Color::new(r, g, b);

    // Ajuste de iluminación ambiental para darle suavidad al efecto
    let ambient_intensity = 0.4;
    let ambient_color = Color::new(100, 50, 30); 

    // Mezcla del color base y el color ambiental
    base_color * fragment.intensity + ambient_color * ambient_intensity
}



pub fn ring_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color1 = Color::new(220, 200, 180);  
    let color2 = Color::new(150, 100, 70);   
    let color3 = Color::new(50, 30, 20);     

    let position = Vec3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.depth,
    );

    let zoom = 10.0;
    let noise_zoom = 50.0;

    let line_pattern = (position.y * zoom).sin().abs();

    let ruido = uniforms.noise_open_simplex.get_noise_3d(
        position.x * noise_zoom,
        position.y * noise_zoom,
        position.z * noise_zoom,
    );

    let val_normalizado = (line_pattern * 0.7 + ruido * 0.3).clamp(0.0, 1.0);

    let color_intermediate = color1.lerp(&color2, val_normalizado);
    let final_color = color_intermediate.lerp(&color3, val_normalizado);

    final_color * 0.9
}

pub fn rocky_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let color_roca = Color::new(139, 69, 19);        
    let color_sombra = Color::new(105, 60, 45);      
    let color_mineral = Color::new(189, 183, 107);   

    // Ajuste de la frecuencia para el patrón de mosaico
    let zoom = 1000.0; 
    let x = fragment.vertex_position.x * zoom;
    let y = fragment.vertex_position.y * zoom;

    let noise_value = uniforms.noise_value.get_noise_2d(x, y);
    let normalized_noise = ((noise_value + 1.0) * 0.5).clamp(0.0, 1.0);

    // Definir el umbral para el efecto de fractura
    let fracture_threshold = 0.35;
    let is_fracture = normalized_noise > fracture_threshold;

    // Interpolación patrón rocoso y simular grietas
    let color_intermediate = color_roca.lerp(&color_sombra, normalized_noise * 0.8);
    let base_color = color_intermediate.lerp(&color_mineral, normalized_noise * 0.5);

    let final_color = if is_fracture {
        Color::new(60, 30, 10)
    } else {
        base_color
    };

    final_color * fragment.intensity
}

fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 80.0;
    let x = fragment.vertex_position.x * zoom;
    let y = fragment.vertex_position.y * zoom;

    let ruido = ruido_fractal(&uniforms.noise_open_simplex, x, y, 3, 2.76, 0.12);

    let color_base = Color::new(50, 50, 50);
    let color_sombra = Color::new(20, 20, 20); 
    let color_claro = Color::new(150, 150, 150);

    let factor = (ruido + 1.0) as f32 / 2.0;
    let mut color_final = color_base.lerp(&color_sombra, factor * 0.8); 
    color_final = color_final.lerp(&color_claro, factor * 0.5);

    color_final * fragment.intensity
}

fn ruido_fractal(noise: &FastNoiseLite, x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += noise.get_noise_2d(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;

        amplitude *= gain;
        frequency *= lacunarity;
    }

    total / max_value
}



fn earth_like_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let zoom = 200.0; 

    let noise_value = uniforms.noise_perlin.get_noise_2d(x * zoom, y * zoom);

    let normalized_noise = ((noise_value + 1.0) / 2.0).clamp(0.0, 1.0);

    let sea_level = 0.6;

    let is_land = normalized_noise > sea_level;

    let ocean_color = Color::new(0, 105, 148); 
    let land_color = Color::new(34, 139, 34);  

    let base_color = if is_land {
        land_color
    } else {
        ocean_color
    };

    base_color * fragment.intensity
}





pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: &str) -> Color {
    match shader_type {
        "solar_surface" => solar_shader(fragment, uniforms),
        "volcanic_planet_shader" => volcanic_planet_shader(fragment, uniforms),
        "molten_core_planet_shader" => molten_core_planet_shader(fragment, uniforms),
        "crystal_planet_shader" => crystal_planet_shader(fragment, uniforms),
        "vortex" => vortex_planet_shader(fragment, uniforms),
        "ringed_planet" => ringed_planet(fragment, uniforms),
        "ring_shader" => ring_shader(fragment, uniforms),
        "moon_shader" => moon_shader(fragment, uniforms),
        "rocky_planet" => rocky_planet(fragment, uniforms),
        "earth_like_planet_shader" => earth_like_planet_shader(fragment, uniforms),
        _ => Color::new(0, 0, 0),
    }
}

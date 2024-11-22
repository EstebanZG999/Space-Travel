# SpaceTravel: Simulación 3D de un Sistema Solar en Rust

## Descripción
Este proyecto es una aplicación de visualización 3D desarrollada en Rust, que permite a los usuarios navegar y explorar modelos tridimensionales interactivos. Utiliza una interfaz intuitiva para controlar la cámara, realizar zoom y desplazarse a puntos de interés predefinidos.

## Características
- **Gráficos 3D en tiempo real**: Renderizado de esferas que representan planetas y estrellas.
- **Iluminación dinámica**: Efectos de luz y sombreado calculados en tiempo real.
- **Órbitas simuladas**: Movimiento orbital basado en cálculos matemáticos.
- **Shaders personalizados**: Simulaciones de superficies como lava, cristales, atmósferas gaseosas y más.
- **Interacción en tiempo real**: Rotación, escalado y movimiento de cuerpos celestes durante la ejecución.
- **Cambio dinámico de shaders**: Alterna entre diferentes shaders para explorar visualizaciones únicas.

## Controles
- **Rotación de la Cámara:**
  - `W`: Rotar hacia arriba
  - `S`: Rotar hacia abajo
- **Dirección de la Cámara:**
  - `Up Arrowkey`: Mover hacia arriba
  - `Down Arrowkey`: Mover hacia abajo
  - `Left Arrowkey`: Mover hacia la izquierda
  - `Right Arrowkey`: Mover hacia la derecha
- **Zoom:**
  - `Q`: Acercar
  - `E`: Alejar
- **Warp a Puntos de Interés:**
  - `1` - `7`: Teletransportarse a los puntos de warp correspondientes

## Instalación
### Prerrequisitos
- **Rust:** Asegúrate de tener Rust instalado. Puedes descargarlo desde [rust-lang.org](https://www.rust-lang.org/).
- **Cargo:** El gestor de paquetes de Rust, que generalmente se instala junto con Rust.
- **Dependencias del Sistema:** Algunas bibliotecas gráficas pueden requerir dependencias adicionales según el sistema operativo.

### Pasos de Instalación
1. **Clonar el Repositorio:**
    ```bash
    git clone https://github.com/usuario/proyecto-visualizacion-3d.git
    ```
2. **Navegar al Directorio del Proyecto:**
    ```bash
    cd proyecto-visualizacion-3d
    ```
3. **Instalar Dependencias:**
    ```bash
    cargo install --path .
    ```
4. **Ejecutar la Aplicación:**
    ```bash
    cargo run
    ```

# Estructura del Proyecto

## Directorio `src/`

- `color.rs`: Define estructuras y operaciones relacionadas con colores.
- `fragment.rs`: Módulo para implementar shaders de fragmento.
- `framebuffer.rs`: Maneja el framebuffer para renderizado de gráficos.
- `line.rs`: Contiene utilidades para el dibujo de líneas en la pantalla.
- `main.rs`: Archivo principal que contiene la lógica de la aplicación.
- `obj.rs`: Carga y procesamiento de modelos 3D en formato OBJ.
- `shaders.rs`: Contiene los shaders personalizados para efectos visuales avanzados.
- `skybox.rs`: Implementa la lógica para renderizar un cielo alrededor del entorno 3D.
- `triangle.rs`: Funciones para renderizar triángulos en la escena.
- `vertex.rs`: Define estructuras y operaciones para vértices, incluyendo transformaciones.

- `assets/`
- `Cargo.toml`: Archivo de configuración del proyecto.
- `README.md`: Documentación del proyecto.

## Capturas de pantalla

https://github.com/user-attachments/assets/8b7ce407-9854-4d0d-ab0b-baf25f2a2805


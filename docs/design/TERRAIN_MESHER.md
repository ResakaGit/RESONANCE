# BLUEPRINT — Motor de Mallas de Terreno (Procedural Terrain Mesher)

> **Nota Histórica**: Este documento reemplaza al obsoleto `BLUEPRIN_GEOMETRIY_ENGINE.MD`, el cual establecía erróneamente que "la energía dicta la topología". En Resonance, el **Axioma de Topología** impera: El planeta (Terreno) existe antes de la energía, y determina la altitud física. La energía (V7) sólo modula el aspecto visual e influencia las entidades (Geometry Flow).

Diseñar el motor gráfico del terreno como un módulo Stateless (sin estado) y bajo el paradigma DoD (Diseño Orientado a Datos) sigue siendo la decisión arquitectónica más eficiente, especialmente trabajando en Rust y Bevy.

Al ser *stateless*, el motor de mallas se convierte en una tubería funcional pura (Pipeline). No guarda memoria de las celdas directamente; simplemente recibe arreglos contiguos en memoria (Altitud y Aspecto Visual) y escupe geometría (vértices, normales, colores e índices) para que Bevy la renderice.

Aquí tienes el Blueprint de alto nivel del motor ajustado a la física de Resonance.

## 1. Diseño Orientado a Datos (DoD): El Layout de Memoria

En DoD, la regla de oro es el respeto por la memoria Caché de la CPU. No usamos objetos tipo `Clase Terreno { x, y, z, energía, bioma }`. 
El input del mesher divide las responsabilidades:
1. **Altitud y Geometría base**: Viene estricta y únicamente del `TerrainField` (Topología procedural calculada en Startup).
2. **Apariencia Visual**: Viene derivada del cruce entre el Terreno y el `EnergyFieldGrid` (generando la paleta/tintes en un paquete temporal como `TerrainVisuals`).

```rust
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::topology::terrain_field::TerrainField;

// --- INPUT DOD: LA APARIENCIA DEL TERRENO ---
// Un arreglo plano precalculado en la frontera hexagonal por
// el sistema `visual_derivation` (cruza TerrainField + EnergyFieldGrid).
pub struct TerrainVisuals {
    // Tinte o material derivado por la alquimia/energía que reposa en la celda
    pub vertex_colors: Vec<[f32; 4]>, 
}
```

## 2. El Módulo Stateless: La Tubería de Terreno

Este módulo no tiene estado interno general. Es un conjunto de funciones puras. Su trabajo final es fusionar el Terreno Físico y sus Visuales Efímeros en un `Mesh` nativo de Bevy.

```rust
// --- EL MOTOR (Stateless) ---
pub struct ProceduralTerrainMesher;

impl ProceduralTerrainMesher {
    
    /// La función principal de la tubería. Recibe el sustrato topológico y los visuales.
    pub fn generate_terrain_mesh(terrain: &TerrainField, visuals: &TerrainVisuals) -> Mesh {
        // 1. Calcular dimensiones en las grillas (garantizado O(1) alignment)
        let width = terrain.width as usize;
        let height = terrain.height as usize;
        let num_vertices = width * height;
        let num_indices = (width - 1) * (height - 1) * 6;
        
        // 2. Data-Oriented: Arreglos planos y pre-asignados
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        // 3. PASO DE VÉRTICES (Altitud desde Topología real)
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                
                // La ALTURA viene exclusivamente del mapa topológico, no de la energía.
                let altitude = terrain.altitude[idx]; 
                
                let pos_x = (x as f32) * terrain.cell_size;
                let pos_z = (y as f32) * terrain.cell_size;
                
                positions.push([pos_x, altitude, pos_z]);
                uvs.push([x as f32 / width as f32, y as f32 / height as f32]);
                colors.push(visuals.vertex_colors[idx]); // Tinte por energía
                
                // Las normales reales se recalculan cruzando alturas vecinas.
                normals.push([0.0, 1.0, 0.0]); 
            }
        }

        // 4. PASO DE ÍNDICES (Unir vértices)
        Self::generate_indices(&mut indices, width, height);
        
        // 5. RECALCULAR NORMALES SUAVES
        Self::calculate_smooth_normals(&positions, &indices, &mut normals);

        // 6. CONSTRUIR EL MESH DE BEVY
        Self::build_bevy_mesh(positions, normals, uvs, colors, indices)
    }

    // --- Funciones Puras Internas ---
    
    fn generate_indices(indices: &mut Vec<u32>, width: usize, height: usize) {
        for y in 0..(height - 1) {
            for x in 0..(width - 1) {
                let current = (y * width + x) as u32;
                let next_row = ((y + 1) * width + x) as u32;
                // Triángulo 1
                indices.push(current);
                indices.push(next_row);
                indices.push(current + 1);
                // Triángulo 2
                indices.push(current + 1);
                indices.push(next_row);
                indices.push(next_row + 1);
            }
        }
    }

    fn calculate_smooth_normals(positions: &[[f32; 3]], indices: &[u32], normals: &mut [[f32; 3]]) {
        // Implementación de promediado de caras por vértice para un terreno sin costuras...
    }

    fn build_bevy_mesh(
        positions: Vec<[f32; 3]>, 
        normals: Vec<[f32; 3]>, 
        uvs: Vec<[f32; 2]>, 
        colors: Vec<[f32; 4]>,
        indices: Vec<u32>
    ) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList, 
            RenderAssetUsages::default()
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}
```

## 3. Integración asíncrona en Bevy

El motor **no tiene estado**, por lo que se ejecuta fuera del hilo principal (`Main Thread`). 
1. `Startup` pre-calcula el `TerrainField`.
2. Las disipaciones y emisiones V7 ocurren asíncronamente en el simulador.
3. Se toman snapshots de la visibilidad (`TerrainVisuals`).
4. Bevy despacha un `Async Task`. El worker calcula todo el SoA y retorna el `Mesh` empaquetado y subido al VRAM vía AssetServer. 

### Justificación Objetiva de las Capas (Resonance "Yanagi"):

- **Invariante Físico:** En la Tierra Real, la radiación en Chernobyl no altera la altura de las montañas. Modifica la pigmentación o muta plantas (cuyos modelos los hace el **Geometry Flow**), pero el piso sigue siendo el piso. Resonance separa la Altitud del Mundo `TerrainField` de la alquimia de energía `EnergyFieldGrid`.
- **Aislamiento de Render:** La malla en Bevy no importa al simulador mecánico. Este motor es una traducción puramente cosméstica entre el *Modelo Matemático* y la *Tarjeta de Video*. 
- **Caché Eficiente:** Arrays planos alineados al mismo `cell_size` e índices equivalentes significa `O(1)` index access para todo el sistema visual. CPU Feliz, FPS fluidos.

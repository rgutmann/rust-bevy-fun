use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use noise::{utils::*, Fbm, Perlin};

// TODO:
// pub struct TerrainMap {
//     size: (usize, usize),
//     border_value: f64,
//     map: Vec<f64>,
// }

pub fn generate_noisemap(extent: f64, width: usize, depth: usize, frequency: f64, lacunarity: f64, octaves: usize, create_file: bool) -> NoiseMap {
    let mut fbm = Fbm::<Perlin>::default();
    fbm.frequency = frequency;
    fbm.lacunarity = lacunarity;
    fbm.octaves = octaves;
    let noisemap = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(fbm)
        .set_size(width, depth)
        .set_x_bounds(-extent, extent)
        .set_y_bounds(-extent, extent)
        .set_is_seamless(true)
        .build();

    if create_file {
        noisemap.write_to_file("assets/fbm.png");
    }
    noisemap
}

///
/// https://lejondahl.com/heightmap/
/// https://www.renderosity.com/freestuff/items/77673/seamless-tileable-elevation-map-with-texture-map
pub fn create_mesh(extent: f64, width: usize, depth: usize, noisemap: NoiseMap, intensity: f32) -> Mesh {
    let vertices_count: usize = (width + 1) * (depth + 1);
    let triangle_count: usize = width * depth * 2 * 3;

    // Cast
    let (width_u32, depth_u32) = (width as u32, depth as u32);
    let (width_f32, depth_f32) = (width as f32, depth as f32);
    let extent_f32 = extent as f32;

    // Defining vertices.
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);

    let mut min_height = f32::MAX;
    let mut max_height = f32::MIN;
    for d in 0..=width {
        for w in 0..=depth {
            let (w_f32, d_f32) = (w as f32, d as f32);

            let pos = [
                (w_f32 - width_f32 / 2.) * extent_f32 / width_f32,
                (noisemap.get_value(w, d) as f32) * intensity,
                (d_f32 - depth_f32 / 2.) * extent_f32 / depth_f32,
            ];
            positions.push(pos);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([w_f32 / width_f32, d_f32 / depth_f32]);

            // TODO: remove when not needed anymore
            let height = noisemap.get_value(w, d) as f32;
            min_height = min_height.min(height);
            max_height = max_height.max(height);
        }
    }
    println!("Terrain MIN height: {} - MAX height: {}", min_height, max_height);

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

    for d in 0..depth_u32 {
        for w in 0..width_u32 {
            // First tringle
            triangles.push((d * (width_u32 + 1)) + w);
            triangles.push(((d + 1) * (width_u32 + 1)) + w);
            triangles.push(((d + 1) * (width_u32 + 1)) + w + 1);
            // Second triangle
            triangles.push((d * (width_u32 + 1)) + w);
            triangles.push(((d + 1) * (width_u32 + 1)) + w + 1);
            triangles.push((d * (width_u32 + 1)) + w + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(triangles)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}
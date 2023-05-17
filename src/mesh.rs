use image::io::Reader as ImageReader;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use noise::{utils::*, Fbm, Perlin};

/// 
pub struct ElevationMap {
    size: (usize, usize),
    map: Vec<f64>,
}

// TODO:
impl ElevationMap {

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            map: vec![0.0; width * height],
        }
    }

    pub fn new_with_data(width: usize, height: usize, map: Vec<f64>) -> Self {
        assert!(map.len() == width * height, "map length mismatch!");
        Self {
            size: (width, height),
            map,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: f64) {
        let (width, height) = self.size;

        if x < width && y < height {
            self.map[x + y * width] = value;
        } else {
            eprintln!("illegal position given: ({}, {})", width, height);
        }
    }

    pub fn get_value(&self, x: usize, y: usize) -> f64 {
        let (width, height) = self.size;

        if x < width && y < height {
            self.map[x + y * width]
        } else if (x == width && y <= height) || (y == height && x <= width) {
            0. // nornal border
        } else {
            eprintln!("illegal position requested: ({}, {})", x, y);
            -1.
        }
    }

}


pub fn load_elevation_map(filename: &str, max_height: f64) -> ElevationMap {
    let dyn_image = ImageReader::open(filename).unwrap().decode().unwrap();
    let gray_image = dyn_image.as_luma8().unwrap();
    println!("image loaded with dimension: {:?}", gray_image.dimensions());
    ElevationMap::new_with_data(
        gray_image.width() as usize, 
        gray_image.height() as usize, 
        gray_image.to_vec().into_iter().map(|x| (x as f64) * max_height / 256.0).collect()
    )
}


/// 
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
        noisemap.write_to_file("fbm.png");
    }
    noisemap
}

///
/// https://lejondahl.com/heightmap/
/// https://www.renderosity.com/freestuff/items/77673/seamless-tileable-elevation-map-with-texture-map
pub fn create_mesh(extent: f64, width: usize, depth: usize, map: ElevationMap, intensity: f32) -> Mesh {
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
    for d in 0..=depth {
        for w in 0..=width {
            let (w_f32, d_f32) = (w as f32, d as f32);

            let pos = [
                (w_f32 - width_f32 / 2.) * extent_f32 / width_f32,
                (map.get_value(w, d) as f32) * intensity,
                (d_f32 - depth_f32 / 2.) * extent_f32 / depth_f32,
            ];
            positions.push(pos);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([w_f32 / width_f32, d_f32 / depth_f32]);

            // TODO: remove when not needed anymore
            let height = map.get_value(w, d) as f32;
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
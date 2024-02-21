use image::io::Reader as ImageReader;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use noise::{utils::*, Fbm, Perlin};

/// Represents an elevation map with a given size and elevation values.
pub struct ElevationMap {
    size: (usize, usize),
    map: Vec<f64>,
}

impl ElevationMap {
    /// Creates a new `ElevationMap` with the specified width and height.
    pub fn _new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            map: vec![0.0; width * height],
        }
    }

    /// Creates a new `ElevationMap` with the specified width, height, and elevation data.
    /// Panics if the length of the provided map does not match the width * height.
    pub fn new_with_data(width: usize, height: usize, map: Vec<f64>) -> Self {
        assert!(map.len() == width * height, "map length mismatch!");
        Self {
            size: (width, height),
            map,
        }
    }

    /// Returns the size (width, height) of the elevation map.
    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    /// Sets the elevation value at the specified position (x, y).
    /// Prints an error message if the position is out of bounds.
    pub fn _set_value(&mut self, x: usize, y: usize, value: f64) {
        let (width, height) = self.size;

        if x < width && y < height {
            self.map[x + y * width] = value;
        } else {
            eprintln!("illegal position given: ({}, {})", width, height);
        }
    }

    /// Returns the elevation value at the specified position (x, y).
    /// If the position is out of bounds, prints an error message and returns -1.0.
    pub fn get_value(&self, x: usize, y: usize) -> f64 {
        let (width, height) = self.size;

        if x < width && y < height {
            self.map[x + y * width]
        } else if (x == width && y <= height) || (y == height && x <= width) {
            0.0 // normal border
            // TODO: this probably needs to be changed to the overlapping border value
        } else {
            eprintln!("illegal position requested: ({}, {})", x, y);
            -1.0
        }
    }
}

/// Loads an elevation map from the specified image file and returns an `ElevationMap` object.
/// The maximum height of the map is specified by `max_height`.
pub fn load_elevation_map(filename: &str, max_height: f64) -> ElevationMap {
    let dyn_image = ImageReader::open(filename).unwrap().decode().unwrap();
    let gray_image = dyn_image.as_luma8().unwrap();
    println!("image loaded with dimension: {:?}", gray_image.dimensions());
    ElevationMap::new_with_data(
        gray_image.width() as usize,
        gray_image.height() as usize,
        gray_image.to_vec().into_iter().map(|x| (x as f64) * max_height / 256.0).collect(),
    )
}

/// Generates a noise map using the Fast Brownian Motion algorithm and returns a `NoiseMap` object.
/// The `extent` parameter determines the size of the map.
/// The `width` and `depth` parameters determine the resolution of the map.
/// The `frequency`, `lacunarity`, and `octaves` parameters control the characteristics of the noise.
/// If `create_file` is true, the generated noise map will be saved as "fbm.png".
pub fn _generate_noisemap(
    extent: f64,
    width: usize,
    depth: usize,
    frequency: f64,
    lacunarity: f64,
    octaves: usize,
    create_file: bool,
) -> NoiseMap {
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

/// Creates a mesh based on the given parameters and returns a `Mesh` object.
/// The `extent` parameter determines the size of the mesh in the real world.
/// The `mesh_width` and `mesh_depth` parameters determine the resolution of the mesh.
/// The `map` parameter is an `ElevationMap` object containing the elevation data.
/// The `intensity` parameter controls the vertical scaling of the mesh.
pub fn create_mesh(extent: f64, mesh_pos: (isize, isize), mesh_size: (usize, usize), map: &ElevationMap, intensity: f32) -> Mesh {
    let (map_width, map_depth) = map.size;
    let (mesh_width, mesh_depth) = mesh_size;
    let (mut mesh_x, mut mesh_y) = mesh_pos;
    if mesh_x < 0 { mesh_x += map_width as isize; }
    if mesh_y < 0 { mesh_y += map_depth as isize; }

    let vertices_count: usize = (mesh_width + 1) * (mesh_depth + 1);
    let triangle_count: usize = mesh_width * mesh_depth * 2 * 3;

    // Cast
    let (width_u32, depth_u32) = (mesh_width as u32, mesh_depth as u32);
    let (width_f32, depth_f32) = (mesh_width as f32, mesh_depth as f32);
    let extent_f32 = extent as f32;

    // Defining vertices.
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);
    for d in 0..=mesh_depth {
        for w in 0..=mesh_width {
            // Calculate the position in the elevation map, considering repetition
            let map_x = (mesh_x as usize + w) % map_width;
            let map_y = (mesh_y as usize + d) % map_depth;

            // Cast
            let (w_f32, d_f32) = (w as f32, d as f32);

            let pos = [
                (w_f32 - width_f32 / 2.) * extent_f32 / width_f32,
                (map.get_value(map_x, map_y) as f32) * intensity,
                (d_f32 - depth_f32 / 2.) * extent_f32 / depth_f32,
            ];
            positions.push(pos);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([w_f32 / width_f32, d_f32 / depth_f32]);
        }
    }

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);
    for d in 0..depth_u32 {
        for w in 0..width_u32 {
            // First triangle
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
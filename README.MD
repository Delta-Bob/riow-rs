## Personal Raytracing Project Made in Rust
Based upon the works of the “Ray Tracing in One Weekend Series” book series v4.0.2 by authors Peter Shirley, Trevor David Black, Steve Hollasch. Adapted to the Rust programming language.
Source: [_Ray Tracing in One Weekend_](https://raytracing.github.io/books/RayTracingInOneWeekend.html)

### How To Run:
Pipe output to an image file
```
cargo run --release > output.ppm
```

### Features:
- Different materials
  - Matte
  - Transparent
  - Metallic
  - Emissive
  - Volumetric
  - Perlin noise
- Advanced camera settings
- Primatives
  - Triangles
  - Spheres
  - Planes
- BVH acceleration
- Parallelisation
- Textures

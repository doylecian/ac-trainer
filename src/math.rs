pub fn euclid_dist(start: (f32, f32), end: (f32, f32)) -> f32{
    ((end.0 - start.0).powf(2.0) + (end.1 - start.1).powf(2.0)).abs().sqrt()
}
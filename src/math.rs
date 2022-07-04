pub fn euclid_dist(start: (f32, f32), end: (f32, f32)) -> f32{
    ((end.0 - start.0).powf(2.0) + (end.1 - start.1).powf(2.0)).abs().sqrt()
}

pub fn angle_to_y((x, y): (f32, f32)) -> f32 {
    let angle: f32 = {
        if y < 0.0 {
            println!("180 - {:.4}", (x / y).atan().to_degrees());
            if x < 0.0 {
                180.0 - (x / y).atan().to_degrees().abs()
            }
            else {
                -180.0 + (x / y).atan().to_degrees().abs()
            }
        }
        else {
            (x / y).atan().to_degrees()
        }
    };
    angle
}
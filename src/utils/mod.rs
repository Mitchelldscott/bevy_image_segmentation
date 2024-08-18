use bevy::color::Color;


pub fn random_color() -> Color {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r: f32 = rng.gen(); // Generates a random float between 0.0 and 1.0
    let g: f32 = rng.gen();
    let b: f32 = rng.gen();

    Color::srgb(r, g, b)
}

pub mod image_copy;
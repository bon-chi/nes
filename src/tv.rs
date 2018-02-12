use graphics::types::Color;
use graphics::{Context, Graphics};
use NesController;

pub struct Tv {
    size: u8,
}
impl Tv {
    pub fn new() -> Tv {
        Tv { size: 1 }
    }
    pub fn draw<G: Graphics>(&self, controller: &NesController, c: &Context, g: &mut G) {
        use graphics::Rectangle;
        Rectangle::new([0.0, 1.0, 0.0, 1.0]).draw([0.0, 0.0, 85.0, 80.0], &c.draw_state, c.transform, g);
        Rectangle::new([1.0, 0.0, 0.0, 1.0]).draw([85.0, 80.0, 85.0, 80.0], &c.draw_state, c.transform, g);
        Rectangle::new([0.0, 0.0, 1.0, 1.0]).draw([170.0, 160.0, 85.0, 80.0], &c.draw_state, c.transform, g);

    }
    pub fn get_rectangle(&self, x: u8, y: u8) -> [f64; 4] {
        let size = self.size as f64;
        [(size * (x as f64)), (size * (y as f64)), size, size]
    }
}

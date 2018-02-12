use nes::nes::Nes;

pub struct NesController {
    nes: Nes,
}

impl NesController {
    pub fn new(nes: Nes) -> NesController {
        NesController { nes }
    }
}

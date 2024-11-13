mod ast;
mod compile;
mod engine;
mod grid;
mod util;

use engine::Cpu;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    let codel_size = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let data = if path.starts_with("http") {
        let mut buf = Vec::new();
        let response = ureq::get(&path).call().unwrap();
        response.into_reader().read_to_end(&mut buf).unwrap();
        buf
    } else {
        std::fs::read(&path).unwrap()
    };

    let image = image::load_from_memory(&data).unwrap().to_rgb8();
    let program = compile::compile(&image, codel_size);

    Cpu::default().run(&program);
}

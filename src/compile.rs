use std::collections::HashSet;

use image::{Rgb, RgbImage};

use crate::ast::{Codel, Color, ColorBlock, Hue, Lightness, Program};
use crate::grid::Grid;
use crate::util::{iter_2d, Direction};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum RawCodel {
    #[default]
    White,
    Black,
    Color(Color),
}

pub fn compile(image: &RgbImage, codel_size: u32) -> Program {
    parse(&lex(image, codel_size))
}

impl RawCodel {
    fn decode(pixel: Rgb<u8>) -> Option<Self> {
        let rgb = pixel.0;
        // quantize samples to the nearest "canonical value"
        let rgb = rgb.map(|n| match n {
            0x00..0x60 => 0x00,
            0x60..0xE0 => 0xC0,
            0xE0..=0xFF => 0xFF,
        });

        let min = rgb.iter().copied().min().unwrap();
        let max = rgb.iter().copied().max().unwrap();

        if rgb.iter().any(|&v| v != min && v != max) {
            return None;
        }

        let lightness = match (min, max) {
            (0x00, 0x00) => return Some(RawCodel::Black),
            (0xFF, 0xFF) => return Some(RawCodel::White),
            (0xC0, 0xFF) => Lightness::Light,
            (0x00, 0xFF) => Lightness::Normal,
            (0x00, 0xC0) => Lightness::Dark,
            _ => return None,
        };

        let hue = match rgb.map(|v| v == max) {
            [true, false, false] => Hue::Red,
            [true, true, false] => Hue::Yellow,
            [false, true, false] => Hue::Green,
            [false, true, true] => Hue::Cyan,
            [false, false, true] => Hue::Blue,
            [true, false, true] => Hue::Magenta,
            _ => unreachable!(),
        };

        Some(RawCodel::Color(Color { hue, lightness }))
    }
}

fn lex(image: &RgbImage, codel_size: u32) -> Grid<RawCodel> {
    let w = image.width() / codel_size;
    let h = image.height() / codel_size;
    Grid::from_fn(w as usize, h as usize, |x, y| {
        RawCodel::decode(*image.get_pixel(codel_size * x as u32, codel_size * y as u32))
            .unwrap_or(RawCodel::White)
    })
}

fn parse(raw: &Grid<RawCodel>) -> Program {
    let (w, h) = raw.size();
    let mut blocks = vec![];
    let mut grid = Grid::<Codel>::new(w, h);

    for (x, y) in iter_2d(0..w, 0..h) {
        if grid[(x, y)] != Codel::White {
            // this codel has already been handled by the color block flood fill
            continue;
        }

        let color = match raw[(x, y)] {
            RawCodel::White => continue,
            RawCodel::Black => {
                grid[(x, y)] = Codel::Black;
                continue;
            }
            RawCodel::Color(c) => c,
        };

        let codels = flood_fill(&raw, (x, y));
        let block_id = blocks.len();
        for coord in &codels {
            grid[*coord] = Codel::ColorBlock(block_id);
        }
        blocks.push(ColorBlock { color, codels });
    }

    Program { blocks, grid }
}

fn flood_fill<T: PartialEq>(grid: &Grid<T>, start: (usize, usize)) -> HashSet<(usize, usize)> {
    let value = &grid[start];
    let mut to_visit = vec![start];
    let mut visited = HashSet::from([start]);

    let wh = grid.size();

    while let Some(xy) = to_visit.pop() {
        for dir in Direction::NESW {
            let Some(neighbor) = dir.go(xy, wh) else {
                continue;
            };

            if grid[neighbor] != *value {
                continue;
            }

            let Some(neighbor_value) = grid.get(neighbor.0, neighbor.1) else {
                continue;
            };

            if neighbor_value == value && !visited.contains(&neighbor) {
                to_visit.push(neighbor);
                visited.insert(neighbor);
            }
        }
    }

    visited
}

use std::collections::HashSet;

use crate::engine::Command;
use crate::grid::Grid;
use crate::util::{Direction, Rotation};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum Hue {
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i8)]
pub enum Lightness {
    Light = 0,
    Normal = 1,
    Dark = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
    pub hue: Hue,
    pub lightness: Lightness,
}

impl Color {
    pub fn compare(old: Self, new: Self) -> Command {
        use crate::engine::{BinaryOp::*, Command::*, UnaryOp::*};
        const COMMANDS: [[Command; 3]; 6] = [
            [Nop, Push, Unary(Pop)],
            [Binary(Add), Binary(Subtract), Binary(Multiply)],
            [Binary(Divide), Binary(Mod), Unary(Not)],
            [Binary(Greater), Unary(Pointer), Unary(Switch)],
            [Unary(Duplicate), Binary(Roll), InputNum],
            [InputChar, Unary(OutNum), Unary(OutChar)],
        ];

        let hue_change = (new.hue as i8 - old.hue as i8).rem_euclid(6);
        let lightness_change = (new.lightness as i8 - old.lightness as i8).rem_euclid(3);
        COMMANDS[hue_change as usize][lightness_change as usize]
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Codel {
    #[default]
    White,
    Black,
    ColorBlock(usize),
}

pub struct Program {
    pub blocks: Vec<ColorBlock>,
    pub grid: Grid<Codel>,
}

pub struct ColorBlock {
    pub color: Color,
    pub codels: HashSet<(usize, usize)>,
}

impl ColorBlock {
    pub fn find_edge(&self, dp: Direction, cc: Rotation) -> (usize, usize) {
        // find the furthest coordinate (x OR y) along the direction pointer
        let (x, y) = dp.find_max(self.codels.iter().copied());

        // find all codels that share the coordinate
        let is_along_edge = |xy: &(usize, usize)| match dp {
            Direction::Left | Direction::Right => xy.0 == x,
            Direction::Up | Direction::Down => xy.1 == y,
        };
        let along_edge = self.codels.iter().copied().filter(is_along_edge);

        // from those codels, choose the one that's furthest along the codel chooser
        dp.turn(cc).find_max(along_edge)
    }
}

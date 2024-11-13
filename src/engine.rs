use std::collections::HashSet;
use std::io::Write;

use crate::ast::{Codel, Color, ColorBlock, Program};
use crate::util::{self, Direction, Rotation};

#[derive(Default)]
pub struct Cpu {
    stack: Vec<i64>,
    regs: Registers,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Registers {
    pc: (usize, usize),
    dp: Direction,
    cc: Rotation,
}

impl Cpu {
    pub fn run(&mut self, program: &Program) {
        while let Some((command, operand)) = self.advance_pc(program) {
            self.execute_instruction(command, operand);
        }
    }

    fn advance_pc(&mut self, program: &Program) -> Option<(Command, i64)> {
        let mut deja_vu = HashSet::new();
        let mut toggled_cc = false;

        loop {
            // If we're inside a color block (as opposed to sliding through white)
            if let Some(block) = self.current_block(program) {
                // Navigate to "an end" of "the edge" of the current color block
                self.regs.pc = block.find_edge(self.regs.dp, self.regs.cc);
            }

            if !deja_vu.insert(self.regs) {
                // We've been here before, meaning we're stuck in an infinite loop.
                return None;
            }

            let target_pos = self.regs.dp.go(self.regs.pc, program.grid.size());
            // The edges of the program are treated identically to black codels.
            let target_codel = target_pos.map_or(Codel::Black, |p| program.grid[p]);

            match target_codel {
                Codel::White => {
                    self.regs.pc = target_pos.unwrap();
                    toggled_cc = false;
                }
                Codel::Black => {
                    if !toggled_cc {
                        self.regs.cc.toggle();
                        toggled_cc = true;
                    } else {
                        self.regs.dp.rotate(Rotation::Clock);
                        toggled_cc = false;
                    }
                }
                Codel::ColorBlock(id) => {
                    let mut command = Command::Nop;
                    let mut operand = 0;

                    // execute an operation if we didn't slide across white first
                    if let Some(old_block) = self.current_block(program) {
                        command = Color::compare(old_block.color, program.blocks[id].color);
                        operand = old_block.codels.len() as i64;
                    }

                    self.regs.pc = target_pos.unwrap(); // finally!

                    return Some((command, operand));
                }
            }
        }
    }

    fn current_block<'a>(&self, program: &'a Program) -> Option<&'a ColorBlock> {
        match program.grid[self.regs.pc] {
            Codel::ColorBlock(id) => Some(&program.blocks[id]),
            _ => None,
        }
    }

    fn execute_instruction(&mut self, command: Command, operand: i64) {
        std::io::stderr().flush().unwrap();
        match command {
            Command::Nop => {}
            Command::Push => {
                self.stack.push(operand);
            }
            Command::InputNum => {
                let line = std::io::stdin().lines().next().unwrap().unwrap();
                let n = line.trim_end_matches(['\r', '\n']).parse::<i64>().unwrap();
                self.stack.push(n);
            }
            Command::InputChar => {
                print!("? ");
                std::io::stdout().flush().unwrap();
                let c = util::read_utf8_char(std::io::stdin()).unwrap();
                self.stack.push(c as u32 as i64);
            }
            Command::Unary(op) => {
                let Some(x) = self.stack.pop() else { return };
                self.execute_unary(op, x);
            }
            Command::Binary(op) => {
                let Some(x) = self.stack.pop() else { return };
                let Some(y) = self.stack.pop() else {
                    self.stack.push(x);
                    return;
                };

                self.execute_binary(op, x, y);
            }
        }
    }

    fn execute_unary(&mut self, operation: UnaryOp, value: i64) {
        match operation {
            UnaryOp::Pop => {}
            UnaryOp::Not => self.stack.push(if value == 0 { 1 } else { 0 }),
            UnaryOp::Pointer => {
                let rotation = if value < 0 {
                    Rotation::Counter
                } else {
                    Rotation::Clock
                };
                for _ in 0..value.abs() {
                    self.regs.dp.rotate(rotation);
                }
            }
            UnaryOp::Switch => {
                for _ in 0..value.abs() {
                    self.regs.cc.toggle();
                }
            }
            UnaryOp::Duplicate => self.stack.extend([value, value]),
            UnaryOp::OutNum => print!("{value} "),
            UnaryOp::OutChar => {
                print!("{}", char::from_u32(value.try_into().unwrap()).unwrap());
                std::io::stdout().flush().unwrap();
            }
        }
    }

    fn execute_binary(&mut self, op: BinaryOp, x: i64, y: i64) {
        let result = match op {
            BinaryOp::Add => x + y,
            BinaryOp::Subtract => y - x,
            BinaryOp::Multiply => x * y,
            BinaryOp::Divide => y / x,
            BinaryOp::Mod => y % x,
            BinaryOp::Greater => (y > x) as i64,

            // This is the only case that does something other than push one result value.
            BinaryOp::Roll => {
                // if y > self.stack.len() as i64 {
                // return;
                // }
                let start = self.stack.len().saturating_sub(y as usize);
                let slice = &mut self.stack[start..];
                let amount = x.abs() as usize % slice.len();
                if x < 0 {
                    slice.rotate_left(amount);
                } else {
                    slice.rotate_right(amount);
                }
                return;
            }
        };

        self.stack.push(result);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Command {
    Nop,
    Push,
    InputNum,
    InputChar,
    Unary(UnaryOp),
    Binary(BinaryOp),
}

// Opcodes that require popping one value from the stack.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Pop,
    Not,
    Pointer,
    Switch,
    Duplicate,
    OutNum,
    OutChar,
}

// Opcodes that require popping two values from the stack.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Greater,
    Roll,
}

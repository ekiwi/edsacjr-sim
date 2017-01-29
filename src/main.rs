// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// Copyright 2017 by Kevin Läufer <kevin.laeufer@rwth-aachen.de>

// TODO: remove once code stabilizes
#![allow(dead_code)]

use std::fmt;

// TODO: remove redundancy in instruction definition with the use of macros
//       or compiler plugin in the spirit of:
//       https://crates.io/crates/enum_primitive
const ADD: u16    = 0b00001;
const SUB: u16    = 0b10000;
const STORE: u16  = 0b00010;
const CLEAR: u16  = 0b00011;
const OR: u16     = 0b00000;
const AND: u16    = 0b00100;
const SHIFTR: u16 = 0b00101;
const SHIFTL: u16 = 0b00110;
const BGE: u16    = 0b00111;
const BLT: u16    = 0b01000;
const END: u16    = 0b01010;

fn op_to_str(op: u16) -> &'static str{
	match op {
		ADD    => "ADD",
		SUB    => "SUB",
		STORE  => "STORE",
		CLEAR  => "CLEAR",
		OR     => "OR",
		AND    => "AND",
		SHIFTR => "SHIFTR",
		SHIFTL => "SHIFTL",
		BGE    => "BGE",
		BLT    => "BLT",
		END    => "END",
		_ => panic!("Unknown op code ({:05b})", op),
	}
}

const MAX_MEM: usize = 2048;

fn mask(bits: u32) -> u16 { (1 << bits) - 1 }

struct Regs { acc: i16, pc: u16 }
struct Instruction { w: u16 }
struct Integer {w: u16}

impl fmt::Display for Regs {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "pc: {:04}; acc: {:>6}", self.pc, self.acc)
	}
}

impl Instruction {
	fn new(op: u16, n: u16) -> Instruction {
		Instruction { w: (op << 11) | (n & mask(11)) }
	}
	fn load(pc: u16, mem: &[u16]) -> Instruction {
		Instruction { w: mem[pc as usize] }
	}
	fn n(&self) -> u16 { self.w & mask(11) }
	fn op(&self) -> u16 { self.w >> 11 }
	fn arg(&self, mem: &[u16]) -> i16 { mem[self.n() as usize] as i16 }
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {}", op_to_str(self.op()), self.n())
	}
}

fn add(n: u16)   -> u16 { Instruction::new(ADD, n).w }
fn sub(n: u16)   -> u16 { Instruction::new(SUB, n).w }
fn store(n: u16) -> u16 { Instruction::new(STORE, n).w }
fn clear()       -> u16 { Instruction::new(CLEAR, 0).w }
fn or(n: u16)    -> u16 { Instruction::new(OR, n).w }
fn and(n: u16)   -> u16 { Instruction::new(AND, n).w }
fn end()         -> u16 { Instruction::new(END, 0).w }
fn con(n: i16)   -> u16 { n as u16 }


fn exec(old: Regs, mem: &[u16]) -> Regs {
	let instr = Instruction::load(old.pc, mem);
	match instr.op() {
		// TODO: implement correct under/overflow behavior
		ADD    => Regs { acc: old.acc + instr.arg(mem), pc: old.pc + 1},
		SUB    => Regs { acc: old.acc - instr.arg(mem), pc: old.pc + 1},
		STORE  => Regs { acc: old.acc, pc: old.pc + 1}, // TODO: side effect
		CLEAR  => Regs { acc: 0, pc: old.pc + 1},
		OR     => Regs { acc: old.acc | instr.arg(mem), pc: old.pc + 1},
		AND    => Regs { acc: old.acc & instr.arg(mem), pc: old.pc + 1},
		SHIFTR => Regs { acc: old.acc >> instr.n(), pc: old.pc + 1},
		SHIFTL => Regs { acc: old.acc << instr.n(), pc: old.pc + 1},
		BGE    => Regs { acc: old.acc,
		                 pc: if old.acc >= 0 { instr.n() } else { old.pc + 1} },
		BLT    => Regs { acc: old.acc,
		                 pc: if old.acc <  0 { instr.n() } else { old.pc + 1} },
		END    => Regs { acc: old.acc, pc: old.pc}, // TODO: signal halted
		_ => panic!("Unknown op code ({:05b})", instr.op()),
	}
}

//fn print()

fn run(pc: u16, mem: &[u16]) -> Regs {
	let mut regs = Regs {acc: 0, pc: pc};
	while Instruction::load(regs.pc, mem).op() != END {
		print!("{:04}: {}", regs.pc, Instruction::load(regs.pc, mem));
		let acc_old = regs.acc;
		regs = exec(regs, mem);
		println!("\tacc: {:>6} => {:>6}", acc_old, regs.acc);
	}
	regs
}

fn main() {
	let mem: [u16; 6] = [
		add(4),
		add(5),
		clear(),
		end(),
		con(20),
		con(-30)];
	run(0, &mem);
}

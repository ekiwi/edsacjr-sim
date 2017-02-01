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
use std::ops;

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

struct Regs { acc: Integer, pc: u16 }
struct Instruction { w: u16 }
#[derive(Copy, Clone)]
struct Integer { w: u16}

impl Regs {
	fn new(pc: u16) -> Regs {
		Regs {acc: Integer::new(0), pc: pc}
	}
}
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
	fn arg(&self, mem: &[u16]) -> Integer { Integer::load(self.n(), mem) }
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {}", op_to_str(self.op()), self.n())
	}
}

impl Integer {
	fn new(ii: i16) -> Integer {
		let abs_val = (ii.abs() as u16) & ((1<<15) - 1);
		if ii >= 0 { Integer { w: abs_val } }
		else { Integer { w: (1<<15) | abs_val } }
	}
	fn load(index: u16, mem: &[u16]) -> Integer { Integer { w: mem[index as usize] } }
	fn is_positive(&self) -> bool { self.w & (1 << 15) == 0}
	fn abs(&self) -> u16 { self.w & ((1<<15) - 1) }
	fn sign(&self) -> u16 { self.w & (1<<15) }
	fn less_than_zero(&self) -> bool { !self.is_positive() && self.abs() != 0 }
}

impl fmt::Display for Integer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}", if self.is_positive() { "" } else { "-" }, self.abs())
	}
}

impl ops::Add for Integer {
	type Output = Integer;
	fn add(self, other: Integer) -> Integer {
		if self.is_positive() == other.is_positive() {
			let result = self.abs() as u32 + other.abs() as u32;
			if result > ((1<<15) - 1) {
				panic!("Overflow detected trying to execute {} + {}", self, other);
			}
			Integer { w: result as u16 | self.sign() }
		} else {
			let pos = if self.is_positive() { self.abs()  } else { other.abs() };
			let neg = if self.is_positive() { other.abs() } else { self.abs() };
			if pos >= neg {
				Integer { w: pos - neg }
			} else {
				Integer { w: neg - pos | (1 << 15) }
			}
		}
	}
}

impl ops::Sub for Integer {
	type Output = Integer;
	fn sub(self, other: Integer) -> Integer {
		self + Integer { w: other.w ^ (1<<15) }
	}
}

impl ops::BitOr for Integer {
	type Output = Integer;
	fn bitor(self, other: Integer) -> Integer {
		Integer { w: self.w | other.w }
	}
}

impl ops::BitAnd for Integer {
	type Output = Integer;
	fn bitand(self, other: Integer) -> Integer {
		Integer { w: self.w & other.w }
	}
}

impl ops::Shr<u16> for Integer {
	type Output = Integer;
	fn shr(self, other: u16) -> Integer {
		Integer { w: (self.abs() >> other) | self.sign() }
	}
}

impl ops::Shl<u16> for Integer {
	type Output = Integer;
	fn shl(self, other: u16) -> Integer {
		Integer { w: (self.abs() << other) | self.sign() }
	}
}

fn add(n: u16)   -> u16 { Instruction::new(ADD, n).w }
fn sub(n: u16)   -> u16 { Instruction::new(SUB, n).w }
fn store(n: u16) -> u16 { Instruction::new(STORE, n).w }
fn clear()       -> u16 { Instruction::new(CLEAR, 0).w }
fn or(n: u16)    -> u16 { Instruction::new(OR, n).w }
fn and(n: u16)   -> u16 { Instruction::new(AND, n).w }
fn end()         -> u16 { Instruction::new(END, 0).w }
fn con(n: i16)   -> u16 { Integer::new(n).w }


fn exec(old: Regs, mem: &mut[u16]) -> Regs {
	let instr = Instruction::load(old.pc, mem);
	match instr.op() {
		// TODO: implement correct under/overflow behavior
		ADD    => Regs { acc: old.acc + instr.arg(mem), pc: old.pc + 1},
		SUB    => Regs { acc: old.acc - instr.arg(mem), pc: old.pc + 1},
		STORE  => { mem[instr.n() as usize] = old.acc.w;
		          Regs { acc: old.acc, pc: old.pc + 1} },
		CLEAR  => Regs { acc: Integer::new(0), pc: old.pc + 1},
		OR     => Regs { acc: old.acc | instr.arg(mem), pc: old.pc + 1},
		AND    => Regs { acc: old.acc & instr.arg(mem), pc: old.pc + 1},
		SHIFTR => Regs { acc: old.acc >> instr.n(), pc: old.pc + 1},
		SHIFTL => Regs { acc: old.acc << instr.n(), pc: old.pc + 1},
		BGE    => Regs { acc: old.acc,
		                 pc: if !old.acc.less_than_zero() { instr.n() } else { old.pc + 1} },
		BLT    => Regs { acc: old.acc,
		                 pc: if old.acc.less_than_zero() { instr.n() } else { old.pc + 1} },
		END    => Regs { acc: old.acc, pc: old.pc}, // TODO: signal halted
		_ => panic!("Unknown op code ({:05b})", instr.op()),
	}
}

fn run(pc: u16, mem: &mut[u16]) -> Regs {
	let mut regs = Regs::new(pc);
	while Instruction::load(regs.pc, mem).op() != END {
		print!("{:04}: {}", regs.pc, Instruction::load(regs.pc, mem));
		let acc_old = regs.acc;
		regs = exec(regs, mem);
		println!("\tacc: {:>6} => {:>6}", acc_old, regs.acc);
	}
	regs
}

fn print_mem(mem: &[u16]) {
	let mut ii = 0;
	for w in mem {
		println!("{0:04}: {1:016b} ({2:>6} | {3:<10})", ii, w,
				Integer { w: *w }.to_string(),
				Instruction { w: *w }.to_string());
		ii = ii + 1;
	}
}

fn main() {
	let mut mem: [u16; 7] = [
		clear(),
		add(5),		// load 20
		add(6),		// add -30
		store(5),	// store result
		end(),
		con(20),
		con(-30)];

	print_mem(&mem);
	println!();
	run(0, &mut mem);
	println!();
	print_mem(&mem);
}

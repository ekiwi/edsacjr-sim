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

use std::fmt;
use std::ops;

const MAX_MEM: usize = 2048;
fn mask(bits: u32) -> u16 { (1 << bits) - 1 }

// Machine State
struct Regs { acc: Integer, pc: u16 }

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


// Instruction Definitions
#[allow(unused)]
struct InstrType { name: &'static str, opcode: u16, exec: fn(u16, Regs, &mut[u16]) -> Regs }

// unfortunately the synthax for generating function pointers on the fly
// is somewhat difficult to read for now, this should be fixed once the
// following change is merged: https://github.com/rust-lang/rfcs/pull/1558
const ADD:    InstrType = InstrType { name: "ADD",    opcode: 0b00001,
exec: { fn exec(n: u16, old: Regs, mem: &mut[u16]) -> Regs {
	Regs { acc: old.acc + Integer::load(n, mem), pc: old.pc + 1}
	} exec } };

const SUB:    InstrType = InstrType { name: "SUB",    opcode: 0b10000,
exec: { fn exec(n: u16, old: Regs, mem: &mut[u16]) -> Regs {
	Regs { acc: old.acc - Integer::load(n, mem), pc: old.pc + 1}
	} exec } };

const STORE:  InstrType = InstrType { name: "STORE",  opcode: 0b00010,
exec: { fn exec(n: u16, old: Regs, mem: &mut[u16]) -> Regs {
	{ mem[n as usize] = old.acc.w; Regs { acc: old.acc, pc: old.pc + 1} }
	} exec } };

const CLEAR:  InstrType = InstrType { name: "CLEAR",  opcode: 0b00011,
exec: { fn exec(_: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: Integer::new(0), pc: old.pc + 1}
	} exec } };

const OR:     InstrType = InstrType { name: "OR",     opcode: 0b00000,
exec: { fn exec(n: u16, old: Regs, mem: &mut[u16]) -> Regs {
	Regs { acc: old.acc | Integer::load(n, mem), pc: old.pc + 1}
	} exec } };

const AND:    InstrType = InstrType { name: "AND",    opcode: 0b00100,
exec: { fn exec(n: u16, old: Regs, mem: &mut[u16]) -> Regs {
	Regs { acc: old.acc & Integer::load(n, mem), pc: old.pc + 1}
	} exec } };

const SHIFTR: InstrType = InstrType { name: "SHIFTR", opcode: 0b00101,
exec: { fn exec(n: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: old.acc >> n, pc: old.pc + 1}
	} exec } };

const SHIFTL: InstrType = InstrType { name: "SHIFTL", opcode: 0b00110,
exec: { fn exec(n: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: old.acc << n, pc: old.pc + 1}
	} exec } };

const BGE:    InstrType = InstrType { name: "BGE",    opcode: 0b00111,
exec: { fn exec(n: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: old.acc, pc: if !old.acc.less_than_zero() { n } else { old.pc + 1} }
	} exec } };

const BLT:    InstrType = InstrType { name: "BLT",    opcode: 0b01000,
exec: { fn exec(n: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: old.acc, pc: if old.acc.less_than_zero() { n } else { old.pc + 1} }
	} exec } };

const END:    InstrType = InstrType { name: "END",    opcode: 0b01010,
exec: { fn exec(_: u16, old: Regs, _: &mut[u16]) -> Regs {
	Regs { acc: old.acc, pc: old.pc} // TODO: signal halted
	} exec } };


static INSTRUCTION_TYPES: [InstrType; 11] =
	[ADD, SUB, STORE, CLEAR, OR, AND, SHIFTR, SHIFTL, BGE, BLT, END];

impl InstrType {
	fn new(&self, n: u16) -> Instruction {
		Instruction::new(self.opcode, n)
	}
}

// Instruction represents an actual instruction in the program
struct Instruction { w: u16 }

impl Instruction {
	fn new(op: u16, n: u16) -> Instruction {
		Instruction { w: (op << 11) | (n & mask(11)) }
	}
	fn load(pc: u16, mem: &[u16]) -> Instruction {
		Instruction { w: mem[pc as usize] }
	}
	fn get_type(&self) -> &'static InstrType {
		let opcode = self.op();
		for tt in INSTRUCTION_TYPES.iter() {
			if tt.opcode == opcode {
				return tt;
			}
		}
		panic!("Unknown op code ({:05b})", opcode)
	}
	fn n(&self) -> u16 {
		return self.w & mask(11)
	}
	fn op(&self) -> u16 {
		return self.w >> 11
	}
	fn exec(&self, old: Regs, mem: &mut[u16]) -> Regs {
		(self.get_type().exec)(self.n(), old, mem)
	}
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let tt = self.get_type();
		write!(f, "{} {}", tt.name, self.n())
	}
}


// 16bit Integer format
#[derive(Copy, Clone)]
struct Integer { w: u16}
impl Integer {
	fn new(ii: i16) -> Integer {
		let abs_val = (ii.abs() as u16) & mask(15);
		if ii >= 0 { Integer { w: abs_val } }
		else { Integer { w: (1<<15) | abs_val } }
	}
	fn load(index: u16, mem: &[u16]) -> Integer { Integer { w: mem[index as usize] } }
	fn is_positive(&self) -> bool { self.w & (1 << 15) == 0}
	fn abs(&self) -> u16 { self.w & mask(15) }
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
			if result > mask(15) as u32 {
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

fn run(pc: u16, mem: &mut[u16]) -> Regs {
	let mut regs = Regs::new(pc);
	while {
		let instr = Instruction::load(regs.pc, mem);
		print!("{:04}: {}", regs.pc, instr);
		let acc_old = regs.acc;
		regs = instr.exec(regs, mem);
		println!("\tacc: {:>6} => {:>6}", acc_old, regs.acc);
		instr.op() != END.opcode
	}{}
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

fn add(n: u16)   -> u16 { ADD.new(n).w }
fn sub(n: u16)   -> u16 { SUB.new(n).w }
fn store(n: u16) -> u16 { STORE.new(n).w }
fn clear()       -> u16 { CLEAR.new(0).w }
fn or(n: u16)    -> u16 { OR.new(n).w }
fn and(n: u16)   -> u16 { AND.new(n).w }
fn end()         -> u16 { END.new(0).w }
fn con(n: i16)   -> u16 { Integer::new(n).w }

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

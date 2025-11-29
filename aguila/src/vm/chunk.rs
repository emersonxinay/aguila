use crate::vm::value::Value;

// Instrucciones de 32 bits (Register-Based)
// Formato: [OP:8] [A:8] [B:8] [C:8]
// A: Registro destino
// B: Registro fuente 1 / Constante
// C: Registro fuente 2

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    CargarConstante = 0, // R[A] = Constantes[Bx]
    Mover = 1,           // R[A] = R[B]
    Sumar = 2,           // R[A] = R[B] + R[C]
    Restar = 3,          // R[A] = R[B] - R[C]
    Retornar = 4,        // Return R[A]
    Imprimir = 5,        // Print R[A]
    Menor = 6,           // R[A] = R[B] < R[C]
    Igual = 7,           // R[A] = R[B] == R[C]
    Saltar = 8,          // PC += Bx (Salto relativo hacia adelante)
    SaltarSiFalso = 9,   // Si !R[A], PC += Bx
    SaltarAtras = 10,    // PC -= Bx (Para bucles)
    Llamar = 11,         // R[A] = Call(R[B], Args...)
    Multiplicar = 12,    // R[A] = R[B] * R[C]
    Dividir = 13,        // R[A] = R[B] / R[C]
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

pub struct Chunk {
    pub code: Vec<u32>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, instruction: u32) {
        self.code.push(instruction);
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    // Helpers para codificar instrucciones
    pub fn encode_abc(op: OpCode, a: u8, b: u8, c: u8) -> u32 {
        ((op as u32) << 24) | ((a as u32) << 16) | ((b as u32) << 8) | (c as u32)
    }

    pub fn encode_abx(op: OpCode, a: u8, bx: u16) -> u32 {
        ((op as u32) << 24) | ((a as u32) << 16) | (bx as u32)
    }
}

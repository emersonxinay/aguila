use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

// NaN-Boxing en 64 bits (IEEE 754)
//
// Estructura de un f64:
// S EEEEEEEEEEE MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
// 0 11111111111 1000... (QNaN, Intel)
//
// Usaremos los bits altos para etiquetar tipos cuando es un NaN.
// Mascara QNaN: 0x7FF8000000000000
//
// Tipos:
// - Numero (f64): Cualquier patrón que no sea un NaN "nuestro".
// - Nulo:         QNaN | 0x01
// - Logico:       QNaN | 0x02 | (val << 0)
// - Obj (Ptr):    QNaN | 0x8000000000000000 | Puntero (48 bits)

pub const QNAN: u64 = 0x7ffc000000000000;
pub const SIGN_BIT: u64 = 0x8000000000000000;

pub const TAG_NULO: u64 = 1;
pub const TAG_FALSE: u64 = 2;
pub const TAG_TRUE: u64 = 3;

#[derive(Clone, Copy, PartialEq)]
pub struct Value(pub u64);

impl Value {
    #[inline(always)]
    pub fn numero(n: f64) -> Self {
        Self(n.to_bits())
    }

    #[inline(always)]
    pub fn nulo() -> Self {
        Self(QNAN | TAG_NULO)
    }

    #[inline(always)]
    pub fn logico(b: bool) -> Self {
        if b {
            Self(QNAN | TAG_TRUE)
        } else {
            Self(QNAN | TAG_FALSE)
        }
    }

    #[inline(always)]
    pub fn es_numero(&self) -> bool {
        (self.0 & QNAN) != QNAN
    }

    #[inline(always)]
    pub fn es_nulo(&self) -> bool {
        self.0 == (QNAN | TAG_NULO)
    }

    #[inline(always)]
    pub fn es_logico(&self) -> bool {
        (self.0 | 1) == (QNAN | TAG_TRUE)
    }

    #[inline(always)]
    pub fn a_numero(&self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline(always)]
    pub fn a_logico(&self) -> bool {
        self.0 == (QNAN | TAG_TRUE)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.es_numero() {
            write!(f, "{}", self.a_numero())
        } else if self.es_nulo() {
            write!(f, "nulo")
        } else if self.es_logico() {
            write!(f, "{}", self.a_logico())
        } else {
            write!(f, "<desconocido: {:#x}>", self.0)
        }
    }
}

// Tests unitarios para verificar la magia de bits
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeros() {
        let v = Value::numero(123.456);
        assert!(v.es_numero());
        assert!(!v.es_nulo());
        assert_eq!(v.a_numero(), 123.456);
    }

    #[test]
    fn test_nulo() {
        let v = Value::nulo();
        assert!(v.es_nulo());
        assert!(!v.es_numero());
    }

    #[test]
    fn test_logicos() {
        let v = Value::logico(true);
        assert!(v.es_logico());
        assert!(v.a_logico());

        let v = Value::logico(false);
        assert!(v.es_logico());
        assert!(!v.a_logico());
    }
}

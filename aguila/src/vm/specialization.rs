#![allow(dead_code)]
/// Type Specialization para Operaciones Aritméticas
///
/// Problema: Cada suma chequea tipos en runtime
/// Solución: Detectar patrón de tipo y generar versión especializada
///
/// Si detectamos que una operación siempre recibe enteros,
/// compilamos una versión fast-path que solo suma enteros
/// sin type checking.
use crate::vm::value::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationType {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    LessEq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypePattern {
    IntInt,     // int + int
    FloatFloat, // float + float
    IntFloat,   // int + float (puede variar)
    Unknown,    // Sin pattern claro
}

pub struct SpecializationInfo {
    pub pattern: TypePattern,
    pub pattern_count: u32,
    pub mismatch_count: u32,
}

impl SpecializationInfo {
    pub fn confidence(&self) -> f32 {
        let total = (self.pattern_count + self.mismatch_count) as f32;
        if total == 0.0 {
            0.0
        } else {
            self.pattern_count as f32 / total
        }
    }

    pub fn is_specialized(&self) -> bool {
        self.pattern != TypePattern::Unknown && self.confidence() > 0.95
    }
}

pub struct SpecializationCache {
    // Key: (pc, operation_type)
    // Value: TypePattern info
    cache: HashMap<(usize, OperationType), SpecializationInfo>,
}

impl SpecializationCache {
    pub fn new() -> Self {
        SpecializationCache {
            cache: HashMap::new(),
        }
    }

    /// Registrar acceso para aprender patrón de tipos
    pub fn record(
        &mut self,
        pc: usize,
        op: OperationType,
        left: Value,
        right: Value,
    ) -> TypePattern {
        let entry = self.cache.entry((pc, op)).or_insert(SpecializationInfo {
            pattern: TypePattern::Unknown,
            pattern_count: 0,
            mismatch_count: 0,
        });

        // Detectar patrón
        let observed = if left.es_entero() && right.es_entero() {
            TypePattern::IntInt
        } else if left.es_numero() && right.es_numero() {
            TypePattern::FloatFloat
        } else {
            TypePattern::IntFloat
        };

        // Aprender
        if entry.pattern == TypePattern::Unknown {
            entry.pattern = observed;
            entry.pattern_count = 1;
        } else if entry.pattern == observed {
            entry.pattern_count = entry.pattern_count.saturating_add(1);
        } else {
            entry.mismatch_count = entry.mismatch_count.saturating_add(1);

            // Si hay muchos mismatches, volver a Unknown
            if entry.mismatch_count > 10 {
                entry.pattern = TypePattern::Unknown;
            }
        }

        entry.pattern
    }

    /// Obtener patrón aprendido para una operación
    pub fn get_pattern(&self, pc: usize, op: OperationType) -> Option<TypePattern> {
        self.cache
            .get(&(pc, op))
            .filter(|info| info.is_specialized())
            .map(|info| info.pattern)
    }

    /// Fast-path para suma si está especializado
    #[inline]
    pub fn fast_add(&self, pc: usize, a: i32, b: i32) -> Option<Value> {
        match self.get_pattern(pc, OperationType::Add) {
            Some(TypePattern::IntInt) => {
                // Fast-path: suma entera sin checking
                if let Some(result) = a.checked_add(b) {
                    return Some(Value::entero(result));
                } else {
                    // Overflow, convertir a float
                    return Some(Value::numero(a as f64 + b as f64));
                }
            }
            _ => None,
        }
    }

    pub fn stats(&self) -> String {
        let specialized = self
            .cache
            .values()
            .filter(|info| info.is_specialized())
            .count();
        format!(
            "Specialization: {} operations specialized out of {}",
            specialized,
            self.cache.len()
        )
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_pattern_learning() {
        let mut spec = SpecializationCache::new();

        let int_val = Value::entero(5);
        let int_val2 = Value::entero(3);

        // Simular 100 sumas de enteros
        for _ in 0..100 {
            spec.record(0, OperationType::Add, int_val, int_val2);
        }

        // Debería detectar patrón IntInt
        let pattern = spec.get_pattern(0, OperationType::Add);
        assert_eq!(pattern, Some(TypePattern::IntInt));
    }

    #[test]
    fn test_type_mismatch_resets() {
        let mut spec = SpecializationCache::new();

        let int_val = Value::entero(5);
        let float_val = Value::numero(3.5);

        // Aprender IntInt
        for _ in 0..50 {
            spec.record(0, OperationType::Add, int_val, int_val);
        }

        // Muchos mismatches
        for _ in 0..15 {
            spec.record(0, OperationType::Add, int_val, float_val);
        }

        // Debería resetear a Unknown
        let pattern = spec.get_pattern(0, OperationType::Add);
        assert_eq!(pattern, None);
    }
}

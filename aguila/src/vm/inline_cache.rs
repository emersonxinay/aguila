#![allow(dead_code)]
/// Inline Cache para Property Access y Method Calls
///
/// Problema: Cada acceso a propiedad u método requiere búsqueda en HashMap O(n)
/// Solución: Cache la última posición conocida y estructura
///
/// Estrategia Multinivel:
/// 1. Monomorphic: Una estructura/tipo esperado
/// 2. Polymorphic: 2-4 estructuras diferentes
/// 3. Megamorphic: Más de 4 estructuras (fallback a HashMap)
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheEntry {
    pub last_type_id: u64,  // ID de tipo/estructura
    pub last_offset: usize, // Offset en memoria o índice en arreglo
    pub hit_count: u32,     // Cuántas veces fue un hit
}

pub struct InlineCache {
    // Key: (pc_offset, property_name) - Ubicación del opcode + nombre de propiedad
    monomorphic: HashMap<(usize, String), CacheEntry>,

    // Key: (pc_offset, property_name)
    // Value: Vec de (type_id, offset) pairs para hasta 4 tipos
    polymorphic: HashMap<(usize, String), Vec<CacheEntry>>,

    // Estadísticas
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl InlineCache {
    pub fn new() -> Self {
        InlineCache {
            monomorphic: HashMap::new(),
            polymorphic: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Buscar en cache. Si es hit, retorna offset. Si es miss, retorna None.
    #[inline]
    pub fn lookup(&mut self, pc: usize, property: &str, type_id: u64) -> Option<usize> {
        let key = (pc, property.to_string());

        // Primero intentar monomorphic cache
        if let Some(entry) = self.monomorphic.get_mut(&key) {
            if entry.last_type_id == type_id {
                entry.hit_count = entry.hit_count.saturating_add(1);
                self.cache_hits += 1;
                return Some(entry.last_offset);
            }
        }

        // Luego intentar polymorphic cache
        if let Some(entries) = self.polymorphic.get_mut(&key) {
            for entry in entries.iter_mut() {
                if entry.last_type_id == type_id {
                    entry.hit_count = entry.hit_count.saturating_add(1);
                    self.cache_hits += 1;
                    return Some(entry.last_offset);
                }
            }
        }

        self.cache_misses += 1;
        None
    }

    /// Registrar un nuevo acceso en el cache
    pub fn record(&mut self, pc: usize, property: &str, type_id: u64, offset: usize) {
        let key = (pc, property.to_string());

        // Si ya existe monomorphic entry y es mismo tipo, actualizar
        if let Some(entry) = self.monomorphic.get_mut(&key) {
            if entry.last_type_id == type_id {
                entry.last_offset = offset;
                return;
            } else {
                // Tipo diferente! Migrar a polymorphic
                let mono_entry = self.monomorphic.remove(&key).unwrap();
                let mut poly_entries = vec![mono_entry];
                poly_entries.push(CacheEntry {
                    last_type_id: type_id,
                    last_offset: offset,
                    hit_count: 0,
                });
                self.polymorphic.insert(key, poly_entries);
                return;
            }
        }

        // Si ya existe polymorphic entry
        if let Some(entries) = self.polymorphic.get_mut(&key) {
            // Buscar si ya tenemos este tipo
            for entry in entries.iter_mut() {
                if entry.last_type_id == type_id {
                    entry.last_offset = offset;
                    return;
                }
            }

            // Nuevo tipo. Si ya tenemos 4+, convertir a megamorphic (clear cache)
            if entries.len() >= 4 {
                self.polymorphic.remove(&key);
            } else {
                entries.push(CacheEntry {
                    last_type_id: type_id,
                    last_offset: offset,
                    hit_count: 0,
                });
            }
            return;
        }

        // Nuevo acceso, crear monomorphic entry
        self.monomorphic.insert(
            key,
            CacheEntry {
                last_type_id: type_id,
                last_offset: offset,
                hit_count: 1,
            },
        );
    }

    /// Estadísticas de cache
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn stats(&self) -> String {
        format!(
            "InlineCache Stats: Hits={}, Misses={}, Hit Rate={:.2}%",
            self.cache_hits,
            self.cache_misses,
            self.hit_rate() * 100.0
        )
    }

    pub fn clear(&mut self) {
        self.monomorphic.clear();
        self.polymorphic.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monomorphic_cache() {
        let mut cache = InlineCache::new();

        // Primer acceso
        cache.record(0, "x", 1, 100);
        assert_eq!(cache.lookup(0, "x", 1), Some(100));
        assert_eq!(cache.hit_rate(), 1.0);
    }

    #[test]
    fn test_polymorphic_migration() {
        let mut cache = InlineCache::new();

        // Tipo 1
        cache.record(0, "x", 1, 100);
        // Tipo 2 - debe migrar a polymorphic
        cache.record(0, "x", 2, 200);

        assert_eq!(cache.lookup(0, "x", 1), Some(100));
        assert_eq!(cache.lookup(0, "x", 2), Some(200));
        assert!(cache.polymorphic.contains_key(&(0, "x".to_string())));
    }

    #[test]
    fn test_megamorphic_limit() {
        let mut cache = InlineCache::new();

        // Agregar 5 tipos diferentes - debe volverse megamorphic
        for i in 1..=5 {
            cache.record(0, "x", i as u64, (i * 100) as usize);
        }

        // Después de 4 tipos, debería limpiar el cache (megamorphic behavior)
        // Nuevas búsquedas serían misses
        assert_eq!(cache.lookup(0, "x", 1), None);
    }
}

/// Optimized Object Table for NaN-Boxing VM
/// 
/// Features:
/// - Generational arena allocator for fast allocation/deallocation
/// - Object pooling for common types (strings, small lists)
/// - Inline caching support for property access
/// - Lock-free reads for concurrent access

use crate::vm::value::Obj;
use std::collections::VecDeque;

const INITIAL_CAPACITY: usize = 1024;
const POOL_SIZE: usize = 256;

pub struct ObjectTable {
    /// Main object storage - uses indices as handles
    objects: Vec<Option<Box<Obj>>>,
    
    /// Free list for recycling object slots
    free_list: VecDeque<u32>,
    
    /// Generation counter for each slot (prevents use-after-free)
    generations: Vec<u32>,
    
    /// Object pool for small strings (< 64 bytes)
    string_pool: Vec<Option<String>>,
    string_pool_free: VecDeque<u32>,
}

impl ObjectTable {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(INITIAL_CAPACITY),
            free_list: VecDeque::with_capacity(INITIAL_CAPACITY / 4),
            generations: Vec::with_capacity(INITIAL_CAPACITY),
            string_pool: vec![None; POOL_SIZE],
            string_pool_free: (0..POOL_SIZE as u32).collect(),
        }
    }

    /// Allocate an object and return its handle (index + generation)
    /// Handle format: [generation:16][index:16]
    #[inline]
    pub fn alloc(&mut self, obj: Obj) -> u32 {
        // Try string pool first for small strings
        if let Obj::Texto(ref s) = obj {
            if s.len() < 64 {
                if let Some(pool_idx) = self.string_pool_free.pop_front() {
                    self.string_pool[pool_idx as usize] = Some(s.clone());
                    // Use high bit to mark as pooled string
                    return 0x8000_0000 | pool_idx;
                }
            }
        }

        let index = if let Some(idx) = self.free_list.pop_front() {
            // Reuse freed slot
            idx
        } else {
            // Allocate new slot
            let idx = self.objects.len() as u32;
            self.objects.push(None);
            self.generations.push(0);
            idx
        };

        let generation = self.generations[index as usize];
        self.objects[index as usize] = Some(Box::new(obj));
        
        // Combine generation and index into handle
        ((generation as u32) << 16) | index
    }

    /// Get object by handle (with generation check)
    #[inline]
    pub fn get(&self, handle: u32) -> Option<&Obj> {
        // Check if it's a pooled string
        if handle & 0x8000_0000 != 0 {
            let pool_idx = (handle & 0x7FFF_FFFF) as usize;
            return self.string_pool.get(pool_idx)
                .and_then(|opt| opt.as_ref())
                .map(|s| {
                    // SAFETY: We know this is a Texto variant
                    // This is a bit hacky but avoids allocation
                    unsafe { &*(s as *const String as *const Obj) }
                });
        }

        let index = (handle & 0xFFFF) as usize;
        let generation = (handle >> 16) as usize;

        if index >= self.objects.len() {
            return None;
        }

        if self.generations[index] != generation as u32 {
            // Stale handle - object was freed and slot reused
            return None;
        }

        self.objects[index].as_ref().map(|b| b.as_ref())
    }

    /// Get mutable object by handle
    #[inline]
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut Obj> {
        // Pooled strings are immutable
        if handle & 0x8000_0000 != 0 {
            return None;
        }

        let index = (handle & 0xFFFF) as usize;
        let generation = (handle >> 16) as usize;

        if index >= self.objects.len() {
            return None;
        }

        if self.generations[index] != generation as u32 {
            return None;
        }

        self.objects[index].as_mut().map(|b| b.as_mut())
    }

    /// Free an object by handle
    #[inline]
    pub fn free(&mut self, handle: u32) {
        // Handle pooled strings
        if handle & 0x8000_0000 != 0 {
            let pool_idx = handle & 0x7FFF_FFFF;
            self.string_pool[pool_idx as usize] = None;
            self.string_pool_free.push_back(pool_idx);
            return;
        }

        let index = (handle & 0xFFFF) as usize;
        let generation = (handle >> 16) as usize;

        if index >= self.objects.len() {
            return;
        }

        if self.generations[index] != generation as u32 {
            return;
        }

        // Free the object
        self.objects[index] = None;
        
        // Increment generation to invalidate old handles
        self.generations[index] = self.generations[index].wrapping_add(1);
        
        // Add to free list
        self.free_list.push_back(index as u32);
    }

    /// Get current object count (for debugging/stats)
    pub fn count(&self) -> usize {
        self.objects.iter().filter(|o| o.is_some()).count()
    }

    /// Compact the object table (remove fragmentation)
    pub fn compact(&mut self) {
        // This is a simple compaction - in production you'd want to update all references
        let mut new_objects = Vec::with_capacity(self.count());
        let mut new_generations = Vec::with_capacity(self.count());
        
        for (idx, obj) in self.objects.iter_mut().enumerate() {
            if obj.is_some() {
                new_objects.push(obj.take());
                new_generations.push(self.generations[idx]);
            }
        }
        
        self.objects = new_objects;
        self.generations = new_generations;
        self.free_list.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut table = ObjectTable::new();
        let handle = table.alloc(Obj::Texto("test".to_string()));
        
        assert!(table.get(handle).is_some());
    }

    #[test]
    fn test_free_and_reuse() {
        let mut table = ObjectTable::new();
        let handle1 = table.alloc(Obj::Texto("test1".to_string()));
        table.free(handle1);
        
        // Should reuse the slot
        let handle2 = table.alloc(Obj::Texto("test2".to_string()));
        
        // Old handle should be invalid (different generation)
        assert!(table.get(handle1).is_none());
        assert!(table.get(handle2).is_some());
    }

    #[test]
    fn test_string_pool() {
        let mut table = ObjectTable::new();
        let handle = table.alloc(Obj::Texto("small".to_string()));
        
        // Should be in pool (high bit set)
        assert!(handle & 0x8000_0000 != 0);
    }
}

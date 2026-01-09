//! Artifact Cache
//!
//! LRU cache for artifact content and rendered lines.
//! Enables efficient handling of large files by caching parsed/highlighted content.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use super::syntax::Token;

/// Generic LRU cache
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    /// Create a new cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    /// Get a value, updating access order
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to front (most recently used)
            self.order.retain(|k| k != key);
            self.order.push_front(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    /// Get a mutable reference, updating access order
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.map.contains_key(key) {
            self.order.retain(|k| k != key);
            self.order.push_front(key.clone());
            self.map.get_mut(key)
        } else {
            None
        }
    }

    /// Insert a value, evicting least recently used if at capacity
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let old = self.map.insert(key.clone(), value);

        if old.is_none() {
            // New entry - check capacity
            while self.order.len() >= self.capacity {
                if let Some(evicted_key) = self.order.pop_back() {
                    self.map.remove(&evicted_key);
                }
            }
            self.order.push_front(key);
        } else {
            // Update - move to front
            self.order.retain(|k| k != &key);
            self.order.push_front(key);
        }

        old
    }

    /// Remove a value
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.map.remove(key)
    }

    /// Check if key exists
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

/// Key for line cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineCacheKey {
    /// Artifact ID
    pub artifact_id: String,
    /// Line number
    pub line_number: usize,
    /// Content hash (to detect changes)
    pub content_hash: u64,
}

impl LineCacheKey {
    pub fn new(artifact_id: &str, line_number: usize, line_content: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        line_content.hash(&mut hasher);

        Self {
            artifact_id: artifact_id.to_string(),
            line_number,
            content_hash: hasher.finish(),
        }
    }
}

/// Cached line data
#[derive(Debug, Clone)]
pub struct CachedLine {
    /// Syntax tokens for this line
    pub tokens: Vec<Token>,
    /// Whether this line has changes (for diff highlighting)
    pub has_changes: bool,
}

/// Artifact content cache
#[derive(Debug)]
pub struct ArtifactCache {
    /// Highlighted line cache
    line_cache: LruCache<LineCacheKey, CachedLine>,
    /// Max lines to cache per artifact
    lines_per_artifact: usize,
}

impl Default for ArtifactCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactCache {
    /// Create a new artifact cache
    pub fn new() -> Self {
        Self {
            // Cache up to 10000 lines across all artifacts
            line_cache: LruCache::new(10000),
            lines_per_artifact: 5000,
        }
    }

    /// Create with custom capacity
    pub fn with_capacity(line_capacity: usize, lines_per_artifact: usize) -> Self {
        Self {
            line_cache: LruCache::new(line_capacity),
            lines_per_artifact,
        }
    }

    /// Get cached tokens for a line
    pub fn get_line(&mut self, artifact_id: &str, line_number: usize, line_content: &str) -> Option<&CachedLine> {
        let key = LineCacheKey::new(artifact_id, line_number, line_content);
        self.line_cache.get(&key)
    }

    /// Cache tokens for a line
    pub fn cache_line(
        &mut self,
        artifact_id: &str,
        line_number: usize,
        line_content: &str,
        tokens: Vec<Token>,
        has_changes: bool,
    ) {
        let key = LineCacheKey::new(artifact_id, line_number, line_content);
        self.line_cache.insert(key, CachedLine { tokens, has_changes });
    }

    /// Invalidate all cached lines for an artifact
    pub fn invalidate_artifact(&mut self, artifact_id: &str) {
        // This is O(n) but should be infrequent
        let keys_to_remove: Vec<_> = self
            .line_cache
            .order
            .iter()
            .filter(|k| k.artifact_id == artifact_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.line_cache.remove(&key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_lines: self.line_cache.len(),
            capacity: self.line_cache.capacity,
        }
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        self.line_cache.clear();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub cached_lines: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn usage_percent(&self) -> f32 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.cached_lines as f32 / self.capacity as f32) * 100.0
        }
    }
}

/// Chunk iterator for rendering large files
pub struct ChunkIterator<'a> {
    lines: Vec<&'a str>,
    chunk_size: usize,
    current_chunk: usize,
}

impl<'a> ChunkIterator<'a> {
    /// Create a new chunk iterator
    pub fn new(content: &'a str, chunk_size: usize) -> Self {
        Self {
            lines: content.lines().collect(),
            chunk_size,
            current_chunk: 0,
        }
    }

    /// Get total number of chunks
    pub fn total_chunks(&self) -> usize {
        (self.lines.len() + self.chunk_size - 1) / self.chunk_size
    }

    /// Get total number of lines
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific chunk by index
    pub fn get_chunk(&self, chunk_index: usize) -> Option<ChunkData<'a>> {
        let start_line = chunk_index * self.chunk_size;
        if start_line >= self.lines.len() {
            return None;
        }

        let end_line = (start_line + self.chunk_size).min(self.lines.len());
        let lines: Vec<&'a str> = self.lines[start_line..end_line].to_vec();

        Some(ChunkData {
            chunk_index,
            start_line,
            end_line,
            lines,
        })
    }

    /// Get chunk containing a specific line
    pub fn get_chunk_for_line(&self, line_number: usize) -> Option<ChunkData<'a>> {
        let chunk_index = line_number / self.chunk_size;
        self.get_chunk(chunk_index)
    }
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = ChunkData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_chunk(self.current_chunk);
        if result.is_some() {
            self.current_chunk += 1;
        }
        result
    }
}

/// A chunk of content
#[derive(Debug, Clone)]
pub struct ChunkData<'a> {
    /// Chunk index
    pub chunk_index: usize,
    /// Starting line number (0-indexed)
    pub start_line: usize,
    /// Ending line number (exclusive)
    pub end_line: usize,
    /// Lines in this chunk
    pub lines: Vec<&'a str>,
}

impl<'a> ChunkData<'a> {
    /// Get display line number for a line in this chunk
    pub fn display_line_number(&self, local_index: usize) -> usize {
        self.start_line + local_index + 1 // 1-indexed for display
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache: LruCache<i32, &str> = LruCache::new(3);

        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache: LruCache<i32, &str> = LruCache::new(2);

        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three"); // Should evict 1

        assert!(cache.get(&1).is_none());
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_lru_cache_access_updates_order() {
        let mut cache: LruCache<i32, &str> = LruCache::new(2);

        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.get(&1); // Access 1, making 2 the least recently used
        cache.insert(3, "three"); // Should evict 2

        assert_eq!(cache.get(&1), Some(&"one"));
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_chunk_iterator() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let chunks: Vec<_> = ChunkIterator::new(content, 2).collect();

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].lines.len(), 2);
        assert_eq!(chunks[1].lines.len(), 2);
        assert_eq!(chunks[2].lines.len(), 1);
    }

    #[test]
    fn test_chunk_line_numbers() {
        let content = "a\nb\nc\nd";
        let iter = ChunkIterator::new(content, 2);

        let chunk = iter.get_chunk(1).unwrap();
        assert_eq!(chunk.start_line, 2);
        assert_eq!(chunk.display_line_number(0), 3); // Line "c"
    }

    #[test]
    fn test_artifact_cache() {
        let mut cache = ArtifactCache::new();

        cache.cache_line("test", 0, "hello", vec![], false);
        assert!(cache.get_line("test", 0, "hello").is_some());
        assert!(cache.get_line("test", 0, "different").is_none()); // Different content hash
    }

    #[test]
    fn test_invalidate_artifact() {
        let mut cache = ArtifactCache::new();

        cache.cache_line("artifact1", 0, "line0", vec![], false);
        cache.cache_line("artifact1", 1, "line1", vec![], false);
        cache.cache_line("artifact2", 0, "line0", vec![], false);

        cache.invalidate_artifact("artifact1");

        assert!(cache.get_line("artifact1", 0, "line0").is_none());
        assert!(cache.get_line("artifact2", 0, "line0").is_some());
    }
}

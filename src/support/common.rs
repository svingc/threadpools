// SPDX-License-Identifier: MIT

/// A struct for splitting a range of items into evenly distributed chunks.
pub struct ChunkSplitter {
    /// Total number of items to be split into chunks.
    total_items: usize,
    /// The current position in the range of items.
    current_position: usize,
    /// The index of the current chunk being processed.
    current_chunk_index: usize,
    /// The base size of each chunk (without considering extra items).
    base_chunk_size: usize,
    /// The number of extra items to distribute across the first few chunks.
    extra_items: usize,
}

impl ChunkSplitter {
    /// Creates a new `ChunkSplitter` to divide `total_items` into `num_chunks`.
    ///
    /// # Arguments
    /// - `total_items`: The total number of items to be split.
    /// - `num_chunks`: The number of chunks to split the items into.
    ///
    /// # Returns
    /// A new `ChunkSplitter` instance.
    pub fn new(total_items: usize, num_chunks: usize) -> Self {
        ChunkSplitter {
            total_items,
            current_position: 0,
            current_chunk_index: 0,
            // Each chunk gets at least this many items.
            base_chunk_size: total_items / num_chunks,
            // Remaining items to distribute across the first few chunks.
            extra_items: total_items % num_chunks,
        }
    }
}

impl Iterator for ChunkSplitter {
    type Item = (usize, usize);

    /// Produces the next chunk as a tuple `(start, end)` representing the
    /// indices.
    ///
    /// # Returns
    /// - `Some((start, end))` if there are more chunks to process.
    /// - `None` when all chunks have been processed.
    fn next(&mut self) -> Option<Self::Item> {
        // The starting index of the current chunk.
        let start = self.current_position;

        // If we've reached the total number of items, iteration is complete.
        if start == self.total_items {
            return None;
        }

        // Calculate the ending index of the current chunk.
        let end = start
            + self.base_chunk_size
            + (self.current_chunk_index < self.extra_items) as usize;

        // Update for the next iteration.
        self.current_chunk_index += 1;
        self.current_position = end;

        // Return the current chunk as a tuple of indices.
        Some((start, end))
    }
}

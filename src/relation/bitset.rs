use std::rc::Rc;

/// Bitset, mapping indices to boolean flags
///
/// Model:
///
/// ```plain
///        | <-------  word  --------> | <-------  word  --------> | ...
/// (mask) | 1<<0 1<<1 ... 1<<62 1<<63 | 1<<0 1<<1 ... 1<<62 1<<63 | ...
/// ```
#[derive(Clone, Debug, Default)]
pub struct OffsetBitSet {
    /// First word index, e.g. `offset = 3` indicates the `id` of the first bit is `3 * 64 = 192`
    offset: usize,
    /// Bit flags
    bits: Vec<u64>,
}

impl OffsetBitSet {
    /// Creates an empty [`OffsetBitSet`].
    pub fn new() -> Self {
        Self::default()
    }

    // Returns the word index corresponding to the given bit id.
    // #[inline]
    // fn word_index(&self, id: usize) -> usize {
    //     (id >> 6) - self.offset
    // }

    /// Returns the bit mask corresponding to the given bit id within its word.
    #[inline]
    fn bit_mask(id: usize) -> u64 {
        1 << (id & 63)
    }

    /// Inserts the given id into the bitset.
    ///
    /// The bitset is expanded if the id is outside the current range.
    /// If the id is before the current offset, the internal storage is shifted
    /// to include the new word range.
    pub fn insert(&mut self, id: usize) {
        let word = id >> 6;
        if self.bits.is_empty() {
            self.offset = word;
            self.bits.push(0);
        }

        if word < self.offset {
            let word_shift = self.offset - word;

            let mut new_bits = vec![0; word_shift + self.bits.len()];

            new_bits[word_shift..].copy_from_slice(&self.bits);

            self.bits = new_bits;
            self.offset = word;
        }

        let index = word - self.offset;

        if index >= self.bits.len() {
            self.bits.resize(index + 1, 0);
        }

        self.bits[index] |= Self::bit_mask(id);
    }

    /// Removes the given id from the bitset.
    ///
    /// Returns `true` if the bit was set before removal, or `false` if the id
    /// was not present.
    pub fn take(&mut self, id: usize) -> bool {
        let word = id >> 6;
        if word < self.offset {
            return false;
        }

        let index = word - self.offset;
        if index >= self.bits.len() {
            return false;
        }

        let mask = Self::bit_mask(id);
        let existed = self.bits[index] & mask != 0;
        self.bits[index] &= !mask;

        existed
    }

    /// Checks whether the given id is set in the bitset.
    pub fn contains(&self, id: usize) -> bool {
        let word = id >> 6;
        if word < self.offset {
            return false;
        }

        let index = word - self.offset;
        if index >= self.bits.len() {
            return false;
        }

        self.bits[index] & Self::bit_mask(id) != 0
    }

    /// Merges all bits from `other` into this bitset.
    ///
    /// The resulting bitset contains every bit that was set in either `self` or
    /// `other`. The internal storage is expanded when necessary.
    pub fn union_with(&mut self, other: &Self) {
        if other.bits.is_empty() {
            return;
        }
        if self.bits.is_empty() {
            *self = other.clone();
            return;
        }

        let self_end = self.offset + self.bits.len();
        let other_end = other.offset + other.bits.len();

        // `other` is fully contained inside `self`
        if self.offset <= other.offset && self_end >= other_end {
            let offset = other.offset - self.offset;
            for (i, &word) in other.bits.iter().enumerate() {
                self.bits[offset + i] |= word;
            }
            return;
        }

        let new_offset = self.offset.min(other.offset);
        let new_end = self_end.max(other_end);

        let mut new_bits = vec![0; new_end - new_offset];

        {
            let offset = self.offset - new_offset;
            for (i, &word) in self.bits.iter().enumerate() {
                new_bits[offset + i] |= word;
            }
        }

        {
            let offset = other.offset - new_offset;
            for (i, &word) in other.bits.iter().enumerate() {
                new_bits[offset + i] |= word;
            }
        }

        self.offset = new_offset;
        self.bits = new_bits;
    }

    /// Removes all bits from `self` that are also set in `other`.
    ///
    /// The resulting bitset contains `self \ other`.
    pub fn difference_with(&mut self, other: &Self) {
        if self.bits.is_empty() || other.bits.is_empty() {
            return;
        }

        let self_end = self.offset + self.bits.len();
        let other_end = other.offset + other.bits.len();

        // No overlap between the two ranges.
        if self_end <= other.offset || other_end <= self.offset {
            return;
        }

        let start = self.offset.max(other.offset);
        let end = self_end.min(other_end);

        let self_offset = start - self.offset;
        let other_offset = start - other.offset;

        for i in 0..(end - start) {
            self.bits[self_offset + i] &= !other.bits[other_offset + i];
        }
    }

    /// Returns an iterator over all set bit ids in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits
            .iter()
            .enumerate()
            .flat_map(move |(word_index, &word)| BitIter {
                word,
                base: (self.offset + word_index) << 6,
            })
    }

    /// Make `Rc<OffsetBitSet>` into an iterator over all set bit ids in ascending order.
    pub fn into_iter(self: Rc<Self>) -> OffsetBitSetIter {
        OffsetBitSetIter {
            set: self,
            word_index: 0,
            current: BitIter { word: 0, base: 0 },
        }
    }

    /// Removes unused leading and trailing zero words from the bitset.
    ///
    /// After cleanup, `offset` points to the first word that contains a set bit,
    /// and the internal storage contains no redundant zero words at either end.
    pub fn cleanup(&mut self) {
        // remove trailing zero words
        while self.bits.last() == Some(&0) {
            self.bits.pop();
        }

        // remove leading zero words
        let leading = self.bits.iter().take_while(|&&x| x == 0).count();
        if leading > 0 {
            self.bits.drain(..leading);
            self.offset += leading;
        }
    }

    /// Trims the bitset to the given bit-id range `[start, end)`.
    ///
    /// Bits outside the range are removed. The internal storage is also adjusted
    /// so that its range only covers the specified interval.
    ///
    /// If `start >= end`, the bitset becomes empty.
    pub fn trim(&mut self, start: usize, end: usize) {
        if self.bits.is_empty() || start >= end {
            self.bits.clear();
            self.offset = 0;
            return;
        }

        let start_word = start >> 6;
        let end_word = (end - 1) >> 6;

        let self_end = self.offset + self.bits.len();

        // No overlap with [start_word, end_word].
        if end_word < self.offset || start_word >= self_end {
            self.bits.clear();
            self.offset = 0;
            return;
        }

        let new_offset = self.offset.max(start_word);
        let new_end = self_end.min(end_word + 1);

        let begin = new_offset - self.offset;
        let len = new_end - new_offset;

        // Keep only the overlapping words.
        self.bits = self.bits[begin..begin + len].to_vec();
        self.offset = new_offset;

        // Clear bits before `start`.
        if start_word == self.offset {
            let bit = start & 63;
            if bit != 0 {
                self.bits[0] &= u64::MAX << bit;
            }
        }

        // Clear bits at or after `end`.
        if end_word == self.offset + self.bits.len() - 1 {
            let bit = end & 63;
            if bit != 0 {
                let last = self.bits.len() - 1;
                self.bits[last] &= (1u64 << bit) - 1;
            }
        }
    }

    /// Get the range of word index
    pub fn range(&self) -> (usize, usize) {
        (self.offset, self.offset + self.bits.len())
    }
}

struct BitIter {
    word: u64,
    base: usize,
}

impl Iterator for BitIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word == 0 {
            return None;
        }

        let bit = self.word.trailing_zeros() as usize;
        self.word &= self.word - 1;

        Some(self.base + bit)
    }
}

pub struct OffsetBitSetIter {
    set: Rc<OffsetBitSet>,
    word_index: usize,
    current: BitIter,
}

impl Iterator for OffsetBitSetIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(value) = self.current.next() {
                return Some(value);
            }

            let word = *self.set.bits.get(self.word_index)?;
            self.current = BitIter {
                word,
                base: (self.set.offset + self.word_index) << 6,
            };

            self.word_index += 1;
        }
    }
}

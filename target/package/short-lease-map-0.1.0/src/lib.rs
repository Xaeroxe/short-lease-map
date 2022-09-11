use std::time::{Duration, Instant};

/// A HashMap like collection, but optimized for really short term internship.
///
/// It's easiest to think of this like a hotel. When you check in, a room number
/// is assigned to you. When you leave, that room can now be assigned to someone else.
#[derive(Clone, Debug)]
pub struct ShortLeaseMap<T>(Vec<Option<(T, Instant)>>);

impl<T> ShortLeaseMap<T> {
    /// Creates a new ShortLeaseMap with zero capacity. Capacity will grow as items are added.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new ShortLeaseMap with space reserved for `size` entries.
    pub fn with_capacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }

    /// Adds a value to the map. The value returned can later be used to retrieve it. The returned
    /// key is not guaranteed to be unique once the value has been removed from this map.
    pub fn insert(&mut self, t: T) -> usize {
        let idx = self.0.iter_mut().enumerate().find(|(_, v)| v.is_none());
        match idx {
            None => {
                let idx = self.0.len();
                self.0.push(Some((t, Instant::now())));
                idx
            }
            Some((i, v)) => {
                *v = Some((t, Instant::now()));
                i
            }
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        Option::flatten(self.0.get(idx).map(Option::as_ref)).map(|o| &o.0)
    }

    /// Removes the value with this index. The index may be assigned again after it has been removed.
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.0.get_mut(idx).and_then(Option::take).map(|o| o.0)
    }

    /// Evict guests which have overstayed their welcome. If a value has been in the map longer than
    /// the `max_age` given, it will be dropped. Returns a count of how many items were removed.
    ///
    // Clippy will suggest we simplify this code by using Iterator::flatten. It is wrong, the code
    // is not able to change the iterated value to `None` while using Iterator::flatten.
    #[allow(clippy::manual_flatten)]
    pub fn dump_old_values(&mut self, max_age: Duration) -> usize {
        let mut total_dumped = 0;
        for e in &mut self.0 {
            if let Some((_, insert_time)) = e {
                if insert_time.elapsed() > max_age {
                    *e = None;
                    total_dumped += 1;
                }
            }
        }
        total_dumped
    }

    /// Iterates immutably over the collection, returning a tuple of a reference to the item and its
    /// ID value.
    pub fn iter(&self) -> impl Iterator<Item = (&T, usize)> + DoubleEndedIterator {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, e)| (e.as_ref().map(|o| (&o.0, i))))
    }

    /// Iterates mutably over the collection, returning a tuple of a mutable reference to the item
    /// and its ID value.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut T, usize)> + DoubleEndedIterator {
        self.0
            .iter_mut()
            .enumerate()
            .filter_map(|(i, e)| e.as_mut().map(|o| (&mut o.0, i)))
    }
}

impl<T> Default for ShortLeaseMap<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_lease_map() {
        const CAPACITY: usize = 10;
        let mut map = ShortLeaseMap::with_capacity(CAPACITY);
        assert_eq!(map.0.capacity(), CAPACITY);
        for i in 0..CAPACITY + 1 {
            assert_eq!(map.insert(i), i);
        }
        assert_eq!(map.remove(3), Some(3));
        assert_eq!(map.insert(0), 3);
        assert_eq!(map.insert(5), CAPACITY + 1);
        assert_eq!(map.remove(3), Some(0));
        assert_eq!(map.insert(0), 3);
    }
}

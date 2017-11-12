#![deny(missing_docs)]
#![allow(unknown_lints)]

//! A map-esque data structure that small integer keys for you on insertion.
//! Key of removed entries are reused for new insertions.
//! Underlying data is stored in a vector, keys are just indexes of that vector.
//! The main trick is keeping in-place linked list of freed indexes for reuse.
//!
//! Serde is supported. If you need pre-computed length at serialization time
//! (for example, for bincode), use `serde_ser_len` feature.


#[cfg(test)]
mod test;

use std::mem;
use std::usize;
use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::Ordering;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::slice;
use std::vec;
use std::fmt;
use std::clone::Clone;

const SENTINEL: usize = usize::MAX;

#[derive(Clone)]
enum Entry<V> {
    Empty(usize),
    Occupied(V),
}

impl<V> Entry<V> {
    fn is_empty(&self) -> bool {
        match *self {
            Entry::Empty(_) => true,
            _ => false,
        }
    }
}

/// A map that chooses small integer keys for you.
/// You store something into this map and then access it by ID returned by it.
/// For small V entries are expected to take 16 bytes.
///
/// Example:
///
/// ```
/// use compactmap::CompactMap;
///
/// let mut mymap : CompactMap<String> = CompactMap::new();
/// let id_qwerty = mymap.insert("qwerty".to_string());
/// let id_qwertz = mymap.insert("qwertz".to_string());
/// assert_eq!(mymap[id_qwerty], "qwerty");
/// for (id, val) in mymap {
///     println!("{}:{}", id, val);
/// }
/// ```
#[derive(Clone)]
pub struct CompactMap<V> {
    data: Vec<Entry<V>>,
    free_head: usize,
}

impl<V> CompactMap<V> {
    /// Creates an empty `CompactMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    /// let mut map: CompactMap<String> = CompactMap::new();
    /// ```
    pub fn new() -> CompactMap<V> {
        CompactMap {
            data: vec![],
            free_head: SENTINEL,
        }
    }

    /// Creates an empty `CompactMap` with space for at least `capacity`
    /// elements before resizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    /// let mut map: CompactMap<String> = CompactMap::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        CompactMap {
            data: Vec::with_capacity(capacity),
            free_head: SENTINEL,
        }
    }

    /// Returns capacity of the underlying vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves capacity for `CompactMap`'s underlying vector.
    /// If you just cleared M elements from the map and want to insert N
    /// more elements, you'll probably need to reserve N-M elements.
    pub fn reserve(&mut self, len: usize) {
        self.data.reserve(len);
    }

    /// Reserves capacity for `CompactMap`'s underlying vector.
    /// If you just cleared M elements from the map and want to insert N
    /// more elements, you'll probably need to reserve N-M elements.
    pub fn reserve_exact(&mut self, len: usize) {
        self.data.reserve_exact(len);
    }

    // TODO: entry
    // TODO: DoubleEndedIterator

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    ///
    /// let mut a = CompactMap::new();
    /// a.insert("a");
    /// a.clear();
    /// assert!(a.is_empty_slow());
    /// ```
    pub fn clear(&mut self) {
        self.free_head = SENTINEL;
        self.data.clear();
    }

    /// Iterating the map to check if it is empty.
    /// O(n) where n is historical maximum element count.
    pub fn is_empty_slow(&self) -> bool {
        self.len_slow() == 0
    }

    /// Inserts a value into the map. The map generates and returns ID of
    /// the inserted element.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    ///
    /// let mut map = CompactMap::new();
    /// assert_eq!(map.is_empty_slow(), true);
    /// assert_eq!(map.insert(37), 0);
    /// assert_eq!(map.is_empty_slow(), false);
    ///
    /// assert_eq!(map.insert(37), 1);
    /// assert_eq!(map.insert(37), 2);
    /// assert_eq!(map.insert(44), 3);
    /// assert_eq!(map.len_slow(), 4);
    /// ```
    pub fn insert(&mut self, v: V) -> usize {
        let head = self.free_head;
        let entry = Entry::Occupied(v);
        if head == SENTINEL {
            self.data.push(entry);
            self.data.len() - 1
        } else {
            match mem::replace(&mut self.data[head], entry) {
                Entry::Empty(next) => {
                    self.free_head = next;
                    head
                }
                Entry::Occupied(_) => unreachable!(),
            }
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    /// ```
    /// use compactmap::CompactMap;
    ///
    /// let mut map = CompactMap::new();
    /// let id = map.insert("a");
    /// assert_eq!(map.remove(id), Some("a"));
    /// assert_eq!(map.remove(123), None);
    /// ```
    pub fn remove(&mut self, i: usize) -> Option<V> {
        if i >= self.data.len() {
            return None;
        }
        if let Entry::Empty(_) = self.data[i] {
            // Early return to avoid further wrong mem::replace
            return None;
        }

        let empty_entry = Entry::Empty(self.free_head);
        if let Entry::Occupied(v) = mem::replace(&mut self.data[i], empty_entry) {
            if i == self.data.len() - 1 {
                self.data.truncate(i);
            } else {
                self.free_head = i;
            }
            Some(v)
        } else {
            unreachable!();
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, i: usize) -> Option<&V> {
        self.data.get(i).and_then(|entry| match *entry {
            Entry::Empty(_) => None,
            Entry::Occupied(ref v) => Some(v),
        })
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, i: usize) -> Option<&mut V> {
        self.data.get_mut(i).and_then(|entry| match *entry {
            Entry::Empty(_) => None,
            Entry::Occupied(ref mut v) => Some(v),
        })
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order.
    /// The iterator's element type is `(usize, &'r V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    ///
    /// let mut map = CompactMap::new();
    /// map.insert("a");
    /// map.insert("c");
    /// map.insert("b");
    ///
    /// // Print `1: a`, `2: b` and `3: c`.
    /// for (key, value) in map.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<V> {
        Iter {
            iter: self.data.iter(),
            counter: 0,
        }
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order,
    /// with mutable references to the values.
    /// The iterator's element type is `(usize, &'r mut V)`
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut {
            iter: self.data.iter_mut(),
            counter: 0,
        }
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order,
    /// the keys, consuming the original `CompactMap`.
    /// The iterator's element type is `(usize, V)`.
    pub fn into_iter(self) -> IntoIter<V> {
        IntoIter {
            iter: self.data.into_iter(),
            counter: 0,
        }
    }

    /// Returns an iterator visiting all keys in some order.
    /// The iterator's element type is `usize`.
    pub fn keys(&self) -> Keys<V> {
        Keys { iter: self.iter() }
    }
    /// Returns an iterator visiting all values in ascending order of the keys.
    /// The iterator's element type is `&'r V`.
    pub fn values(&self) -> Values<V> {
        Values { iter: self.iter() }
    }
    /// Returns an iterator visiting all values in ascending order of the keys.
    /// The iterator's element type is `&'r mut V`.
    pub fn values_mut(&mut self) -> ValuesMut<V> {
        ValuesMut { iter_mut: self.iter_mut() }
    }

    /// Iterates the map to get number of elements.
    /// O(n) where n is historical maximum element count.
    pub fn len_slow(&self) -> usize {
        self.iter().count()
    }

    /// Trims the `CompactMap` of any excess capacity.
    ///
    /// Rescans the whole map to reindex empty slots. O(n).
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    /// let mut map: CompactMap<&str> = CompactMap::with_capacity(10);
    /// assert_eq!(map.capacity(), 10);
    /// map.shrink_to_fit();
    /// assert_eq!(map.capacity(), 0);
    /// map.insert("qwe");
    /// map.insert("345");
    /// map.insert("555");
    /// map.remove(1);
    /// map.shrink_to_fit();
    /// assert_eq!(map.capacity(), 2);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        // strip off trailing `Empty`s
        if let Some(idx) = self.data.iter().rposition(Entry::is_empty) {
            self.data.truncate(idx + 1);
        } else {
            self.data.clear();
        };

        self.data.shrink_to_fit();
        self.reindex();
    }

    /// Returns an iterator visiting all key-value pairs in ascending order of
    /// the keys, emptying (but not consuming) the original `CompactMap`.
    /// The iterator's element type is `(usize, V)`. Keeps the allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use compactmap::CompactMap;
    ///
    /// let mut map = CompactMap::new();
    /// map.insert("a");
    /// map.insert("b");
    /// map.insert("c");
    /// map.remove(1);
    ///
    /// let vec: Vec<(usize, &str)> = map.drain().collect();
    ///
    /// assert_eq!(vec, [(0, "a"), (2, "c")]);
    /// assert!(map.is_empty_slow());
    /// assert!(map.capacity() > 0);
    /// ```
    pub fn drain(&mut self) -> Drain<V> {
        fn filter<A>((i, v): (usize, Entry<A>)) -> Option<(usize, A)> {
            match v {
                Entry::Empty(_) => None,
                Entry::Occupied(x) => Some((i,x)),
            }
        }
        let filter: fn((usize, Entry<V>)) -> Option<(usize, V)> = filter; // coerce to fn ptr

        self.free_head = SENTINEL;
        Drain { iter: self.data.drain(..).enumerate().filter_map(filter) }
    }

    fn reindex(&mut self) {
        self.free_head = SENTINEL;
        for i in 0..self.data.len() {
            if let Entry::Empty(ref mut head) = self.data[i] {
                *head = self.free_head;
                self.free_head = i;
            }
        }
    }
}

impl<V> Default for CompactMap<V> {
    fn default() -> CompactMap<V> {
        CompactMap::new()
    }
}


impl<V> Hash for CompactMap<V>
where
    V: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for i in 0..(self.data.len()) {
            if let Entry::Occupied(ref j) = self.data[i] {
                state.write_usize(i);
                j.hash(state);
            }
        }
    }
}

// [Partial]Eq impls are based on onces from VecMap

impl<V: PartialEq> PartialEq for CompactMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

macro_rules! iterate_for_ord_and_eq {
    ($self_:ident, $other:expr, $greater:expr, $less:expr, $j:ident, $k:ident, both_found $code:block) => {
        for i in 0..($self_.data.len()) {
            if let Entry::Occupied(ref $j) = $self_.data[i] {
                if i >= $other.data.len() {
                    return $greater;
                }
                if let Entry::Occupied(ref $k) = $other.data[i] {
                    $code
                } else {
                    return $greater
                }
            } else {
                if i >= $other.data.len() {
                    continue;
                }
                if let Entry::Occupied(_) = $other.data[i] {
                    return $less
                } 
            }
        }
        for i in ($self_.data.len())..($other.data.len()) {
            if let Entry::Occupied(_) = $other.data[i] {
                return $less;
            }
        }
    }
}

impl<V: Eq> Eq for CompactMap<V> {}

// We are greater then them iif { { we have i'th slot
// filled in and they don't } or { data in i'th slot compares
// "greater" to our data } } and filledness status and contained data
// prior to i is the same.
impl<V> PartialOrd<CompactMap<V>> for CompactMap<V>
where
    V: PartialOrd<V>,
{
    fn partial_cmp(&self, other: &CompactMap<V>) -> Option<Ordering> {
        iterate_for_ord_and_eq!(self, other,
                                Some(Ordering::Greater), Some(Ordering::Less),
                                j, k, 
            both_found {
                let o = k.partial_cmp(j);
                if o == Some(Ordering::Equal) {
                    continue;
                }
                return o;
            });
        Some(Ordering::Equal)
    }
}

impl<V> Ord for CompactMap<V>
where
    V: Ord,
{
    fn cmp(&self, other: &CompactMap<V>) -> Ordering {
        iterate_for_ord_and_eq!(self, other,
                                Ordering::Greater, Ordering::Less,
                                j, k, 
            both_found {
                let o = k.cmp(j);
                if o == Ordering::Equal {
                    continue;
                }
                return o;
            });
        Ordering::Equal
    }
}

impl<V> FromIterator<V> for CompactMap<V> {
    fn from_iter<I>(iter: I) -> CompactMap<V>
    where
        I: IntoIterator<Item = V>,
    {
        let mut c = CompactMap::new();
        // TODO size hint here maybe
        for i in iter {
            c.insert(i);
        }
        c
    }
}

impl<'a, V> FromIterator<&'a V> for CompactMap<V>
where
    V: Copy,
{
    #[allow(map_clone)]
    fn from_iter<I>(iter: I) -> CompactMap<V>
    where
        I: IntoIterator<Item = &'a V>,
    {
        FromIterator::<V>::from_iter(iter.into_iter().map(|&value| value))
    }
}

impl<V> Extend<V> for CompactMap<V> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = V>,
    {
        // TODO: maybe use size hint here
        for i in iter {
            self.insert(i);
        }
    }
}
impl<'a, V> Extend<&'a V> for CompactMap<V>
where
    V: Copy,
{
    #[allow(map_clone)]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a V>,
    {
        self.extend(iter.into_iter().map(|&value| value));
    }
}

// Debug, Index and IntexMut mostly borrowed from VecMap
impl<V> Index<usize> for CompactMap<V> {
    type Output = V;
    #[inline]
    fn index(&self, i: usize) -> &V {
        self.get(i).expect("key not present")
    }
}
impl<'a, V> Index<&'a usize> for CompactMap<V> {
    type Output = V;
    fn index(&self, i: &usize) -> &V {
        self.get(*i).expect("key not present")
    }
}
impl<V> IndexMut<usize> for CompactMap<V> {
    fn index_mut(&mut self, i: usize) -> &mut V {
        self.get_mut(i).expect("key not present")
    }
}
impl<'a, V> IndexMut<&'a usize> for CompactMap<V> {
    fn index_mut(&mut self, i: &usize) -> &mut V {
        self.get_mut(*i).expect("key not present")
    }
}
impl<V: fmt::Debug> fmt::Debug for CompactMap<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}


macro_rules! generate_iterator {
    ($self_:ident, mut) => {
        generate_iterator!($self_ ; & mut Entry::Occupied(ref mut x), x);
    };
    ($self_:ident, const) => {
        generate_iterator!($self_ ; &     Entry::Occupied(ref     x), x);
    };
    ($self_:ident, plain) => {
        generate_iterator!($self_ ;       Entry::Occupied(        x), x);
    };
    ($self_:ident ; $pp:pat, $x:ident) => {
        loop {
            let e = $self_.iter.next();
            $self_.counter+=1;
            if let Some(a) = e {
                if let $pp = a {
                    return Some(($self_.counter-1, $x));
                }
            } else {
                return None;
            }
        }
    };
}

/// An iterator over the key-value pairs of a map.
pub struct Iter<'a, V: 'a> {
    iter: slice::Iter<'a, Entry<V>>,
    counter: usize,
}
// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Iter<'a, V> {
    fn clone(&self) -> Iter<'a, V> {
        Iter {
            iter: self.iter.clone(),
            counter: self.counter,
        }
    }
}
impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (usize, &'a V);

    #[allow(match_ref_pats)]
    fn next(&mut self) -> Option<(usize, &'a V)> {
        generate_iterator!(self, const);
    }
}
impl<'a, V> IntoIterator for &'a CompactMap<V> {
    type Item = (usize, &'a V);
    type IntoIter = Iter<'a, V>;
    fn into_iter(self) -> Iter<'a, V> {
        Iter {
            iter: self.data.iter(),
            counter: 0,
        }
    }
}

/// An iterator over the key-value pairs of a map, with the
/// values being mutable.
pub struct IterMut<'a, V: 'a> {
    iter: slice::IterMut<'a, Entry<V>>,
    counter: usize,
}
impl<'a, V: 'a> Iterator for IterMut<'a, V> {
    type Item = (usize, &'a mut V);

    #[allow(unused_lifetimes, match_ref_pats)]
    fn next<'b>(&'b mut self) -> Option<(usize, &'a mut V)> {
        generate_iterator!(self, mut);
    }
}

impl<'a, V: 'a> IntoIterator for &'a mut CompactMap<V> {
    type Item = (usize, &'a mut V);
    type IntoIter = IterMut<'a, V>;
    fn into_iter(self) -> IterMut<'a, V> {
        IterMut {
            iter: self.data.iter_mut(),
            counter: 0,
        }
    }
}

/// A consuming iterator over the key-value pairs of a map.
pub struct IntoIter<V> {
    iter: vec::IntoIter<Entry<V>>,
    counter: usize,
}
impl<V> Iterator for IntoIter<V> {
    type Item = (usize, V);

    fn next(&mut self) -> Option<(usize, V)> {
        generate_iterator!(self, plain);
    }
}
impl<V> IntoIterator for CompactMap<V> {
    type Item = (usize, V);
    type IntoIter = IntoIter<V>;
    fn into_iter(self) -> IntoIter<V> {
        IntoIter {
            iter: self.data.into_iter(),
            counter: 0,
        }
    }
}


/// An iterator over the keys of a map.
pub struct Keys<'a, V: 'a> {
    iter: Iter<'a, V>,
}
// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Keys<'a, V> {
    fn clone(&self) -> Keys<'a, V> {
        Keys { iter: self.iter.clone() }
    }
}
impl<'a, V> Iterator for Keys<'a, V> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.iter.next().map(|e| e.0)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator over the values of a map.
pub struct Values<'a, V: 'a> {
    iter: Iter<'a, V>,
}
// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Values<'a, V> {
    fn clone(&self) -> Values<'a, V> {
        Values { iter: self.iter.clone() }
    }
}
impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        self.iter.next().map(|e| e.1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator over the values of a map.
pub struct ValuesMut<'a, V: 'a> {
    iter_mut: IterMut<'a, V>,
}
impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<&'a mut V> {
        self.iter_mut.next().map(|e| e.1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_mut.size_hint()
    }
}

/// A draining iterator over the key-value pairs of a map.
pub struct Drain<'a, V: 'a> {
    iter: std::iter::FilterMap<
        std::iter::Enumerate<std::vec::Drain<'a, Entry<V>>>,
        fn((usize, Entry<V>)) -> Option<(usize, V)>
        >
}

impl<'a, V> Iterator for Drain<'a, V> {
    type Item = (usize, V);

    fn next(&mut self) -> Option<(usize, V)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}



#[cfg(feature = "serde")]
mod serdizer {
    extern crate serde;

    use super::CompactMap;
    use super::Entry;

    use super::SENTINEL;
    use self::serde::ser::SerializeMap;

    impl<V: serde::Serialize> serde::Serialize for CompactMap<V> {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            #[cfg(feature = "serde_ser_len")]
            let len = Some(self.len_slow());
            #[cfg(not(feature = "serde_ser_len"))]
            let len = None;

            let mut map = s.serialize_map(len)?;
            for (k, v) in self {
                map.serialize_entry(&k, v)?;
            }
            map.end()
        }
    }

    // Deserializer based on https://serde.rs/deserialize-map.html

    use std::fmt;
    use std::marker::PhantomData;

    use self::serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

    struct MyMapVisitor<V> {
        marker: PhantomData<fn() -> CompactMap<V>>,
    }

    impl<V> MyMapVisitor<V> {
        fn new() -> Self {
            MyMapVisitor { marker: PhantomData }
        }
    }

    impl<'de, V> Visitor<'de> for MyMapVisitor<V>
    where
        V: Deserialize<'de>,
    {
        type Value = CompactMap<V>;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with small nonnegative integer keys")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = CompactMap::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((key, value)) = access.next_entry()? {

                // because of Vec::resize_default is unstable
                while map.data.len() <= key {
                    map.data.push(Entry::Empty(SENTINEL));
                }
                map.data[key] = Entry::Occupied(value);
            }
            map.reindex();

            Ok(map)
        }
    }

    // This is the trait that informs Serde how to deserialize MyMap.
    impl<'de, V> Deserialize<'de> for CompactMap<V>
    where
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Instantiate our Visitor and ask the Deserializer to drive
            // it over the input data, resulting in an instance of MyMap.
            deserializer.deserialize_map(MyMapVisitor::new())
        }
    }
}

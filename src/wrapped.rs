use ::std::marker::PhantomData;
use ::std::convert::From;
use ::std::iter::FromIterator;
use ::std::ops::{Index, IndexMut};
use ::std::fmt;

/// Special version of CompactMap that uses your usize-equivalent types as keys
/// You are expected to use newtype-style structs like `struct MyToken(usize);` for this
/// If needed, you can cheat with `into_unwrapped`, `unwrapped` and so on.
///
/// For example, this will fail:
///
/// ```compile_fail
/// #[macro_use]
/// extern crate compactmap;
/// fn main() {
///   use compactmap::wrapped::CompactMap;
///   
///   declare_compactmap_token!(Mom);
///   declare_compactmap_token!(Lol);
///   
///   let mut m1: CompactMap<Mom, u64> = CompactMap::new();
///   let mut m2: CompactMap<Lol, u64> = CompactMap::new();
///   
///   let q = m1.insert(123);
///   m2.remove(q); // expected type `main::Lol`, found type `main::Mom`
/// }
/// ```
#[derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompactMap<K : Into<usize> + From<usize>, V> {
    inner: super::CompactMap<V>,
    _pd: PhantomData<K>,
}

impl<K:Into<usize> + From<usize>, V> CompactMap<K,V> {
    /// Extract underlying unwrapped map
    pub fn into_unwrapped(self) -> super::CompactMap<V> {
        self.inner
    }
    
    /// Wrap the map. You are responsible that it is the correct one
    pub fn from_unwrapped(s: super::CompactMap<V>) -> Self {
        CompactMap {
            inner : s,
            _pd : Default::default(),
        }
    }
    
    /// Temporarily use the map without the safety wrapper
    pub fn unwrapped(&self) -> &super::CompactMap<V> {
        &self.inner
    }
    
    /// Temporarily use the map without the safety wrapper
    pub fn unwrapped_mut(&mut self) -> &mut super::CompactMap<V> {
        &mut self.inner
    }
}

// Forwarded content
impl<K:Into<usize> + From<usize>, V> CompactMap<K,V> {
    
    /// See [`super::CompactMap::new`](../struct.CompactMap.html#method.new)
    pub fn new() -> Self {
        CompactMap {
            inner: super::CompactMap::new(),
            _pd: Default::default(),
        }
    }

    /// See [`super::CompactMap::with_capacity`](../struct.CompactMap.html#method.with_capacity)
    pub fn with_capacity(capacity: usize) -> Self {
        CompactMap {
            inner: super::CompactMap::with_capacity(capacity),
            _pd: Default::default(),
        }
    }

    /// See [`super::CompactMap::capacity`](../struct.CompactMap.html#method.capacity)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// See [`super::CompactMap::reserve`](../struct.CompactMap.html#method.reserve)
    pub fn reserve(&mut self, len: usize) {
        self.inner.reserve(len)
    }

    /// See [`super::CompactMap::reserve_exact`](../struct.CompactMap.html#method.reserve_exact)
    pub fn reserve_exact(&mut self, len: usize) {
        self.inner.reserve_exact(len)
    }

    // TODO: entry
    // TODO: DoubleEndedIterator

    /// See [`super::CompactMap::clear`](../struct.CompactMap.html#method.clear)
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// See [`super::CompactMap::is_empty_slow`](../struct.CompactMap.html#method.is_empty_slow)
    pub fn is_empty_slow(&self) -> bool {
        self.inner.is_empty_slow()
    }

    /// See [`super::CompactMap::insert`](../struct.CompactMap.html#method.insert)
    pub fn insert(&mut self, v: V) -> K {
        From::from(self.inner.insert(v))
    }

    /// See [`super::CompactMap::remove`](../struct.CompactMap.html#method.remove)
    pub fn remove(&mut self, i: K) -> Option<V> {
        self.inner.remove(i.into())
    }
    
    /// See [`super::CompactMap::get`](../struct.CompactMap.html#method.get)
    pub fn get(&self, i: K) -> Option<&V> {
        self.inner.get(i.into())
    }
    
    /// See [`super::CompactMap::get_mut`](../struct.CompactMap.html#method.get_mut)
    pub fn get_mut(&mut self, i: K) -> Option<&mut V> {
        self.inner.get_mut(i.into())
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order.
    /// The iterator's element type is `(K, &'r V)`.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            inner: self.inner.iter(),
             _pd: Default::default(),
        }
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order,
    /// with mutable references to the values.
    /// The iterator's element type is `(K, &'r mut V)`
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            inner: self.inner.iter_mut(),
             _pd: Default::default(),
        }
    }

    /// Returns an iterator visiting all key-value pairs in unspecified order,
    /// the keys, consuming the original `CompactMap`.
    /// The iterator's element type is `(K, V)`.
    pub fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            inner: self.inner.into_iter(),
             _pd: Default::default(),
        }
    }

    /// Returns an iterator visiting all keys in some order.
    /// The iterator's element type is `K`.
    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.inner.keys(), _pd: Default::default() }
    }
    
    /// See [`super::CompactMap::values`](../struct.CompactMap.html#method.values)
    pub fn values(&self) -> super::Values<V> {
        self.inner.values()
    }
    
    /// See [`super::CompactMap::values_mut`](../struct.CompactMap.html#method.values_mut)
    pub fn values_mut(&mut self) -> super::ValuesMut<V> {
        self.inner.values_mut()
    }
    
    /// See [`super::CompactMap::len_slow`](../struct.CompactMap.html#method.len_slow)
    pub fn len_slow(&self) -> usize {
        self.inner.len_slow()
    }

    /// See [`super::CompactMap::shrink_to_fit`](../struct.CompactMap.html#method.shrink_to_fit)
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Returns an iterator visiting all key-value pairs in ascending order of
    /// the keys, emptying (but not consuming) the original `CompactMap`.
    /// The iterator's element type is `(K, V)`. Keeps the allocated memory for reuse.
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain { 
            inner: self.inner.drain(),
            _pd : Default::default(),
        }
    }
}


impl<K:Into<usize> + From<usize>, V> FromIterator<V> for CompactMap<K, V> {
    fn from_iter<I>(iter: I) -> CompactMap<K, V>
    where
        I: IntoIterator<Item = V>,
    {
        CompactMap::from_unwrapped(super::CompactMap::from_iter(iter))
    }
}

impl<'a, K:Into<usize> + From<usize>, V> FromIterator<&'a V> for CompactMap<K, V>
where
    V: Copy,
{
    #[allow(map_clone)]
    fn from_iter<I>(iter: I) -> CompactMap<K, V>
    where
        I: IntoIterator<Item = &'a V>,
    {
        FromIterator::<V>::from_iter(iter.into_iter().map(|&value| value))
    }
}

impl<K:Into<usize> + From<usize>, V> Extend<V> for CompactMap<K,V> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = V>,
    {
        self.inner.extend(iter)
    }
}
impl<'a,K:Into<usize> + From<usize>, V> Extend<&'a V> for CompactMap<K,V>
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

impl<V,K:Into<usize> + From<usize>> Index<K> for CompactMap<K,V> {
    type Output = V;
    #[inline]
    fn index(&self, i: K) -> &V {
        self.inner.index(i.into())
    }
}
impl<'a,K:Copy + Into<usize> + From<usize>, V> Index<&'a K> for CompactMap<K,V> {
    type Output = V;
    fn index(&self, i: &K) -> &V {
        let idx : usize = (*i).into();
        self.inner.index(&idx)
    }
}
impl<K:Into<usize> + From<usize>,V> IndexMut<K> for CompactMap<K,V> {
    fn index_mut(&mut self, i: K) -> &mut V {
        self.inner.index_mut(i.into())
    }
}
impl<'a,K:Copy + Into<usize> + From<usize>, V> IndexMut<&'a K> for CompactMap<K,V> {
    fn index_mut(&mut self, i: &K) -> &mut V {
        let idx : usize = (*i).into();
        self.inner.index_mut(idx)
    }
}
impl<K:Into<usize> + From<usize>, V: fmt::Debug> fmt::Debug for CompactMap<K,V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}


/// An iterator over the key-value pairs of a map.
#[derive(Clone)]
pub struct Iter<'a, K: Into<usize> + From<usize>, V: 'a> {
    inner: super::Iter<'a, V>,
    _pd: PhantomData<K>,
}
impl<'a, K: Into<usize> + From<usize>, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    #[allow(match_ref_pats)]
    fn next(&mut self) -> Option<(K, &'a V)> {
        self.inner.next().map(|(k,v)|(From::from(k),v))
        /*if let Some((k,v)) = self.inner.next() {
            Some((From::from(k), v))
        } else  {
            None
        }*/
    }
}
impl<'a, K: Into<usize> + From<usize>, V> IntoIterator for &'a CompactMap<K,V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        Iter {
            inner: self.inner.iter(),
            _pd: Default::default(),
        }
    }
}


/// An iterator over the key-value pairs of a map, with the
/// values being mutable.
pub struct IterMut<'a, K: Into<usize> + From<usize>, V: 'a> {
    inner: super::IterMut<'a, V>,
    _pd: PhantomData<K>,
}
impl<'a, K: Into<usize> + From<usize>, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next<'b>(&'b mut self) -> Option<(K, &'a mut V)> {
        self.inner.next().map(|(k,v)|(From::from(k),v))
    }
}

impl<'a, K: Into<usize> + From<usize>, V: 'a> IntoIterator for &'a mut CompactMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> IterMut<'a, K, V> {
        IterMut {
            inner: self.inner.iter_mut(),
            _pd: Default::default(),
        }
    }
}


/// A consuming iterator over the key-value pairs of a map.
pub struct IntoIter<K : Into<usize> + From<usize>, V> {
    inner: super::IntoIter<V>,
    _pd: PhantomData<K>,
}
impl<K: Into<usize> + From<usize>, V> Iterator for IntoIter<K,V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.inner.next().map(|(k,v)|(From::from(k),v))
    }
}
impl<K: Into<usize> + From<usize>, V> IntoIterator for CompactMap<K,V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K,V>;
    fn into_iter(self) -> IntoIter<K,V> {
        IntoIter {
            inner: self.inner.into_iter(),
            _pd: Default::default(),
        }
    }
}



/// An iterator over the keys of a map.
#[derive(Clone)]
pub struct Keys<'a, K : Into<usize> + From<usize>, V: 'a> {
    inner: super::Keys<'a, V>,
    _pd: PhantomData<K>,
}
impl<'a, K : Into<usize> + From<usize>, V> Iterator for Keys<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.inner.next().map(|k| From::from(k))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}


/// A draining iterator over the key-value pairs of a map.
pub struct Drain<'a, K : Into<usize> + From<usize>, V: 'a> {
    inner: super::Drain<'a, V>,
    _pd: PhantomData<K>,
}

impl<'a, K : Into<usize> + From<usize>, V> Iterator for Drain<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.inner.next().map(|(k,v)|(From::from(k),v)) 
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}


/// Create usize-equivalent struct that implements `From<usize>` and `Into<usize>`
///
/// For [the wrapper](wrapped/struct.CompactMap.html).
///
/// ```
/// #[macro_use] extern crate compactmap;
/// use compactmap::wrapped::CompactMap;
/// declare_compactmap_token!(MyCompactmapIndex);
/// # fn main(){}
/// ```
#[macro_export]
macro_rules! declare_compactmap_token {
    ($x:ident) => {
        #[derive(Copy,Clone,Ord,PartialOrd,Eq,PartialEq,Hash,Debug)]
        struct $x(usize);
        impl From<usize> for $x {
            fn from(x:usize) -> Self { $x(x) }
        }
        impl From<$x> for usize {
            fn from(x:$x) -> usize {x.0}
        }
    }
}

#[cfg(feature = "serde")]
mod serdizer {
    extern crate serde;

    use super::CompactMap;

    impl<K: Into<usize> + From<usize>, V: serde::Serialize> serde::Serialize for CompactMap<K, V> {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            self.inner.serialize(s)
        }
    }

    use self::serde::de::{Deserialize, Deserializer};

    // This is the trait that informs Serde how to deserialize MyMap.
    impl<'de, K: Into<usize> + From<usize>, V> Deserialize<'de> for CompactMap<K, V>
    where
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let r : super::super::CompactMap<V> = Deserialize::deserialize(deserializer)?;
            Ok(CompactMap::from_unwrapped(r))
        }
    }
}

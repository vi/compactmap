#![allow(unknown_lints)]

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

#[derive(Clone)]
enum Entry<V> {
    Empty(usize),
    Occupied(V)
}

#[derive(Clone)]
pub struct CompactMap<V> {
    data: Vec<Entry<V>>,
    free_head: usize
}

impl<V> CompactMap<V> {
    pub fn new() -> CompactMap<V> {
        CompactMap {
            data: vec![],
            free_head: usize::MAX
        }
    }
    
    pub fn insert(&mut self, v: V) -> usize {
        let head = self.free_head;
        let entry = Entry::Occupied(v);
        if head == usize::MAX {
            self.data.push(entry);
            self.data.len() - 1
        } else {
            match mem::replace(&mut self.data[head], entry) {
                Entry::Empty(next) => {
                    self.free_head = next;
                    head
                }
                Entry::Occupied(_) => unreachable!()
            }
        }
    }
    
    pub fn take(&mut self, i: usize) -> Option<V> {
        if i >= self.data.len() {
            return None
        }
        if let Entry::Empty(_) = self.data[i] {
            // Early return to avoid further wrong mem::replace
            return None
        }
        
        let empty_entry = Entry::Empty(self.free_head);
        if let Entry::Occupied(v) = mem::replace(&mut self.data[i], empty_entry) {
            if i == self.data.len() - 1 {
                self.data.truncate(i);
            } else {
                self.free_head = i;
            }
            Some(v)
        } else { unreachable!(); }
    }
    
    #[inline]
    pub fn remove(&mut self, i: usize) {
        self.take(i);
    }
    
    pub fn get(&self, i: usize) -> Option<&V> {
        self.data.get(i).and_then(|entry| match *entry {
            Entry::Empty(_) => None,
            Entry::Occupied(ref v) => Some(v)
        })
    }
    
    pub fn get_mut(&mut self, i: usize) -> Option<&mut V> {
        self.data.get_mut(i).and_then(|entry| match *entry {
            Entry::Empty(_) => None,
            Entry::Occupied(ref mut v) => Some(v)
        })
    }
    
    pub fn iter<'a>(&'a self) -> ReadOnlyIter<'a, V> {
        ReadOnlyIter { iter: self.data.iter(), counter: 0 }
    }
    
    pub fn iter_mut<'a>(&'a mut self) -> MutableIter<'a, V> {
        MutableIter { iter: self.data.iter_mut(), counter: 0 }
    }
    
    pub fn into_iter(self) -> IntoIter<V> {
        IntoIter { iter: self.data.into_iter(), counter: 0 }
    }
    
    pub fn len_slow(&self) -> usize {
        self.iter().count()
    }
}

impl<V> Default for CompactMap<V> {
    fn default() -> CompactMap<V> {
        CompactMap::new()
    }
}


impl<V> Hash for CompactMap<V> where V: Hash {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
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
impl<V> PartialOrd<CompactMap<V>> for CompactMap<V> where V: PartialOrd<V> {
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

impl<V> Ord for CompactMap<V> where V: Ord {
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
    fn from_iter<I>(iter: I) -> CompactMap<V> where I: IntoIterator<Item=V> {
        let mut c = CompactMap::new();
        // TODO size hint here maybe
        for i in iter {
            c.insert(i);
        }
        c
    }
}

impl<'a, V> FromIterator<&'a V> for CompactMap<V> where V : Copy {
    #[allow(map_clone)]
    fn from_iter<I>(iter: I) -> CompactMap<V> where I: IntoIterator<Item=&'a V> {
        FromIterator::<V>::from_iter(iter.into_iter().map(|&value| value))
    }
}

impl<V> Extend<V> for CompactMap<V> {
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=V> {
        // TODO: maybe use size hint here
        for i in iter {
            self.insert(i);
        }
    }
}
impl<'a, V> Extend<&'a V> for CompactMap<V> where V: Copy {
    #[allow(map_clone)]
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=&'a V> {
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


//TODO: compaction?

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

pub struct ReadOnlyIter<'a, V : 'a> {
    iter: slice::Iter<'a, Entry<V>>,
    counter : usize,
}
impl<'a,V> Iterator for ReadOnlyIter<'a,V> {
    type Item = (usize, &'a V);
    
    #[allow(match_ref_pats)]
    fn next(&mut self) -> Option<(usize, &'a V)> {
        generate_iterator!(self, const);
    }
}
impl<'a,V> IntoIterator for &'a CompactMap<V> {
    type Item = (usize, &'a V);
    type IntoIter = ReadOnlyIter<'a, V>;
    fn into_iter(self) -> ReadOnlyIter<'a, V> {
        ReadOnlyIter { iter: self.data.iter(), counter: 0 }
    }
}


pub struct MutableIter<'a, V : 'a> {
    iter: slice::IterMut<'a, Entry<V>>,
    counter : usize,
}
impl<'a,V:'a> Iterator for MutableIter<'a,V> {
    type Item = (usize, &'a mut V);
    
    #[allow(unused_lifetimes,match_ref_pats)]
    fn next<'b>(&'b mut self) -> Option<(usize, &'a mut V)> {
        generate_iterator!(self, mut);
    }
}

impl<'a,V:'a> IntoIterator for &'a mut CompactMap<V> {
    type Item = (usize, &'a mut V);
    type IntoIter = MutableIter<'a, V>;
    fn into_iter(self) -> MutableIter<'a, V> {
        MutableIter { iter: self.data.iter_mut(), counter: 0 }
    }
}


pub struct IntoIter<V> {
    iter: vec::IntoIter<Entry<V>>,
    counter : usize,
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
        IntoIter { iter: self.data.into_iter(), counter: 0 }
    }
}



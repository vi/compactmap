mod test;

use std::mem;
use std::usize;
use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::Ordering;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

#[derive(Clone,Debug)]
enum Entry<V> {
    Empty(usize),
    Occupied(V)
}

#[derive(Clone,Debug)]
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
    
    pub fn remove(&mut self, i: usize) {
        if i >= self.data.len() {
            return
        }
        if let Entry::Occupied(_) = self.data[i] {
            if i == self.data.len() - 1 {
                self.data.truncate(i);
            } else {
                self.data[i] = Entry::Empty(self.free_head);
                self.free_head = i;
            }
        }
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

// Compare for equality disregarting removed values linked list bookkeeping
impl<V> PartialEq<CompactMap<V>> for CompactMap<V> where V: PartialEq<V> {
    fn eq(&self, other: &CompactMap<V>) -> bool {
        iterate_for_ord_and_eq!(self, other, 
                                false, false, 
                                j, k, 
            both_found {
                if k.ne(j) {
                    return false
                }
            });
        true
    }
}

impl<V> Eq for CompactMap<V> where V: Eq { }

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
        return c
    }
}

impl<'a, V> FromIterator<&'a V> for CompactMap<V> where V : Copy {
    fn from_iter<I>(iter: I) -> CompactMap<V> where I: IntoIterator<Item=&'a V> {
        return FromIterator::<V>::from_iter(iter.into_iter().map(|&value| value))
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
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=&'a V> {
       self.extend(iter.into_iter().map(|&value| value));
    }
}

// Index and IntexMut mostly borrowed from VecMap
impl<V> Index<usize> for CompactMap<V> {
    type Output = V;
    #[inline]
    fn index<'a>(&'a self, i: usize) -> &'a V {
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


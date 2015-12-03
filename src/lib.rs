mod test;

use std::mem;
use std::usize;
use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::Ordering;

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


mod test;

use std::mem;
use std::usize;
use std::hash::Hash;
use std::hash::Hasher;

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

impl<V> PartialEq<CompactMap<V>> for CompactMap<V> where V: PartialEq<V> {
    fn eq(&self, other: &CompactMap<V>) -> bool {
        for i in 0..(self.data.len()) {
            if let Entry::Occupied(ref j) = self.data[i] {
                if i >= other.data.len() {
                    return false
                }
                if let Entry::Occupied(ref k) = other.data[i] {
                    if k.ne(j) {
                        return false
                    }
                } else {
                    return false
                }
            } else {
                if i >= other.data.len() {
                    continue;
                }
                if let Entry::Occupied(_) = other.data[i] {
                    return false;
                } 
            }
        }
        
        for i in (self.data.len())..(other.data.len()) {
            if let Entry::Occupied(_) = other.data[i] {
                return false;
            }
        }
        true
    }
}

impl<V> Eq for CompactMap<V> where V: Eq { }

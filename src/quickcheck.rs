#![allow(dead_code)]
#![allow(unused)]

extern crate quickcheck;
extern crate slab;
#[cfg(feature = "serde")]
extern crate serde_json;
#[cfg(all(feature="serde", feature = "serde_ser_len"))]
extern crate bincode;

// Check against slab

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
enum Action {
    Insert(u16),
    Remove(usize),
    ShrinkToFit,
    #[cfg(feature = "serde")]
    SerdeJson,
    #[cfg(all(feature="serde", feature = "serde_ser_len"))]
    SerdeBincode,
}

type ActionSequence = Vec<Action>;



impl quickcheck::Arbitrary for Action {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        #[cfg(feature = "serde")]
        {if g.gen_weighted_bool(100) {
            return Action::SerdeJson
        }}
        #[cfg(all(feature="serde", feature = "serde_ser_len"))]
        {if g.gen_weighted_bool(100) {
            return Action::SerdeBincode
        }}
        
        if g.gen_weighted_bool(100) {
            Action::ShrinkToFit
        } else
        if g.gen_weighted_bool(2) {
            Action::Insert(g.gen_range(0, 50))
        } else {
            Action::Remove(g.gen_range(0, 50))
        }
    }
}

fn check(s: ActionSequence) -> bool {
    let mut cm   = super::CompactMap::<u16>::new();
    let mut slab = slab::Slab::<u16>::new();
    
    //println!("s={:?}", s);
    
    for a in s {
        match a {
            Action::Insert(x) => {
                let k1 = cm.insert(x);
                let k2 = slab.insert(x);
                //println!("k1={}, k2={}, x={}", k1, k2, x);
                if k1 != k2 {
                    // Divergence may happen, but can be safely ignored
                    let _ = cm.remove(k1);
                    let _ = slab.remove(k2);
                }
            },
            Action::Remove(n) => {
                if slab.contains(n) {
                    //println!("rm {}", n);
                    if cm.remove(n) != Some(slab.remove(n)) { 
                        println!("rm4 cm={} slab=", n);
                        return false
                    }
                } else {
                    if cm.remove(n) != None { 
                        println!("rm5 n={}", n);
                        return false
                    }
                }
            },
            Action::ShrinkToFit => {
                //println!("shrink");
                cm.shrink_to_fit();
            },
            #[cfg(feature = "serde")]
            Action::SerdeJson => {
                let s = serde_json::to_string(&cm).unwrap();
                cm = serde_json::from_str(&s).unwrap();
            },
            #[cfg(all(feature="serde", feature = "serde_ser_len"))]
            Action::SerdeBincode => {
                let s = bincode::serialize(&cm, bincode::Infinite).unwrap();
                cm = bincode::deserialize(&s).unwrap();
            },
        }
    }
    
    
    if cm.len_slow() != slab.len() {
        println!("len {} {}",cm.len_slow(), slab.len());
        return false;
    }
    
    for (k,v) in cm.iter() {
        if Some(v) != slab.get(k) {
            println!("2 k={}", k);
            return false;
        }
    }
    
    for (k,v) in slab.iter() {
        if Some(v) != cm.get(k) {
            println!("3 k={}", k);
            return false;
        }
    }
    
    
    //println!("good");
    true
}

#[test] 
fn qc() {
    quickcheck::QuickCheck::new()
        .tests(1000)
        .quickcheck(check as fn(ActionSequence)->bool);
}

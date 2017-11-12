#![allow(unused_variables)]

use super::CompactMap;
use std::cmp::Ordering;


#[test]
fn it_works() {
    let mut m: CompactMap<u64> = CompactMap::new();
    let i1 = m.insert(44);
    let i2 = m.insert(55);

    assert_eq!(i1, 0);
    assert_eq!(i2, 1);
    assert_eq!(Some(&44), m.get(0));
    assert_eq!(Some(&55), m.get(1));
    assert_eq!(None, m.get(2));

    m.remove(0);

    assert_eq!(None, m.get(0));
    assert_eq!(Some(&55), m.get(1));
    assert_eq!(None, m.get(2));

    let i3 = m.insert(66);

    assert_eq!(i3, 0);
    assert_eq!(Some(&66), m.get(0));
    assert_eq!(Some(&55), m.get(1));
    assert_eq!(None, m.get(2));

    m.remove(1);

    assert_eq!(Some(&66), m.get(0));
    assert_eq!(None, m.get(1));
    assert_eq!(None, m.get(2));

    m.remove(1);

    assert_eq!(Some(&66), m.get(0));
    assert_eq!(None, m.get(1));
    assert_eq!(None, m.get(2));

    m.remove(0);

    assert_eq!(None, m.get(0));
    assert_eq!(None, m.get(1));
    assert_eq!(None, m.get(2));
}

#[test]
fn tricky_removal() {
    let mut m: CompactMap<u64> = CompactMap::new();
    let i44 = m.insert(44);
    let i55 = m.insert(55);
    let i66 = m.insert(66);
    let i77 = m.insert(77);
    let i88 = m.insert(88);
    let i99 = m.insert(99);

    m.remove(i77);
    m.remove(i99);
    m.remove(i88);

    assert_eq!(Some(&44), m.get(0));
    assert_eq!(Some(&55), m.get(1));
    assert_eq!(Some(&66), m.get(2));
    assert_eq!(None, m.get(3));

    let i110 = m.insert(110);

    assert_eq!(Some(&66), m.get(2));
    assert_eq!(Some(&110), m.get(i110));

    let i220 = m.insert(220);


    assert_eq!(Some(&110), m.get(i110));
    assert_eq!(Some(&220), m.get(i220));
}


#[test]
fn eq() {
    let mut m1: CompactMap<u64> = CompactMap::new();
    let mut m2: CompactMap<u64> = CompactMap::new();

    m1.insert(10);
    m1.insert(20);

    m2.insert(10);
    m2.insert(20);

    assert_eq!(m1, m2);

    m1.insert(30);

    assert!(m1 != m2);

    m2.insert(30);
    m2.insert(40);

    assert!(m1 != m2);

    m1.insert(40);

    assert_eq!(m1, m2);

    m1.remove(2); // 30

    assert!(m1 != m2);

    m2.remove(1); // 20

    assert!(m1 != m2);

    m1.remove(1); // 20
    m2.remove(2); // 30

    assert_eq!(m1, m2);

    m1.remove(3);

    assert!(m1 != m2);

    m2.remove(3);

    assert_eq!(m1, m2);
}

#[test]
fn eq2() {
    let mut m1: CompactMap<u64> = CompactMap::new();
    let mut m2: CompactMap<u64> = CompactMap::new();

    m1.insert(10);
    m1.insert(20);
    m1.insert(30);

    m2.insert(10);
    m2.insert(20);
    m2.insert(30);

    assert_eq!(m1, m2);

    m1.remove(1);
    m1.remove(2);

    m2.remove(2);
    m2.remove(1);

    assert_eq!(m1, m2);

    m1.insert(40);
    m2.insert(40);

    assert_eq!(m1, m2);
}

#[test]
fn ord() {
    let mut m1: CompactMap<u64> = CompactMap::new();
    let mut m2: CompactMap<u64> = CompactMap::new();

    assert!(m1.cmp(&m2) == Ordering::Equal);
    m1.insert(10);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m1.insert(20);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m1.insert(30);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m2.insert(10);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m2.insert(20);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m2.insert(30);
    assert!(m1.cmp(&m2) == Ordering::Equal);

    m1.remove(1);
    assert!(m1.cmp(&m2) == Ordering::Less);
    m1.remove(2);
    assert!(m1.cmp(&m2) == Ordering::Less);

    m2.remove(2);
    assert!(m1.cmp(&m2) == Ordering::Less);
    m2.remove(1);
    assert!(m1.cmp(&m2) == Ordering::Equal);

    m1.insert(40);
    assert!(m1.cmp(&m2) == Ordering::Greater);
    m2.insert(40);
    assert!(m1.cmp(&m2) == Ordering::Equal);
}

#[test]
fn iter() {
    let mut m1: CompactMap<u64> = CompactMap::new();
    m1.insert(10);
    m1.insert(20);
    m1.insert(30);

    {
        let mut iter = (&m1).into_iter();
        assert_eq!(iter.next(), Some((0, &10)));
        assert_eq!(iter.next(), Some((1, &20)));
        assert_eq!(iter.next(), Some((2, &30)));
        assert_eq!(iter.next(), None);
    }

    for (i, v) in &mut m1 {
        if i == 1 {
            *v = 99;
        }
    }

    assert_eq!(Some(&10), m1.get(0));
    assert_eq!(Some(&99), m1.get(1));
    assert_eq!(Some(&30), m1.get(2));
    assert_eq!(None, m1.get(3));

    {
        let mut last_sail = m1.into_iter();
        assert_eq!(last_sail.next(), Some((0, 10)));
        assert_eq!(last_sail.next(), Some((1, 99)));
        assert_eq!(last_sail.next(), Some((2, 30)));
        assert_eq!(last_sail.next(), None);
    }
}

#[test]
fn debug() {
    let mut m: CompactMap<u64> = CompactMap::new();
    let i44 = m.insert(44);
    let i55 = m.insert(55);
    let i66 = m.insert(66);
    let i77 = m.insert(77);
    let i88 = m.insert(88);
    let i99 = m.insert(99);

    m.remove(i77);
    m.remove(i99);
    m.remove(i88);

    assert_eq!(format!("{:?}", m), "{0: 44, 1: 55, 2: 66}");
}

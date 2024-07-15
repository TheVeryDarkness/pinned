use pinned_bucket::PinnedMap;
use rayon::prelude::*;

#[test]
fn insert() {
    let input = 0..1000;
    let res = PinnedMap::new();
    input.into_par_iter().for_each(|i| {
        res.insert(i, i * i);
        println!("Inserting {}", i);
    });
    assert_eq!(res.len(), 1000);
    for (k, v) in res.iter() {
        assert_eq!(*k * *k, *v);
    }
}

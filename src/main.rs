use std::collections::HashMap;

use tt::TtPlaythrough;

mod tt;

const PLAYERS: [usize; 3] = [0, 1, 2];

fn main() {
    env_logger::init();

    let mut playthrough = TtPlaythrough::new(PLAYERS.into(), 2);

    for _ in 0..10 {
        if let Err(err) = playthrough.get_next_match() {
            log::error!("{err}")
        };
    }

    log::info!("{:#?}", playthrough.matches);

    playthrough.check_match_possible((1, 0));

    let mut hash_map = HashMap::new();

    hash_map.insert(1, 2);
    hash_map.insert(2, 1);
    hash_map.insert(1, 0);
}

// sanity checks
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn test_hash_map_insert_updates() {
        let mut map: BTreeMap<usize, usize> = BTreeMap::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(1, 10); // updated 1

        let one = map.get(&1).unwrap().to_owned();
        let two = map.get(&2).unwrap().to_owned();

        assert_eq!(one, 10);
        assert_eq!(two, 1);

        map.entry(3).and_modify(|g| *g += 1).or_insert(0);
        assert_eq!(map.get(&3).unwrap().to_owned(), 0);

        map.entry(3).and_modify(|g| *g += 1).or_insert(0);
        assert_eq!(map.get(&3).unwrap().to_owned(), 1);

        map.entry(3).and_modify(|g| *g += 1).or_insert(0);
        assert_eq!(map.get(&3).unwrap().to_owned(), 2);

        let three = map.entry(3).and_modify(|g| *g += 1).or_insert(0);
        assert_eq!(three, &3);
        let got_three = map.get(&3).unwrap().to_owned();
        assert_eq!(got_three, 3);

        let mut sorted: Vec<(&usize, &usize)> = map.iter().collect();
        sorted.sort_by(|a, b| a.1.cmp(b.1));

        assert_eq!(sorted[0], (&2, &1));
        assert_eq!(sorted[1], (&3, &3));
        assert_eq!(sorted[2], (&1, &10));
    }
}

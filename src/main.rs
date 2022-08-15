use rand::seq::SliceRandom;
use std::time::Instant;

use tt::TtPlaythrough;

mod tt;

const GAMES_TOTAL: usize = 10usize.pow(5);
const PLAYERS: [usize; 3] = [0, 1, 2];
const MATCHES: [(usize, usize); 6] = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];

fn main() {
    env_logger::init();

    let mut playthrough = TtPlaythrough::new(PLAYERS.into(), 2);
    let mut random_generator = rand::thread_rng();

    let starttime = Instant::now(); // bench

    // takes 22s with 50.000 games
    for _ in 0..GAMES_TOTAL {
        let game = MATCHES
            .choose(&mut random_generator)
            .expect("MATCHES is not empty");

        playthrough.play_match_if_possible(*game)
    }

    // // takes 22 with 50.000 games
    // (0..GAMES_TOTAL)
    //     .map(|_| MATCHES.choose(&mut random_generator))
    //     .for_each(|game| {
    //         playthrough.play_match_if_possible(*game.expect("MATCHES is not empty"))
    //     });

    let elapsed = starttime.elapsed(); // bench

    playthrough.log_matches_so_far();

    log::info!(
        "Loop execution took: {:.2?} generating {} random games",
        elapsed,
        GAMES_TOTAL
    ); // bench
}

// sanity checks lol
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn test_vec_first_last() {
        let mut list = vec![1, 2, 3];

        assert_eq!(list.first(), Some(&1));
        assert_eq!(list.last(), Some(&3));

        list.push(4);
        assert_eq!(list.first(), Some(&1));
        assert_eq!(list.last(), Some(&4));
    }

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

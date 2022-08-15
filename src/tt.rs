use std::collections::BTreeMap;
use std::fmt;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TtMatch {
    left: usize,
    right: usize,
}

impl TtMatch {
    pub fn new(left: usize, right: usize) -> Self {
        Self { left, right }
    }

    pub fn check_same_players(&self, players: (usize, usize)) -> bool {
        let players_in_match = [self.left, self.right];

        players_in_match.contains(&players.0)
            && players_in_match.contains(&players.1)
    }
}

impl fmt::Display for TtMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "| {} - {} |", self.left, self.right)
    }
}

#[derive(Debug, Clone)]
pub struct TtPlaythrough {
    max_repeting_games_per_player: usize,
    players: Vec<usize>,
    matches: Vec<TtMatch>,
}

impl TtPlaythrough {
    //
    // construction and control
    //
    pub fn new(
        players: Vec<usize>,
        max_repeting_games_per_player: usize,
    ) -> Self {
        Self {
            max_repeting_games_per_player,
            players,
            matches: vec![],
        }
    }

    pub fn log_matches_so_far(&self) {
        let formatted_matches = self
            .matches
            .iter()
            .fold("".to_string(), |acc, m| acc + &format!("{m}\n"));

        println!(
            "MATCHES: \n{}\n\ntotal: {}",
            formatted_matches,
            self.matches.len()
        );
    }

    pub fn clear_match_history(&mut self) {
        self.matches = vec![];
    }

    //
    // public interface
    //
    pub fn play_match_if_possible(
        &mut self,
        players: (usize, usize),
        allow_rule_4: bool,
    ) {
        if self.check_match_possible(players, allow_rule_4) {
            self.append_game(players.0, players.1);
        }
    }

    pub fn check_match_possible(
        &mut self,
        players: (usize, usize),
        allow_rule_4: bool,
    ) -> bool {
        // Rule 1: don't play same players
        !self.check_same_players_as_before(players)

            // Rule 2: don't play three times in a row
            && self.check_not_played_twice_before(players)

            // Rule 3: don't play on the same side of the table as in the game before
            && self.check_not_on_same_side_as_one_game_before(players)

            // Rule 4: don't play on the same sides when facing the same opponent again
            && (allow_rule_4 || self.check_not_on_same_sides_facing_the_same_opponent_again(players))
    }

    //
    // internal methods
    //

    /// Rule 1: don't play same players
    fn check_same_players_as_before(&self, players: (usize, usize)) -> bool {
        if let Some(last_match) = self.matches.last() {
            last_match.check_same_players(players)
        } else {
            false
        }
    }

    /// Rule 2: don't play three times in a row
    fn check_not_played_twice_before(&self, players: (usize, usize)) -> bool {
        let players_map = self.get_last_n_games_counts();

        log::debug!(
            "Checking last {} games: {:?}",
            self.max_repeting_games_per_player,
            &players_map
        );

        if players_map.get(&players.0).unwrap_or(&0).to_owned()
            >= self.max_repeting_games_per_player
        {
            log::debug!(
                "Player {} played {} times in the last {} games already.",
                players.0,
                players_map.get(&players.0).unwrap_or(&0),
                self.max_repeting_games_per_player,
            );
            return false;
        }

        if players_map.get(&players.1).unwrap_or(&0).to_owned()
            >= self.max_repeting_games_per_player
        {
            log::debug!(
                "Player {} played {} times in the last {} games already.",
                players.1,
                players_map.get(&players.1).unwrap_or(&0),
                self.max_repeting_games_per_player,
            );
            return false;
        }

        true
    }

    /// Rule 3: don't play on the same side of the table as in the game before
    fn check_not_on_same_side_as_one_game_before(
        &self,
        players: (usize, usize),
    ) -> bool {
        if let Some(last_match) = self.matches.last() {
            if last_match.left == players.0 {
                return false;
            } else if last_match.right == players.1 {
                return false;
            }
        };

        true
    }

    /// Rule 4: don't play on the same sides when facing the same opponent again
    fn check_not_on_same_sides_facing_the_same_opponent_again(
        &self,
        players: (usize, usize),
    ) -> bool {
        let matches_reversed = self.get_matches_reversed();

        let last_match = matches_reversed
            .into_iter()
            .find(|m| m.check_same_players(players));

        if let Some(found_match) = last_match {
            if found_match.left == players.0 {
                return false;
            } else if found_match.right == players.1 {
                return false;
            }
        };

        true
    }

    fn get_last_n_games_counts(&self) -> BTreeMap<usize, usize> {
        let last_n_matches = self.get_last_n_matches();
        let mut players_map = self.get_empty_player_map();

        for m in last_n_matches {
            players_map
                .entry(m.left)
                .and_modify(|p| *p += 1)
                .or_insert(0);

            players_map
                .entry(m.right)
                .and_modify(|p| *p += 1)
                .or_insert(0);
        }

        players_map
    }

    fn get_last_n_matches(&self) -> Vec<TtMatch> {
        let mut last_n = self
            .matches
            .clone()
            .into_iter()
            .rev()
            .take(self.max_repeting_games_per_player)
            .collect_vec();

        last_n.reverse();

        last_n
    }

    fn get_matches_reversed(&self) -> Vec<TtMatch> {
        let mut matches = self.matches.clone();
        matches.reverse();

        matches
    }

    /// key is the player and value can be used to count
    fn get_empty_player_map(&self) -> BTreeMap<usize, usize> {
        let mut players_map: BTreeMap<usize, usize> = BTreeMap::new();

        for p in self.players.clone().into_iter() {
            players_map.insert(p, 0);
        }

        players_map
    }

    fn append_game(&mut self, left: usize, right: usize) {
        self.matches.push(TtMatch::new(left, right));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PLAYERS: [usize; 3] = [0, 1, 2];

    #[test]
    fn test_forbit_same_players() {
        let tt_match = TtMatch::new(1, 2);

        assert_eq!(tt_match.check_same_players((0, 1)), false);
        assert_eq!(tt_match.check_same_players((0, 2)), false);
        assert_eq!(tt_match.check_same_players((1, 0)), false);
        assert_eq!(tt_match.check_same_players((2, 0)), false);
        assert_eq!(tt_match.check_same_players((1, 2)), true);
        assert_eq!(tt_match.check_same_players((2, 1)), true);
    }

    #[test]
    fn test_get_empty_player_map() {
        let playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        let empty_map = playthrough.get_empty_player_map();
        assert_eq!(empty_map, BTreeMap::from([(0, 0), (1, 0), (2, 0)]));
    }

    #[test]
    fn test_get_last_n_games() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        let last_n_matches = playthrough.get_last_n_matches();
        assert!(last_n_matches.is_empty());

        playthrough.append_game(0, 1);
        let last_n_matches = playthrough.get_last_n_matches();
        assert!(last_n_matches.len() < 3);
        assert_eq!(last_n_matches[0], TtMatch::new(0, 1));

        playthrough.append_game(0, 2);
        let last_n_matches = playthrough.get_last_n_matches();
        assert!(last_n_matches.len() < 3);
        assert_eq!(last_n_matches[0], TtMatch::new(0, 1));
        assert_eq!(last_n_matches[1], TtMatch::new(0, 2));

        playthrough.append_game(1, 2);
        let last_n_matches = playthrough.get_last_n_matches();
        assert!(last_n_matches.len() < 3);
        assert_eq!(last_n_matches[0], TtMatch::new(0, 2));
        assert_eq!(last_n_matches[1], TtMatch::new(1, 2));

        playthrough.append_game(1, 0);
        let last_n_matches = playthrough.get_last_n_matches();
        assert!(last_n_matches.len() < 3);
        assert_eq!(last_n_matches[0], TtMatch::new(1, 2));
        assert_eq!(last_n_matches[1], TtMatch::new(1, 0));
    }

    #[test]
    fn test_get_last_n_games_counts() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);

        let counts = playthrough.get_last_n_games_counts();
        assert_eq!(counts.get(&0).unwrap().to_owned(), 2);
        assert_eq!(counts.get(&1).unwrap().to_owned(), 1);
        assert_eq!(counts.get(&2).unwrap().to_owned(), 1);

        playthrough.append_game(1, 2);

        let counts = playthrough.get_last_n_games_counts();
        assert_eq!(counts.get(&0).unwrap().to_owned(), 1);
        assert_eq!(counts.get(&1).unwrap().to_owned(), 1);
        assert_eq!(counts.get(&2).unwrap().to_owned(), 2);

        playthrough.append_game(1, 0);

        let counts = playthrough.get_last_n_games_counts();
        assert_eq!(counts.get(&0).unwrap().to_owned(), 1);
        assert_eq!(counts.get(&1).unwrap().to_owned(), 2);
        assert_eq!(counts.get(&2).unwrap().to_owned(), 1);
    }

    #[test]
    fn test_get_matches_reversed() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);
        playthrough.append_game(1, 2);

        let manually_reversed =
            vec![TtMatch::new(1, 2), TtMatch::new(0, 2), TtMatch::new(0, 1)];

        assert_eq!(playthrough.get_matches_reversed(), manually_reversed);

        playthrough.append_game(0, 1);
        let manually_reversed = vec![
            TtMatch::new(0, 1),
            TtMatch::new(1, 2),
            TtMatch::new(0, 2),
            TtMatch::new(0, 1),
        ];

        assert_eq!(playthrough.get_matches_reversed(), manually_reversed);
    }

    #[test]
    fn test_check_same_players_as_before() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);

        assert_eq!(playthrough.check_same_players_as_before((0, 1)), false);
        assert_eq!(playthrough.check_same_players_as_before((0, 2)), true);
        assert_eq!(playthrough.check_same_players_as_before((1, 0)), false);
        assert_eq!(playthrough.check_same_players_as_before((2, 0)), true);
        assert_eq!(playthrough.check_same_players_as_before((1, 2)), false);
        assert_eq!(playthrough.check_same_players_as_before((2, 1)), false);
    }

    #[test]
    fn test_check_not_on_same_side_as_one_game_before() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);

        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((0, 1)),
            false
        );
        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((0, 2)),
            false
        );
        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((1, 0)),
            true
        );
        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((2, 0)),
            true
        );
        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((1, 2)),
            false
        );
        assert_eq!(
            playthrough.check_not_on_same_side_as_one_game_before((2, 1)),
            true
        );
    }

    #[test]
    fn test_check_not_on_same_sides_facing_the_same_opponent_again() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);
        playthrough.append_game(2, 1);
        playthrough.append_game(1, 0);
        playthrough.append_game(2, 0);

        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((0, 1)),
            true
        );
        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((0, 2)),
            true
        );
        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((1, 0)),
            false
        );
        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((2, 0)),
            false
        );
        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((1, 2)),
            true
        );
        assert_eq!(
            playthrough
                .check_not_on_same_sides_facing_the_same_opponent_again((2, 1)),
            false
        );
    }

    #[test]
    fn test_check_not_played_twice_before() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        playthrough.append_game(0, 1);
        playthrough.append_game(0, 2);

        assert_eq!(playthrough.check_not_played_twice_before((0, 1)), false);
        assert_eq!(playthrough.check_not_played_twice_before((0, 2)), false);
        assert_eq!(playthrough.check_not_played_twice_before((1, 0)), false);
        assert_eq!(playthrough.check_not_played_twice_before((2, 0)), false);
        assert_eq!(playthrough.check_not_played_twice_before((1, 2)), true);
        assert_eq!(playthrough.check_not_played_twice_before((2, 1)), true);

        playthrough.append_game(1, 2);

        assert_eq!(playthrough.check_not_played_twice_before((0, 1)), true);
        assert_eq!(playthrough.check_not_played_twice_before((0, 2)), false);
        assert_eq!(playthrough.check_not_played_twice_before((1, 0)), true);
        assert_eq!(playthrough.check_not_played_twice_before((2, 0)), false);
        assert_eq!(playthrough.check_not_played_twice_before((1, 2)), false);
        assert_eq!(playthrough.check_not_played_twice_before((2, 1)), false);
    }

    #[test]
    fn test_check_matches_possible_with_rule_4() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        assert_eq!(playthrough.check_match_possible((0, 1), false), true);
        assert_eq!(playthrough.check_match_possible((0, 2), false), true);
        assert_eq!(playthrough.check_match_possible((1, 0), false), true);
        assert_eq!(playthrough.check_match_possible((2, 0), false), true);
        assert_eq!(playthrough.check_match_possible((1, 2), false), true);
        assert_eq!(playthrough.check_match_possible((2, 1), false), true);

        playthrough.append_game(2, 1);
        assert_eq!(playthrough.check_match_possible((0, 1), false), false); // same side
        assert_eq!(playthrough.check_match_possible((0, 2), false), true);
        assert_eq!(playthrough.check_match_possible((1, 0), false), true);
        assert_eq!(playthrough.check_match_possible((2, 0), false), false); // same side
        assert_eq!(playthrough.check_match_possible((1, 2), false), false); // same players
        assert_eq!(playthrough.check_match_possible((2, 1), false), false); // same players

        playthrough.append_game(0, 2);
        assert_eq!(playthrough.check_match_possible((0, 1), false), false); // same side
        assert_eq!(playthrough.check_match_possible((0, 2), false), false); // same players
        assert_eq!(playthrough.check_match_possible((1, 0), false), true);
        assert_eq!(playthrough.check_match_possible((2, 0), false), false); // same players
        assert_eq!(playthrough.check_match_possible((1, 2), false), false); // same side
        assert_eq!(playthrough.check_match_possible((2, 1), false), false); // played twice already

        playthrough.append_game(1, 0);
        assert_eq!(playthrough.check_match_possible((0, 1), false), false); // same players
        assert_eq!(playthrough.check_match_possible((0, 2), false), false); // played twice already
        assert_eq!(playthrough.check_match_possible((1, 0), false), false); // same players
        assert_eq!(playthrough.check_match_possible((2, 0), false), false); // played twice already
        assert_eq!(playthrough.check_match_possible((1, 2), false), false); // same side
        assert_eq!(playthrough.check_match_possible((2, 1), false), false); // same side facing same opponent
    }

    #[test]
    fn test_check_matches_possible_without_rule_4() {
        let mut playthrough = TtPlaythrough::new(TEST_PLAYERS.into(), 2);

        assert_eq!(playthrough.check_match_possible((0, 1), true), true);
        assert_eq!(playthrough.check_match_possible((0, 2), true), true);
        assert_eq!(playthrough.check_match_possible((1, 0), true), true);
        assert_eq!(playthrough.check_match_possible((2, 0), true), true);
        assert_eq!(playthrough.check_match_possible((1, 2), true), true);
        assert_eq!(playthrough.check_match_possible((2, 1), true), true);

        playthrough.append_game(2, 1);
        assert_eq!(playthrough.check_match_possible((0, 1), true), false); // same side
        assert_eq!(playthrough.check_match_possible((0, 2), true), true);
        assert_eq!(playthrough.check_match_possible((1, 0), true), true);
        assert_eq!(playthrough.check_match_possible((2, 0), true), false); // same side
        assert_eq!(playthrough.check_match_possible((1, 2), true), false); // same players
        assert_eq!(playthrough.check_match_possible((2, 1), true), false); // same players

        playthrough.append_game(0, 2);
        assert_eq!(playthrough.check_match_possible((0, 1), true), false); // same side
        assert_eq!(playthrough.check_match_possible((0, 2), true), false); // same players
        assert_eq!(playthrough.check_match_possible((1, 0), true), true);
        assert_eq!(playthrough.check_match_possible((2, 0), true), false); // same players
        assert_eq!(playthrough.check_match_possible((1, 2), true), false); // same side
        assert_eq!(playthrough.check_match_possible((2, 1), true), false); // played twice already

        playthrough.append_game(1, 0);
        assert_eq!(playthrough.check_match_possible((0, 1), true), false); // same players
        assert_eq!(playthrough.check_match_possible((0, 2), true), false); // played twice already
        assert_eq!(playthrough.check_match_possible((1, 0), true), false); // same players
        assert_eq!(playthrough.check_match_possible((2, 0), true), false); // played twice already
        assert_eq!(playthrough.check_match_possible((1, 2), true), false); // same side
        assert_eq!(playthrough.check_match_possible((2, 1), true), true);
    }
}

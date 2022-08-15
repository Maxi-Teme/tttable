use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TtMatch {
    pub left: usize,
    pub right: usize,
}

impl TtMatch {
    pub fn new(left: usize, right: usize) -> Self {
        Self { left, right }
    }

    pub fn forbit_same_players(
        &self,
        player1: &usize,
        player2: &usize,
    ) -> Result<(), String> {
        let players_in_match = [self.left, self.right];

        if players_in_match.contains(player1) && players_in_match.contains(player2) {
            Err("Same players in match!".to_string())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtPlaythrough {
    max_repeting_games_per_player: usize,
    pub players: Vec<usize>,
    pub matches: Vec<TtMatch>,
}

impl TtPlaythrough {
    pub fn new(players: Vec<usize>, max_repeting_games_per_player: usize) -> Self {
        Self {
            max_repeting_games_per_player,
            players,
            matches: vec![],
        }
    }

    pub fn get_next_match(&mut self) -> Result<TtMatch, String> {
        if self.matches.is_empty() {
            let first_to_play = TtMatch::new(self.players[0], self.players[1]);
            self.matches.push(first_to_play.clone());

            Ok(first_to_play)
        } else {
            let next_match = self.get_next_possible()?;
            self.matches.push(next_match.clone());
            Ok(next_match)
        }
    }

    pub fn check_match_possible(&mut self, players: (usize, usize)) -> bool {
        let match_possible = self.check_not_played_twice_before(players);
        if match_possible {
            self.matches.push(TtMatch::new(players.0, players.1));
        }

        match_possible
        // && self.check_correct_side_of_table(players)
    }

    fn get_next_possible(&self) -> Result<TtMatch, String> {
        let possible_players = self.get_not_played_twice_before()?;
        let possible_players = self.get_correct_side_of_table(possible_players)?;

        Ok(TtMatch::new(possible_players.0, possible_players.1))
    }

    fn check_not_played_twice_before(&self, players: (usize, usize)) -> bool {
        let last_n_matches = self.get_last_n_matches();

        let mut players_map = self.get_empty_player_map();

        log::debug!(
            "Checking last {} games: {:?}",
            self.max_repeting_games_per_player,
            &last_n_matches
        );

        for m in last_n_matches {
            {
                let count_left = players_map
                    .entry(m.left)
                    .and_modify(|p| *p += 1)
                    .or_insert(0);

                if *count_left >= self.max_repeting_games_per_player {
                    log::debug!(
                        "Player {} played {} times in the last {} games already.",
                        m.left,
                        count_left,
                        self.max_repeting_games_per_player,
                    );
                    return false;
                }
            }

            let count_right = players_map
                .entry(m.right)
                .and_modify(|p| *p += 1)
                .or_insert(0);

            if *count_right >= self.max_repeting_games_per_player {
                log::debug!(
                    "Player {} played {} times in the last {} games already.",
                    m.right,
                    count_right,
                    self.max_repeting_games_per_player,
                );
                return false;
            }
        }

        true
    }

    fn check_correct_side_of_table(&self, players: (usize, usize)) -> bool {
        true
    }

    /// Rule 2: players cannot play 3 times in a row
    fn get_not_played_twice_before(&self) -> Result<(usize, usize), String> {
        let last_n_matches = self.get_last_n_matches();

        let mut players_map = self.get_empty_player_map();

        for m in last_n_matches {
            let left = players_map.get(&m.left).unwrap_or(&0);
            let new_left = left + 1;

            let right = players_map.get(&m.right).unwrap_or(&0);
            let new_right = right + 1;

            players_map.remove(&m.left);
            players_map.insert(m.left, new_left);

            players_map.remove(&m.right);
            players_map.insert(m.right, new_right.clone());
        }

        let possible_players: Vec<usize> = self
            .players
            .clone()
            .into_iter()
            .filter(|p| {
                players_map.get(p).unwrap_or(&0) <= &self.max_repeting_games_per_player
            })
            .collect();

        if possible_players.len() < 2 {
            Err(format!(
                "Could not find any possible players. \
Last {} players had the following numbers of games: {:#?}",
                self.max_repeting_games_per_player, players_map,
            ))
        } else if possible_players.len() == 2 {
            Ok((possible_players[0], possible_players[1]))
        } else {
            let possible_players: Vec<usize> = players_map
                .into_iter()
                .sorted_by(|a, b| Ord::cmp(&a.1, &b.1))
                .map(|(p, _)| p)
                .take(2)
                .collect();

            let mut rng = rand::thread_rng();
            let first_idx: bool = rng.gen_bool(0.5);

            if first_idx {
                Ok((possible_players[0], possible_players[1]))
            } else {
                Ok((possible_players[1], possible_players[0]))
            }
        }
    }

    /// Rule 3: players should swich places on the tabel e.g.
    fn get_correct_side_of_table(
        &self,
        players: (usize, usize),
    ) -> Result<(usize, usize), String> {
        if let Some(last_match) = self.matches.last() {
            // sanity check
            let player1 = players.0;
            let player2 = players.1;

            last_match.forbit_same_players(&player1, &player2)?;

            if last_match.left == player1 {
                Ok((player2, player1))
            } else if last_match.left == player2 {
                Ok((player1, player2))
            } else {
                Ok((player1, player2))
            }
        } else {
            Ok(players)
        }
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
}

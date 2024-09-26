use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    /// Between 0 and 11
    ///
    /// Low Plat = 1
    /// (...)
    /// Low Dia = 4
    /// (...)
    /// Low Challenger = 7
    /// (...)
    /// Omega = 10
    /// PL = 11
    pub rank: u8,
    pub role_preferences: [Role; 3],

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub blacklisted_players: Vec<String>,
}

impl Player {
    /// Gets how much the player would like to play as the role.
    ///
    /// If the role is their first choice, returns 3
    /// If the role is their second choice, returns 2
    /// If the role is their last choice, returns 1
    pub fn get_preference_of_role(&self, role: Role) -> f64 {
        if role == self.role_preferences[0] {
            return 3.0;
        }

        if role == self.role_preferences[1] {
            return 2.0;
        }
        // It must be the third, otherwise we messed up
        else {
            return 1.0;
        }
    }

    pub fn create_random() -> Player {
        let mut rng = rand::thread_rng();

        let rank: u8 = rng.gen_range(1..=11);

        let mut role_preferences = [Role::Goalie, Role::Midfield, Role::Forward];

        role_preferences.shuffle(&mut rng);

        Player {
            id: Uuid::new_v4().to_string(),
            rank,
            role_preferences,
            blacklisted_players: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub enum Role {
    #[default]
    Goalie,
    Forward,
    Midfield,
}

pub const ROLE_WEIGHT: f64 = 1.0;
pub const RANK_WEIGHT: f64 = 10.0;
pub const BLACKLIST_WEIGHT: f64 = 20.0;
pub const ALLOWED_RANK_DEVIATION: f64 = 5.0;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Team {
    pub goalie: Player,
    pub midfield: Player,
    pub forward: Player,
}

impl Team {
    pub fn new(goalie: Player, midfield: Player, forward: Player) -> Self {
        Self {
            goalie,
            midfield,
            forward,
        }
    }

    /// Scores the team.
    ///
    /// A higher score means we prefer the team composition more.
    pub fn score(&self, target_rank: f64) -> f64 {
        let mut score = 0.0;

        score += self.goalie.get_preference_of_role(Role::Goalie) * ROLE_WEIGHT;
        score += self.midfield.get_preference_of_role(Role::Midfield) * ROLE_WEIGHT;
        score += self.forward.get_preference_of_role(Role::Forward) * ROLE_WEIGHT;

        let mut rank_sum = 0;
        rank_sum += self.goalie.rank;
        rank_sum += self.midfield.rank;
        rank_sum += self.forward.rank;

        let rank_diff = (target_rank - (rank_sum as f64)).abs();

        if rank_diff <= ALLOWED_RANK_DEVIATION {
            // Between 0 and 1, 1 if we are at the perfect rank, 0 if we are on the edge of the range
            let multiplier = (ALLOWED_RANK_DEVIATION - rank_diff) / ALLOWED_RANK_DEVIATION;

            score += RANK_WEIGHT * multiplier;
        }

        let mut has_players_who_hate_eachother = false;

        has_players_who_hate_eachother |=
            self.goalie.blacklisted_players.contains(&self.midfield.id);
        has_players_who_hate_eachother |=
            self.goalie.blacklisted_players.contains(&self.forward.id);

        has_players_who_hate_eachother |=
            self.midfield.blacklisted_players.contains(&self.goalie.id);
        has_players_who_hate_eachother |=
            self.midfield.blacklisted_players.contains(&self.forward.id);

        has_players_who_hate_eachother |=
            self.forward.blacklisted_players.contains(&self.midfield.id);
        has_players_who_hate_eachother |=
            self.forward.blacklisted_players.contains(&self.goalie.id);

        if has_players_who_hate_eachother {
            score -= BLACKLIST_WEIGHT;
        }

        score
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
/// A team with a unique name
pub struct NamedTeam {
    pub players: Team,
    pub name: String,
}

impl NamedTeam {
    /// Creates a named team from a team and our lists, randomly generating a name
    pub fn from_team_and_lists(
		  team: Team,
        adjectives: &Vec<String>,
        nouns: &Vec<String>,
    ) -> Self {
        Self {
            name: Self::generate_name(adjectives, nouns),
				players: team
		  }
    }

    /// Generates a random team name from a random list of adjectives and nouns
    pub fn generate_name(adjectives: &Vec<String>, nouns: &Vec<String>) -> String {
        let mut random = rand::thread_rng();

        let adjective_i = random.gen_range(0..adjectives.len());
        let adjective = adjectives[adjective_i].clone();

        let noun_i = random.gen_range(0..nouns.len());
        let noun = nouns[noun_i].clone();

        format!("{adjective} {noun}")
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Result {
    pub teams: Vec<NamedTeam>,
}

impl Result {
    /// Scores the result.
    ///
    /// A higher score means we prefer the team compositions more.
    pub fn score(&self, target_rank: f64) -> f64 {
        let mut score = 0.0;

        for team in &self.teams {
            score += team.players.score(target_rank);
        }

        score
    }
}

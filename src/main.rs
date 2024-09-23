use std::{collections::HashSet, fmt::Debug};

use color_print::cprintln;
use rand::Rng;
use types::{Player, Result, Role, Team};
use uuid::Uuid;

mod types;

pub const NUM_PLAYERS: usize = 30;

fn main() {

	let read_from_string = std::fs::read_to_string("players.json").expect("Failed to read players.json!");

	 let players: Vec<Player> = serde_json::from_str(&read_from_string).expect("Players.json format is wrong");

	 /*

	 If generating players:

    let mut players = Vec::new();

    for _i in 0..NUM_PLAYERS {
        players.push(Player::create_random());
    }

    // Randomly make some players hate eachother
    for player_index in 0..players.len() {
        // 10 percent chance
        let mut random = rand::thread_rng();
        let random_number = random.gen_range(1..=10);

        if random_number == 1 {
            // pick a random person
            let index = random.gen_range(0..NUM_PLAYERS);

            let other_id = players[index].id.clone();
            let current = players.get_mut(player_index).unwrap();

            current.blacklisted_players.push(other_id);
        }
    }

	 let serialized = serde_json::to_string_pretty(&players).unwrap();

	 std::fs::write("players_generated.json", serialized.to_string()).unwrap();
	 */

    let started = std::time::Instant::now();

    let mut sum_of_ranks = 0;
    for i in 0..players.len() {
        sum_of_ranks += players[i].rank;
    }
    let average_rank = sum_of_ranks as f64 / players.len() as f64;

    cprintln!("<magenta>Average rank: {}</magenta>", average_rank);

    let mut possible_teams = Vec::new();

    for i in 0..players.len() {
        let mut players_without_goalie = players.clone();
        players_without_goalie.remove(i);

        for j in 0..players_without_goalie.len() {
            let mut players_without_goalie_and_midfield = players_without_goalie.clone();
            players_without_goalie_and_midfield.remove(j);

            for k in 0..players_without_goalie_and_midfield.len() {
                let team = Team {
                    goalie: players[i].clone(),
                    midfield: players_without_goalie[j].clone(),
                    forward: players_without_goalie_and_midfield[k].clone(),
                };

                let score = team.score(average_rank * 3_f64);

                possible_teams.push((team, score));
            }
        }
    }

    // Sort by descending score
    possible_teams.sort_by(|x, y| y.1.total_cmp(&x.1));

    // Make a result by just going from the top -- greedy
    let mut teams = Vec::new();
    let mut taken_players: HashSet<String> = HashSet::new();

    let mut phase = 0;

    while possible_teams.len() > 0 {
        let mut team_scores_this_phase = Vec::new();

        cprintln!("<green>Phase {}:</green> <blue>{} teams left</blue>", phase, possible_teams.len());

        let team;

        if possible_teams.len() < 10_000 {
            for i in 0..possible_teams.len() {
                // Assess all the teams in this phase, along with the teams left after we've taken it
                //
                // E.g. look one step ahead
                let team = possible_teams.get(i).unwrap();

                // Hypothetically, take this team and see which ones are left
                let mut taken_players_next = taken_players.clone();
                let mut possible_teams_next = possible_teams.clone();

                // Hypothetically take this team
                taken_players_next.insert(team.0.goalie.id.clone());
                taken_players_next.insert(team.0.midfield.id.clone());
                taken_players_next.insert(team.0.forward.id.clone());

                possible_teams_next = possible_teams_next
                    .into_iter()
                    .filter(|x| {
                        !taken_players_next.contains(&x.0.goalie.id)
                            && !taken_players_next.contains(&x.0.midfield.id)
                            && !taken_players_next.contains(&x.0.forward.id)
                    })
                    .collect();

                // Compute the score of this + leftovers
                let mut uber_score = team.0.score(average_rank * 3_f64);

					 let mut sum_of_scores = 0.0;

                for j in 0..possible_teams_next.len() {
                    sum_of_scores += possible_teams_next[j].1;
                }

					 uber_score += sum_of_scores / possible_teams_next.len() as f64;

                team_scores_this_phase.push((team, uber_score));
            }

            // Sort by the next scores, take the best one
            team_scores_this_phase.sort_by(|x, y| y.1.total_cmp(&x.1));

            team = team_scores_this_phase.get(0).unwrap().0;
        }
		  else {
			  // Take the best team, for now don't bother
			  team = possible_teams.get(0).unwrap();
		  }

        teams.push(team.0.clone());

        // Take this team, actually this time
        taken_players.insert(team.0.goalie.id.clone());
        taken_players.insert(team.0.midfield.id.clone());
        taken_players.insert(team.0.forward.id.clone());

        possible_teams = possible_teams
            .into_iter()
            .filter(|x| {
                !taken_players.contains(&x.0.goalie.id)
                    && !taken_players.contains(&x.0.midfield.id)
                    && !taken_players.contains(&x.0.forward.id)
            })
            .collect();

        phase += 1;
    }

    let result = Result { teams };

    cprintln!("<cyan>Took {:?}</cyan>", started.elapsed());
    cprintln!("<green>Result score: {}</green>", result.score(average_rank * 3_f64));
    cprintln!("");
    cprintln!("Teams: ");
    for i in 0..result.teams.len() {
        let team = &result.teams[i];
        cprintln!("<green>{:.1}:</green>", team.score(average_rank * 3.0_f64),);
        cprintln!("	<blue>goalie  : {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.goalie.id, team.goalie.rank, team.goalie.role_preferences, team.goalie.blacklisted_players);
        cprintln!("	<blue>midfield: {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.midfield.id, team.midfield.rank, team.midfield.role_preferences, team.midfield.blacklisted_players);
        cprintln!("	<blue>forward : {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.forward.id, team.forward.rank, team.forward.role_preferences, team.forward.blacklisted_players);
    }

	 let serialized = serde_json::to_string_pretty(&result).unwrap();

	 std::fs::write("output_teams.json", serialized.as_bytes()).expect("Failed to write output");
}

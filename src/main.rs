use std::{
    collections::HashSet,
    fmt::Debug,
    io::{Read, Write},
};

use color_print::cprintln;
use rand::Rng;
use types::{NamedTeam, Player, Result, Role, StringPlayer, StringTeam, Team};
use uuid::Uuid;

mod types;

pub const NUM_PLAYERS: usize = 30;

fn main() {
    let read_to_string_res = std::fs::read_to_string("players.json");

    if let Err(ref e) = read_to_string_res {
        println!("Failed to read players.json: {}", e);
        pause();
    }

    let read_to_string = read_to_string_res.unwrap();

    let players_res = serde_json::from_str(&read_to_string);

    if let Err(ref e) = players_res {
        println!("Failed to deserialize players.json: {}", e);
        pause();
    }

    let string_players: Vec<StringPlayer> = players_res.unwrap();

    let read_to_string_res = std::fs::read_to_string("adjectives.json");

    if let Err(ref e) = read_to_string_res {
        println!("Failed to read adjectives.json: {}", e);
        pause();
    }

    let read_to_string = read_to_string_res.unwrap();

    let team_name_adjectives_res = serde_json::from_str(&read_to_string);

    if let Err(ref e) = team_name_adjectives_res {
        println!("Failed to deserialize adjectives.json: {}", e);
        pause();
    }

    let team_name_adjectives_deserialized: Vec<String> = team_name_adjectives_res.unwrap();

    let read_to_string_res = std::fs::read_to_string("nouns.json");

    if let Err(ref e) = read_to_string_res {
        println!("Failed to read nouns.json: {}", e);
        pause();
    }

    let read_to_string = read_to_string_res.unwrap();

    let team_name_nouns_res = serde_json::from_str(&read_to_string);

    if let Err(ref e) = team_name_nouns_res {
        println!("Failed to deserialize nouns.json: {}", e);
        pause();
    }

    let team_name_nouns_deserialized: Vec<String> = team_name_nouns_res.unwrap();

    // Cloning names is expensive, so only keep one instance of them while refering by
    // index in other cases
    let mut player_names = Vec::new();
    let mut players = Vec::new();

    for i in 0..string_players.len() {
        let player = string_players.get(i).unwrap();

        player_names.push(player.id.clone());
    }

    for i in 0..string_players.len() {
        let player = string_players.get(i).unwrap();

        let mut player_name_index = 0;

        for j in 0..player_names.len() {
            let random_player_name = player_names.get(j).unwrap();

            if player.id.eq(random_player_name) {
                player_name_index = j;
            }
        }

        let mut blacklisted_players = Vec::new();

        for blacklisted_player_id in player.blacklisted_players.clone().into_iter() {
            let mut blacklisted_player_name_index = usize::MAX;

            for j in 0..player_names.len() {
                let random_player_name = player_names.get(j).unwrap();

                if blacklisted_player_id
                    .to_lowercase()
                    .eq(&random_player_name.to_lowercase())
                {
                    blacklisted_player_name_index = j;
                }
            }

            // We didn't find them player name
            if blacklisted_player_name_index == usize::MAX {
                println!(
                    "Player {} hates {}, but the latter is not a registered player. Ignoring",
                    player.id, blacklisted_player_id
                );
                continue;
            }

            blacklisted_players.push(blacklisted_player_name_index);
        }

        let id_player = Player {
            id: player_name_index,
            blacklisted_players,
            role_preferences: player.role_preferences,
            rank: player.rank,
        };

        players.push(id_player);
    }

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

    let mut sum_of_ranks: f64 = 0.0;
    for i in 0..players.len() {
        sum_of_ranks += players.get(i).unwrap().rank as f64;
    }
    let average_rank = sum_of_ranks / players.len() as f64;

    cprintln!("<magenta>Average rank: {:.2}</magenta>", average_rank);

    let mut possible_teams = Vec::new();

    for goalie_index in 0..players.len() {
        let mut players_without_goalie = players.clone();
        players_without_goalie.remove(goalie_index);

        for midfield_index in 0..players_without_goalie.len() {
            let mut players_without_goalie_and_midfield = players_without_goalie.clone();
            players_without_goalie_and_midfield.remove(midfield_index);

            for forward_index in 0..players_without_goalie_and_midfield.len() {
                let goalie = players[goalie_index].clone();
                let midfield = players_without_goalie[midfield_index].clone();
                let forward = players_without_goalie_and_midfield[forward_index].clone();

                let team = Team::new(goalie, midfield, forward);

                let score = team.score(average_rank * 3_f64);

                possible_teams.push((team, score));
            }
        }
    }

    // Sort by descending score
    possible_teams.sort_by(|x, y| y.1.total_cmp(&x.1));

    // Make a result by just going from the top -- greedy
    let mut teams = Vec::new();

    let mut phase = 0;

    while possible_teams.len() > 0 {
        let mut team_scores_this_phase = Vec::new();

        cprintln!(
            "<green>Phase {}:</green> <blue>{} teams left</blue>",
            phase,
            possible_teams.len()
        );

        let team;

        if possible_teams.len() < 10_000 {
            for i in 0..possible_teams.len() {
                // Assess all the teams in this phase, along with the teams left after we've taken it
                //
                // E.g. look one step ahead
                let team = possible_teams.get(i).unwrap().clone();

                // Hypothetically, take this team and see which ones are left
                let mut possible_teams_next = possible_teams.clone();

                // Hypothetically take this team
                possible_teams_next = possible_teams_next
                    .into_iter()
                    .filter(|other_possible_team| {
                        !other_possible_team.0.in_team(team.0.goalie.id)
                            && !other_possible_team.0.in_team(team.0.midfield.id)
                            && !other_possible_team.0.in_team(team.0.forward.id)
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

            team = team_scores_this_phase.get(0).unwrap().0.clone();
        } else {
            // Take the best team, for now don't bother
            team = possible_teams.get(0).unwrap().clone();
        }

        teams.push(team.0.clone());

        // Take this team, actually this time
        possible_teams = possible_teams
            .into_iter()
            .filter(|other_possible_team| {
                !other_possible_team.0.in_team(team.0.goalie.id)
                    && !other_possible_team.0.in_team(team.0.midfield.id)
                    && !other_possible_team.0.in_team(team.0.forward.id)
            })
            .collect();

        phase += 1;
    }

    // Name the teams
    let mut name_hashset = HashSet::new();

    let mut named_teams = Vec::new();

    let mut random = rand::thread_rng();

    let mut available_team_adjectives = team_name_adjectives_deserialized.clone();
    let mut available_team_nouns = team_name_nouns_deserialized.clone();

    for team in teams.into_iter() {

		  // Refresh the list if it is too small
		  if available_team_adjectives.len() <= 2 && available_team_nouns.len() <= 2 {
			 available_team_nouns = team_name_nouns_deserialized.clone();
			 available_team_adjectives = team_name_adjectives_deserialized.clone();
		  }

        let adjective_i = random.gen_range(0..available_team_adjectives.len());
        let adjective = available_team_adjectives[adjective_i].clone();

        let noun_i = random.gen_range(0..available_team_nouns.len());
        let noun = available_team_nouns[noun_i].clone();

        let mut team_name = format!("{adjective} {noun}");

        let mut iterations: usize = 0;
        let mut team_name_unique = !name_hashset.contains(&team_name);

        while !team_name_unique {
            let adjective_i = random.gen_range(0..available_team_adjectives.len());
            let adjective = available_team_adjectives[adjective_i].clone();

            let noun_i = random.gen_range(0..available_team_nouns.len());
            let noun = available_team_nouns[noun_i].clone();

            team_name = format!("{adjective} {noun}");

				team_name_unique = name_hashset.contains(&team_name);

            iterations += 1;

            if iterations > 100_000 {
                println!("Failed to find a unique name after 100.000 iterations, you likely do not have enough random words");
                println!(
                    "You will have to manually find a different name for '{}'",
                    team_name
                );
                break;
            }
        }

        name_hashset.insert(team_name.clone());

		  available_team_adjectives.remove(adjective_i);
		  available_team_nouns.remove(noun_i);

        let string_team = team.to_string_team(&player_names);

        let named_team = NamedTeam {
            players: string_team,
            name: team_name,
        };

        named_teams.push(named_team);
    }

    let result = Result { teams: named_teams };

    cprintln!("<cyan>Took {:?}</cyan>", started.elapsed());
    cprintln!(
        "<green>Result score: {}</green>",
        result.score(average_rank * 3_f64)
    );
    cprintln!("");
    cprintln!("Teams: ");
    for i in 0..result.teams.len() {
        let team = &result.teams[i];
        cprintln!(
            "<green>{} -> {:.1}:</green>",
            team.name,
            team.players.score(average_rank * 3.0_f64)
        );
        cprintln!("	<blue>goalie  : {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.players.goalie.id, team.players.goalie.rank, team.players.goalie.role_preferences, team.players.goalie.blacklisted_players);
        cprintln!("	<blue>midfield: {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.players.midfield.id, team.players.midfield.rank, team.players.midfield.role_preferences, team.players.midfield.blacklisted_players);
        cprintln!("	<blue>forward : {}, <magenta>rank {}</magenta>, <cyan>role {:?}</cyan>, <red>hates: {:?}</red></blue>", team.players.forward.id, team.players.forward.rank, team.players.forward.role_preferences, team.players.forward.blacklisted_players);
    }

    let serialized = serde_json::to_string_pretty(&result).unwrap();

    let res = std::fs::write("output_teams.json", serialized.as_bytes());

    if let Err(e) = res {
        println!("Failed to write output: {}", e);
    }

    pause();
}

fn pause() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

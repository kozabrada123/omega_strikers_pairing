This is a super quick pairing algorithm for an omega strikers tournament

Basically, we compute an empirical score for each possible team, then we pick teams based off their score and the score of the teams they leave behind.

A team's score is computed based of the role preferences and position of each player and the total "rank" of the team.

## Usage:

Create a players.json (for a larger example, see players_example.json).

This is where the algorithm will pull its players from

```json
[
	{
	  "id": "pogger",
	  "rank": 10,
	  "role_preferences": [
		  "Goalie",
		  "Midfield",
		  "Forward"
	  ]
	},
	{
	  "id": "pogger2",
	  "rank": 5,
	  "role_preferences": [
		  "Forward",
		  "Goalie",
		  "Midfield"
	  ]
	},
	{
	  "id": "logger",
	  "rank": 7,
	  "role_preferences": [
		  "Forward",
		  "Midfield",
		  "Goalie"
	  ],
	  "blacklisted_players": [
		  "pogger2"
	  ]
	}
]
```

Then, run the executable in the same directory.

If running from source: `cargo run --release`

The algorithm will run, creating output_teams.json:

```json
{
  "teams": [
    {
      "goalie": {
        "id": "pogger",
        "rank": 10,
        "role_preferences": [
          "Goalie",
          "Midfield",
          "Forward"
        ]
      },
      "midfield": {
        "id": "logger",
        "rank": 7,
        "role_preferences": [
          "Forward",
          "Midfield",
          "Goalie"
        ],
        "blacklisted_players": [
          "pogger2"
        ]
      },
      "forward": {
        "id": "pogger2",
        "rank": 5,
        "role_preferences": [
          "Forward",
          "Goalie",
          "Midfield"
        ]
      }
    }
  ]
}
```

use crate::parsers::force_int;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{EmbarkId, PlayerLeaderboardData, PlayerInfo, LeaderboardData};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season7RankedCashoutLeaderboardEntry {
    pub rank:   u32,
    pub delta:  i32,
    pub score:  u64,
    pub league_number: i64,
}

#[derive(Deserialize)]
struct Season7RankedCashoutRow {
    #[serde(rename = "1")]
    rank: u32,
    // We get an empty string if there is no change but a number if there is one
    #[serde(rename = "2", deserialize_with = "force_int")]
    delta: i32,
    #[serde(rename = "3")]
    embark_id: String,
    #[serde(rename = "4")]
    league_number: i64,
    #[serde(rename = "5")]
    score: u64,
    #[serde(rename = "6")]
    steam_name: Option<String>,
    #[serde(rename = "7")]
    psn_name: Option<String>,
    #[serde(rename = "8")]
    xbox_name: Option<String>,
    #[serde(rename = "12")]
    club_tag: Option<String>,
}

impl From<Season7RankedCashoutRow> for PlayerLeaderboardData {
    fn from(r: Season7RankedCashoutRow) -> Self {
        let emb = EmbarkId(r.embark_id);

        let steam_name = r.steam_name;
        let psn_name   = r.psn_name;
        let xbox_name  = r.xbox_name;

        let club_tag   = r.club_tag;

        let player_info = PlayerInfo {
            embark_id: emb.clone(),
            steam_name,
            psn_name,
            xbox_name,
        };

        let entry = Season7RankedCashoutLeaderboardEntry {
            rank:  r.rank,
            delta: r.delta,
            score: r.score,
            league_number: r.league_number,
        };

        let leaderboard_data = LeaderboardData {
            season_7_ranked_cashout_leaderboard_entry: Some(entry),
            ..Default::default()
        };

        PlayerLeaderboardData {
            player_info,
            leaderboards: leaderboard_data,
            club_tag: club_tag,
        }
    }
}

pub fn parse(row: &Value) -> Option<PlayerLeaderboardData> {
    // Only object‐rows are supported here
    let obj = row.as_object()?;
    // Try to deserialize; if it fails, return None
    let row: Season7RankedCashoutRow = serde_json::from_value(Value::Object(obj.clone())).ok()?;
    Some(row.into())
}
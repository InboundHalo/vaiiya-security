use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{EmbarkId, PlayerLeaderboardData, PlayerInfo, LeaderboardData};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season7SponsorLeaderboardEntry {
    pub rank: u32,
    pub sponsor: String,
    pub fans: u64,
}

#[derive(Deserialize)]
struct Season7SponsorRow {
    #[serde(rename = "1")]
    rank: u32,

    #[serde(rename = "3")]
    embark_id: String,

    #[serde(rename = "6")]
    steam_name: Option<String>,

    #[serde(rename = "7")]
    psn_name: Option<String>,

    #[serde(rename = "8")]
    xbox_name: Option<String>,

    #[serde(rename = "9")]
    sponsor: String,

    #[serde(rename = "10")]
    fans: u64,

    #[serde(rename = "12")]
    club_tag: Option<String>,
}

impl From<Season7SponsorRow> for PlayerLeaderboardData {
    fn from(r: Season7SponsorRow) -> Self {
        let player_info = PlayerInfo {
            embark_id: EmbarkId(r.embark_id),
            steam_name: r.steam_name,
            psn_name: r.psn_name,
            xbox_name: r.xbox_name,
        };

        let entry = Season7SponsorLeaderboardEntry {
            rank: r.rank,
            sponsor: r.sponsor,
            fans: r.fans,
        };

        let leaderboard_data = LeaderboardData {
            season_7_sponsor_leaderboard_entry: Some(entry),
            ..Default::default()
        };

        PlayerLeaderboardData {
            player_info,
            leaderboards: leaderboard_data,
            club_tag: r.club_tag,
        }
    }
}

pub fn parse(row: &Value) -> Option<PlayerLeaderboardData> {
    let obj = row.as_object()?;
    let row: Season7SponsorRow = serde_json::from_value(Value::Object(obj.clone())).ok()?;
    Some(row.into())
}
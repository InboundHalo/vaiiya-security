use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{EmbarkId, PlayerLeaderboardData, PlayerInfo, LeaderboardData};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season7QuickCashLeaderboardEntry {
    pub rank: u32,
    pub points: u64,
}

#[derive(Deserialize)]
struct Season7QuickCashRow {
    #[serde(rename = "1")]
    rank: u32,

    #[serde(rename = "3")]
    embark_id: String,

    #[serde(rename = "5")]
    points: u64,

    #[serde(rename = "6")]
    steam_name: Option<String>,

    #[serde(rename = "7")]
    psn_name: Option<String>,

    #[serde(rename = "8")]
    xbox_name: Option<String>,

    #[serde(rename = "12")]
    club_tag: Option<String>,
}

impl From<Season7QuickCashRow> for PlayerLeaderboardData {
    fn from(r: Season7QuickCashRow) -> Self {
        let player_info = PlayerInfo {
            embark_id: EmbarkId(r.embark_id),
            steam_name: r.steam_name,
            psn_name: r.psn_name,
            xbox_name: r.xbox_name,
        };

        let entry = Season7QuickCashLeaderboardEntry {
            rank: r.rank,
            points: r.points,
        };

        let leaderboard_data = LeaderboardData {
            season_7_quick_cash_leaderboard_entry: Some(entry),
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

    let row: Season7QuickCashRow = serde_json::from_value(Value::Object(obj.clone())).ok()?;
    Some(row.into())
}
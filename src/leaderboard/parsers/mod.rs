use serde::de::Error;
use serde_json::Value;
use crate::PlayerLeaderboardData;

pub mod season_7_ranked_cashout;
pub mod season_7_sponsor;
pub mod season_7_world_tour;
pub mod season_7_terminal_attack;
pub mod season_7_power_shift;
pub mod season_7_quick_cash;
pub mod season_7_team_deathmatch;
pub mod season_7_cash_ball;
// pub mod season_6_ranked_cashout;
// pub mod season_6_sponsor;
// pub mod season_6_world_tour;
// pub mod season_6_terminal_attack;
// pub mod season_6_power_shift;
// pub mod season_6_quick_cash;
// pub mod season_6_team_deathmatch;
// pub mod season_6_heavy_hitters;
// pub mod season_5_ranked_cashout;
// pub mod season_5_sponsor;
// pub mod season_5_world_tour;
// pub mod season_5_terminal_attack;
// pub mod season_5_power_shift;
// pub mod season_5_quick_cash;
// pub mod season_5_bank_it;
// pub mod season_4_ranked_cashout;
// pub mod season_4_sponsor;
// pub mod season_4_world_tour;
// pub mod season_3_ranked_cashout;
// pub mod season_3_world_tour;
// pub mod season_2_ranked_cashout;
// pub mod season_1_ranked_cashout;
// pub mod open_beta;
// pub mod closed_beta_1;
// pub mod closed_beta_2;
// pub mod the_finals;
// pub mod orf;

pub fn parse_slug(slug: &str, row: &Value) -> Option<PlayerLeaderboardData> {
    match slug {
        // Season 7
        "s7" => season_7_ranked_cashout::parse(row),
        "s7s" => season_7_sponsor::parse(row),
        "s7wt" => season_7_world_tour::parse(row),
        "s7ta" => season_7_terminal_attack::parse(row),
        "s7ps" => season_7_power_shift::parse(row),
        "s7qc" => season_7_quick_cash::parse(row),
        "s7tdm" => season_7_team_deathmatch::parse(row),
        "s7cb" => season_7_cash_ball::parse(row),

        // // Season 6
        // "s6" => season_6_ranked_cashout::parse(row),
        // "s6s" => season_6_sponsor::parse(row),
        // "s6wt" => season_6_world_tour::parse(row),
        // "s6ta" => season_6_terminal_attack::parse(row),
        // "s6ps" => season_6_power_shift::parse(row),
        // "s6qc" => season_6_quick_qash::parse(row),
        // "s6tdm" => season_6_team_deathmatch::parse(row),
        // "s6hh" => season_6_heavy_hitters::parse(row),
        //
        // // Season 5
        // "s5" => season_5_ranked_cashout::parse(row),
        // "s5s" => season_5_sponsor::parse(row),
        // "s5wt" => season_5_world_tour::parse(row),
        // "s5ta" => season_5_terminal_attack::parse(row),
        // "s5ps" => season_5_power_shift::parse(row),
        // "s5qc" => season_5_quick_qash::parse(row),
        // "s5bi" => season_5_bank_it::parse(row),
        //
        // // Season 4
        // "s4" => season_4_ranked_cashout::parse(row),
        // "s4s" => season_4_sponsor::parse(row),
        // "s4wt" => season_4_world_tour::parse(row),
        //
        // // Season 3
        // "s3" => season_3_ranked_cashout::parse(row),
        // "s3wt" => season_3_world_tour::parse(row),
        //
        // // Season 2
        // "s2" => season_2_ranked_cashout::parse(row),
        //
        // // Season 1
        // "s1" => season_1_ranked_cashout::parse(row),
        //
        // // Open Beta
        // "ob" => open_beta::parse(row),
        //
        // // Closed Beta
        // "cb1" => closed_beta_1::parse(row),
        // "cb2" => closed_beta_2::parse(row),
        //
        // // The Finals
        // "tf" => the_finals::parse(row),
        //
        // // ÖRF
        // "orf" => orf::parse(row),

        _ => None,
    }
}

fn force_int<'de, D>(d: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    let v = Value::deserialize(d)?;
    match v {
        Value::Number(n) => n
            .as_i64()
            .map(|x| x as i32)
            .ok_or_else(|| Error::custom("not an i64")),
        Value::String(s) if s.trim().is_empty() => Ok(0),
        _ => Err(D::Error::custom("expected int or string")),
    }
}
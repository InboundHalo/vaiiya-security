mod parsers;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use crate::parsers::parse_slug;
use crate::parsers::season_7_cash_ball::Season7CashBallLeaderboardEntry;
use crate::parsers::season_7_power_shift::Season7PowerShiftLeaderboardEntry;
use crate::parsers::season_7_quick_cash::Season7QuickCashLeaderboardEntry;
use crate::parsers::season_7_ranked_cashout::Season7RankedCashoutLeaderboardEntry;
use crate::parsers::season_7_sponsor::Season7SponsorLeaderboardEntry;
use crate::parsers::season_7_team_deathmatch::Season7TeamDeathmatchLeaderboardEntry;
use crate::parsers::season_7_terminal_attack::Season7TerminalAttackLeaderboardEntry;
use crate::parsers::season_7_world_tour::Season7WorldTourLeaderboardEntry;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EmbarkId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInfo {
    pub embark_id:  EmbarkId,
    pub steam_name: Option<String>,
    pub psn_name:   Option<String>,
    pub xbox_name:  Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerLeaderboardData {
    pub player_info: PlayerInfo,
    pub leaderboards: LeaderboardData,
    pub club_tag:   Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LeaderboardData {
    // Season 7
    pub season_7_ranked_cashout_leaderboard_entry: Option<Season7RankedCashoutLeaderboardEntry>,
    pub season_7_sponsor_leaderboard_entry: Option<Season7SponsorLeaderboardEntry>,
    pub season_7_world_tour_leaderboard_entry: Option<Season7WorldTourLeaderboardEntry>,
    pub season_7_terminal_attack_leaderboard_entry: Option<Season7TerminalAttackLeaderboardEntry>,
    pub season_7_power_shift_leaderboard_entry: Option<Season7PowerShiftLeaderboardEntry>,
    pub season_7_quick_cash_leaderboard_entry: Option<Season7QuickCashLeaderboardEntry>,
    pub season_7_team_deathmatch_leaderboard_entry: Option<Season7TeamDeathmatchLeaderboardEntry>,
    pub season_7_cash_ball_leaderboard_entry: Option<Season7CashBallLeaderboardEntry>,

    // // Season 6
    // pub season_6_ranked_cashout_leaderboard_entry: Option<Season6RankedCashoutLeaderboardEntry>,
    // pub season_6_sponsor_leaderboard_entry: Option<Season6SponsorLeaderboardEntry>,
    // pub season_6_world_tour_leaderboard_entry: Option<Season6WorldTourLeaderboardEntry>,
    // pub season_6_terminal_attack_leaderboard_entry: Option<Season6TerminalAttackLeaderboardEntry>,
    // pub season_6_power_shift_leaderboard_entry: Option<Season6PowerShiftLeaderboardEntry>,
    // pub season_6_quick_cash_leaderboard_entry: Option<Season6QuickCashLeaderboardEntry>,
    // pub season_6_team_deathmatch_leaderboard_entry: Option<Season6TeamDeathmatchLeaderboardEntry>,
    // pub season_6_heavy_hitters_leaderboard_entry: Option<Season6HeavyHittersLeaderboardEntry>,
    //
    // // Season 5
    // pub season_5_ranked_cashout_leaderboard_entry: Option<Season5RankedCashoutLeaderboardEntry>,
    // pub season_5_sponsor_leaderboard_entry: Option<Season5SponsorLeaderboardEntry>,
    // pub season_5_world_tour_leaderboard_entry: Option<Season5WorldTourLeaderboardEntry>,
    // pub season_5_terminal_attack_leaderboard_entry: Option<Season5TerminalAttackLeaderboardEntry>,
    // pub season_5_power_shift_leaderboard_entry: Option<Season5PowerShiftLeaderboardEntry>,
    // pub season_5_quick_cash_leaderboard_entry: Option<Season5QuickCashLeaderboardEntry>,
    // pub season_5_bank_it_leaderboard_entry: Option<Season5BankItLeaderboardEntry>,
    //
    // // Season 4
    // pub season_4_ranked_cashout_leaderboard_entry: Option<Season4RankedCashoutLeaderboardEntry>,
    // pub season_4_sponsor_leaderboard_entry: Option<Season4SponsorLeaderboardEntry>,
    // pub season_4_world_tour_leaderboard_entry: Option<Season4WorldTourLeaderboardEntry>,
    //
    // // Season 3
    // pub season_3_ranked_cashout_leaderboard_entry: Option<Season3RankedCashoutLeaderboardEntry>,
    // pub season_3_world_tour_leaderboard_entry: Option<Season3WorldTourLeaderboardEntry>,
    //
    // // Season 2
    // pub season_2_ranked_cashout_leaderboard_entry: Option<Season2RankedCashoutLeaderboardEntry>,
    //
    // // Season 1
    // pub season_1_ranked_cashout_leaderboard_entry: Option<Season1RankedCashoutLeaderboardEntry>,
    //
    // // Open Beta
    // pub open_beta_leaderboard_entry: Option<OpenBetaLeaderboardEntry>,
    //
    // // Closed Beta
    // pub closed_beta_1_leaderboard_entry: Option<ClosedBeta1LeaderboardEntry>,
    // pub closed_beta_2_leaderboard_entry: Option<ClosedBeta2LeaderboardEntry>,
    //
    // // The Finals
    // pub the_finals_leaderboard_entry: Option<TheFinalsLeaderboardEntry>,
    //
    // // ÖRF
    // pub orf_leaderboard_entry: Option<OrfLeaderboardEntry>,
}

impl LeaderboardData {
    pub fn number_of_leaderboards_some(&self) -> i8 {
        let mut count = 0;

        if self.season_7_cash_ball_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_ranked_cashout_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_sponsor_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_world_tour_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_terminal_attack_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_power_shift_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_quick_cash_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_team_deathmatch_leaderboard_entry.is_some() {
            count += 1;
        }

        if self.season_7_cash_ball_leaderboard_entry.is_some() {
            count += 1;
        }

        count
    }

    pub fn merge(&mut self, other: Self) {
        // Macro to avoid repetition
        macro_rules! merge_field {
            ($field:ident) => {
                if self.$field.is_none() {
                    self.$field = other.$field;
                }
            };
        }

        // Season 7
        merge_field!(season_7_ranked_cashout_leaderboard_entry);
        merge_field!(season_7_sponsor_leaderboard_entry);
        merge_field!(season_7_world_tour_leaderboard_entry);
        merge_field!(season_7_terminal_attack_leaderboard_entry);
        merge_field!(season_7_power_shift_leaderboard_entry);
        merge_field!(season_7_quick_cash_leaderboard_entry);
        merge_field!(season_7_team_deathmatch_leaderboard_entry);
        merge_field!(season_7_cash_ball_leaderboard_entry);

        // // Season 6
        // merge_field!(season_6_ranked_cashout_leaderboard_entry);
        // merge_field!(season_6_sponsor_leaderboard_entry);
        // merge_field!(season_6_world_tour_leaderboard_entry);
        // merge_field!(season_6_terminal_attack_leaderboard_entry);
        // merge_field!(season_6_power_shift_leaderboard_entry);
        // merge_field!(season_6_quick_qash_leaderboard_entry);
        // merge_field!(season_6_team_deathmatch_leaderboard_entry);
        // merge_field!(season_6_heavy_hitters_leaderboard_entry);
        //
        // // Season 5
        // merge_field!(season_5_ranked_cashout_leaderboard_entry);
        // merge_field!(season_5_sponsor_leaderboard_entry);
        // merge_field!(season_5_world_tour_leaderboard_entry);
        // merge_field!(season_5_terminal_attack_leaderboard_entry);
        // merge_field!(season_5_power_shift_leaderboard_entry);
        // merge_field!(season_5_quick_qash_leaderboard_entry);
        // merge_field!(season_5_bank_it_leaderboard_entry);
        //
        // // Season 4
        // merge_field!(season_4_ranked_cashout_leaderboard_entry);
        // merge_field!(season_4_sponsor_leaderboard_entry);
        // merge_field!(season_4_world_tour_leaderboard_entry);
        //
        // // Season 3
        // merge_field!(season_3_ranked_cashout_leaderboard_entry);
        // merge_field!(season_3_world_tour_leaderboard_entry);
        //
        // // Season 2
        // merge_field!(season_2_ranked_cashout_leaderboard_entry);
        //
        // // Season 1
        // merge_field!(season_1_ranked_cashout_leaderboard_entry);
        //
        // // Open Beta
        // merge_field!(open_beta_leaderboard_entry);
        //
        // // Closed Beta
        // merge_field!(closed_beta_1_leaderboard_entry);
        // merge_field!(closed_beta_2_leaderboard_entry);
        //
        // // The Finals
        // merge_field!(the_finals_leaderboard_entry);
        //
        // // ÖRF
        // merge_field!(orf_leaderboard_entry);
    }
}

#[derive(Deserialize)]
struct NextData {
    props: NextProps,
}

#[derive(Deserialize)]
struct NextProps {
    #[serde(rename = "pageProps")]
    page_props: PageProps,
}

#[derive(Deserialize)]
struct PageProps {
    metadata: Metadata,
    entries:  Vec<Value>,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    columns: Vec<ColumnDef>,
}

#[derive(Deserialize, Debug)]
struct ColumnDef {
    #[serde(rename = "type")]
    typ:   String,
    order: usize,
}

#[instrument(level = "debug", skip(client))]
async fn fetch_next_data(
    url: &str,
    client: &Client,
) -> Result<NextData, Box<dyn std::error::Error>> {
    info!("fetching HTML for {}", url);
    let html = client.get(url).send().await?.text().await?;
    const MARKER: &str = "id=\"__NEXT_DATA__\" type=\"application/json\">";
    let start = html
        .find(MARKER)
        .ok_or("no NEXT_DATA marker")?
        + MARKER.len();
    let end = html[start..]
        .find("</script>")
        .ok_or("unclosed script")?;
    let blob = &html[start..start + end];

    debug!(blob_len = blob.len(), "extracted NEXT_DATA JSON blob");

    let nd: NextData = serde_json::from_str(blob)?;
    Ok(nd)
}

#[instrument(level = "info", skip(client))]
pub async fn fetch_one(
    slug: &str,
    client: &Client,
) -> HashMap<EmbarkId, PlayerLeaderboardData> {
    info!("starting fetch_one for {}", slug);

    let nd = match fetch_next_data(
        &format!("https://id.embark.games/the-finals/leaderboards/{}", slug),
        client,
    )
        .await
    {
        Ok(nd) => nd,
        Err(err) => {
            error!(%err, "failed to fetch NEXT_DATA for {}", slug);
            return HashMap::new();
        }
    };

    let raw_entries = &nd.props.page_props.entries;
    debug!(
        total_raw_entries = raw_entries.len(),
        "got raw entries for {}", slug
    );

    let mut players: HashMap<EmbarkId, PlayerLeaderboardData> = HashMap::new();

    for (idx, row) in raw_entries.iter().enumerate() {
        match parse_slug(slug, row) {
            Some(mut parsed) => {
                players
                    .entry(parsed.player_info.embark_id.clone())
                    .and_modify(|existing| {
                        existing
                            .leaderboards
                            .merge(parsed.leaderboards.clone());
                        if existing.club_tag.is_none() {
                            existing.club_tag = parsed.club_tag.clone();
                        }
                        if existing.player_info.steam_name.is_none() {
                            existing.player_info.steam_name =
                                parsed.player_info.steam_name.clone();
                        }
                        if existing.player_info.psn_name.is_none() {
                            existing.player_info.psn_name =
                                parsed.player_info.psn_name.clone();
                        }
                        if existing.player_info.xbox_name.is_none() {
                            existing.player_info.xbox_name =
                                parsed.player_info.xbox_name.clone();
                        }
                    })
                    .or_insert(parsed);
            }
            None => warn!(
                "failed to parse row {} for slug {}: {:?}",
                idx,
                slug,
                row
            )
        }
    }

    info!(
        parsed_players = players.len(),
        dropped_rows = raw_entries.len() - players.len(),
        "parsing complete for {}", slug
    );
    players
}

#[instrument(level = "info", skip(slugs))]
pub async fn fetch_all(
    slugs: &[&str],
) -> HashMap<EmbarkId, PlayerLeaderboardData> {
    info!("starting fetch for all slugs");
    let client = Client::builder().build().unwrap();

    let mut tasks = Vec::with_capacity(slugs.len());
    for &slug in slugs {
        let slug = slug.to_owned();
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            let board = fetch_one(&slug, &c).await;
            (slug, board)
        }));
    }

    let mut combined: HashMap<EmbarkId, PlayerLeaderboardData> = HashMap::new();

    for task in futures::future::join_all(tasks).await {
        let (_slug, board) = match task {
            Ok(pair) => pair,
            Err(err) => {
                error!(%err, "tokio task failed");
                continue;
            }
        };

        for (embark, data) in board {
            combined
                .entry(embark.clone())
                .and_modify(|existing| {
                    existing
                        .leaderboards
                        .merge(data.leaderboards.clone());

                    if existing.club_tag.is_none() {
                        existing.club_tag = data.club_tag.clone();
                    }

                    if existing.player_info.steam_name.is_none() {
                        existing.player_info.steam_name =
                            data.player_info.steam_name.clone();
                    }
                    if existing.player_info.psn_name.is_none() {
                        existing.player_info.psn_name =
                            data.player_info.psn_name.clone();
                    }
                    if existing.player_info.xbox_name.is_none() {
                        existing.player_info.xbox_name =
                            data.player_info.xbox_name.clone();
                    }
                })
                .or_insert(data);
        }
    }

    info!(
        total_unique_players = combined.len(),
        "finished fetching all slugs"
    );
    combined
}

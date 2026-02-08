use crate::settings::rewrite::Rewrite;
use crate::settings::timer::{EventTimers, Timer};
use crate::settings::triggers::TriggerRequest;
use autopulse_utils::join_path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Sonarr {
    /// Rewrite path
    pub rewrite: Option<Rewrite>,
    /// Timer settings
    pub timer: Option<Timer>,
    /// Targets to ignore
    #[serde(default)]
    pub excludes: Vec<String>,
    /// Event-specific timers
    pub event_timers: Option<EventTimers>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct EpisodeFile {
    pub relative_path: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct Series {
    pub path: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct RenamedEpisodeFile {
    pub previous_path: String,
    pub relative_path: String,
}
#[derive(Deserialize, Clone)]
#[serde(tag = "eventType")]
#[doc(hidden)]
pub enum SonarrRequest {
    #[serde(rename = "Download")]
    #[serde(rename_all = "camelCase")]
    Download {
        episode_file: Option<EpisodeFile>,
        #[serde(default)]
        deleted_files: Vec<EpisodeFile>,
        series: Series,
    },
    #[serde(rename = "Rename")]
    #[serde(rename_all = "camelCase")]
    Rename {
        series: Series,
        renamed_episode_files: Vec<RenamedEpisodeFile>,
    },
    #[serde(rename = "SeriesDelete")]
    #[serde(rename_all = "camelCase")]
    SeriesDelete { series: Series },
    #[serde(rename = "ImportComplete")]
    #[serde(rename_all = "camelCase")]
    ImportComplete {
        #[serde(default)]
        episode_files: Vec<EpisodeFile>,
        series: Series,
    },
    #[serde(rename = "EpisodeFileDelete")]
    #[serde(rename_all = "camelCase")]
    EpisodeFileDelete {
        episode_file: Option<EpisodeFile>,
        series: Series,
    },
    #[serde(rename = "Grab")]
    Grab,
    #[serde(rename = "SeriesAdd")]
    SeriesAdd,
    #[serde(rename = "Test")]
    Test,
}

impl TriggerRequest for SonarrRequest {
    fn from_json(json: serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(json).map_err(|e| anyhow::anyhow!(e))
    }

    fn paths(&self) -> Vec<(String, bool)> {
        match self {
            Self::EpisodeFileDelete {
                episode_file,
                series,
            } => match episode_file {
                Some(ef) => vec![(join_path(&series.path, &ef.relative_path), false)],
                None => vec![],
            },
            Self::Rename {
                series,
                renamed_episode_files,
            } => {
                let mut paths = vec![];

                for file in renamed_episode_files {
                    paths.push((file.previous_path.clone(), false));
                    paths.push((join_path(&series.path, &file.relative_path), true));
                }

                paths
            }
            Self::SeriesDelete { series } => vec![(series.path.clone(), false)],
            Self::Download {
                episode_file,
                series,
                deleted_files,
            } => {
                let mut paths = vec![];

                if let Some(ef) = episode_file {
                    paths.push((join_path(&series.path, &ef.relative_path), true));
                }

                for file in deleted_files {
                    paths.push((join_path(&series.path, &file.relative_path), false));
                }

                paths
            }
            Self::ImportComplete {
                episode_files,
                series,
            } => {
                episode_files
                    .iter()
                    .map(|f| (join_path(&series.path, &f.relative_path), true))
                    .collect()
            }
            Self::Grab | Self::SeriesAdd | Self::Test => vec![],
        }
    }
}

use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal token rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTokenRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTokenRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal token rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenRollup {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub floor_value: f64,
    pub checksum: String,
    pub sequence: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalTokenRollupStatus,
}

impl SeasonalTokenRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenRollupStatus) -> bool {
        match self.status {
            SeasonalTokenRollupStatus::Proposed => {
                match next {
                    SeasonalTokenRollupStatus::Rejected | SeasonalTokenRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenRollupStatus::Approved => {
                return next == SeasonalTokenRollupStatus::Rejected;
            }
            SeasonalTokenRollupStatus::Rejected => {
                match next {
                    SeasonalTokenRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTokenRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTokenRollup {
        SeasonalTokenRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            floor_value: self.floor_value,
            checksum: self.checksum.clone(),
            sequence: self.sequence,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal token rollup in status proposed.
pub fn new_seasonal_token_rollup(code: &str, name: &str) -> SeasonalTokenRollup {
    SeasonalTokenRollup {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        floor_value: 0.0,
        checksum: "".to_string(),
        sequence: 0,
        attributes: BTreeMap::new(),
        status: SeasonalTokenRollupStatus::Proposed,
    }
}

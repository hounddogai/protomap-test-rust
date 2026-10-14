use super::*;

use acme_kit::text;

/// Where a seasonal retention preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRetentionPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalRetentionPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalRetentionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal retention preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreference {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub owner_id: String,
    pub scope: String,
    pub host: String,
    pub retries: i64,
    pub grace_days: i64,
    pub region: String,
    pub status: SeasonalRetentionPreferenceStatus,
}

impl SeasonalRetentionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRetentionPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRetentionPreferenceStatus) -> bool {
        match self.status {
            SeasonalRetentionPreferenceStatus::Created => {
                match next {
                    SeasonalRetentionPreferenceStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRetentionPreferenceStatus::Validated => {
                return next == SeasonalRetentionPreferenceStatus::Retired;
            }
            SeasonalRetentionPreferenceStatus::Published => {
                match next {
                    SeasonalRetentionPreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalRetentionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal retention preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal retention preference", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.retries as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRetentionPreference {
        SeasonalRetentionPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            owner_id: self.owner_id.clone(),
            scope: self.scope.clone(),
            host: self.host.clone(),
            retries: self.retries,
            grace_days: self.grace_days,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal retention preference in status created.
pub fn new_seasonal_retention_preference(code: &str, name: &str) -> SeasonalRetentionPreference {
    SeasonalRetentionPreference {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        owner_id: "".to_string(),
        scope: "".to_string(),
        host: "".to_string(),
        retries: 0,
        grace_days: 0,
        region: "".to_string(),
        status: SeasonalRetentionPreferenceStatus::Created,
    }
}

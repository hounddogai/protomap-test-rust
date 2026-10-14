use super::*;

use acme_kit::text;

/// Where a seasonal quote manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQuoteManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalQuoteManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalQuoteManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal quote manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifest {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub quota_units: i64,
    pub label: String,
    pub unit: String,
    pub status: SeasonalQuoteManifestStatus,
}

impl SeasonalQuoteManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQuoteManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQuoteManifestStatus) -> bool {
        match self.status {
            SeasonalQuoteManifestStatus::Planned => {
                return next == SeasonalQuoteManifestStatus::Scheduled
                    || next == SeasonalQuoteManifestStatus::Completed;
            }
            SeasonalQuoteManifestStatus::Scheduled => {
                match next {
                    SeasonalQuoteManifestStatus::InProgress | SeasonalQuoteManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQuoteManifestStatus::InProgress => {
                match next {
                    SeasonalQuoteManifestStatus::Completed | SeasonalQuoteManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQuoteManifestStatus::Completed => {
                match next {
                    SeasonalQuoteManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalQuoteManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal quote manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ceiling + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalQuoteManifest {
        SeasonalQuoteManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            quota_units: self.quota_units,
            label: self.label.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal quote manifest in status planned.
pub fn new_seasonal_quote_manifest(code: &str, title: &str) -> SeasonalQuoteManifest {
    SeasonalQuoteManifest {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        quota_units: 0,
        label: "".to_string(),
        unit: "".to_string(),
        status: SeasonalQuoteManifestStatus::Planned,
    }
}

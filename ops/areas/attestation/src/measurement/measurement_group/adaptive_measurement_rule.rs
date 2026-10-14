use super::*;

use acme_kit::text;

/// Where a adaptive measurement rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMeasurementRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveMeasurementRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveMeasurementRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive measurement rule as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRule {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub version: i64,
    pub cluster: String,
    pub status: AdaptiveMeasurementRuleStatus,
}

impl AdaptiveMeasurementRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMeasurementRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMeasurementRuleStatus) -> bool {
        match self.status {
            AdaptiveMeasurementRuleStatus::Open => {
                return next == AdaptiveMeasurementRuleStatus::Archived;
            }
            AdaptiveMeasurementRuleStatus::InReview => {
                match next {
                    AdaptiveMeasurementRuleStatus::Reopened | AdaptiveMeasurementRuleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementRuleStatus::Resolved => {
                return next == AdaptiveMeasurementRuleStatus::Archived;
            }
            AdaptiveMeasurementRuleStatus::Reopened => {
                match next {
                    AdaptiveMeasurementRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveMeasurementRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive measurement rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.version as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveMeasurementRule {
        AdaptiveMeasurementRule {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            version: self.version,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive measurement rule in status open.
pub fn new_adaptive_measurement_rule(code: &str, name: &str) -> AdaptiveMeasurementRule {
    AdaptiveMeasurementRule {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        version: 0,
        cluster: "".to_string(),
        status: AdaptiveMeasurementRuleStatus::Open,
    }
}

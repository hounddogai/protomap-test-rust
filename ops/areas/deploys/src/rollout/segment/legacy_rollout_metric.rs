use super::*;

use acme_kit::text;

/// Where a legacy rollout metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyRolloutMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyRolloutMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetric {
    pub code: String,
    pub title: String,
    pub label: String,
    pub floor_value: f64,
    pub budget: f64,
    pub owner_id: String,
    pub principal: String,
    pub status: LegacyRolloutMetricStatus,
}

impl LegacyRolloutMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutMetricStatus) -> bool {
        match self.status {
            LegacyRolloutMetricStatus::Created => {
                match next {
                    LegacyRolloutMetricStatus::Validated | LegacyRolloutMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutMetricStatus::Validated => {
                match next {
                    LegacyRolloutMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutMetricStatus::Published => {
                match next {
                    LegacyRolloutMetricStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyRolloutMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy rollout metric", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.floor_value - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutMetric {
        LegacyRolloutMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            budget: self.budget,
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy rollout metric in status created.
pub fn new_legacy_rollout_metric(code: &str, title: &str) -> LegacyRolloutMetric {
    LegacyRolloutMetric {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        floor_value: 0.0,
        budget: 0.0,
        owner_id: "".to_string(),
        principal: "".to_string(),
        status: LegacyRolloutMetricStatus::Created,
    }
}

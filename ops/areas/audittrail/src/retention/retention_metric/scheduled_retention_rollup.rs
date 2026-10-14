use super::*;

use acme_kit::text;

/// Where a scheduled retention rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRetentionRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledRetentionRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledRetentionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled retention rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionRollup {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub burst: i64,
    pub ratio: f64,
    pub status: ScheduledRetentionRollupStatus,
}

impl ScheduledRetentionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRetentionRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRetentionRollupStatus) -> bool {
        match self.status {
            ScheduledRetentionRollupStatus::Created => {
                return next == ScheduledRetentionRollupStatus::Published;
            }
            ScheduledRetentionRollupStatus::Validated => {
                match next {
                    ScheduledRetentionRollupStatus::Published | ScheduledRetentionRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRetentionRollupStatus::Published => {
                match next {
                    ScheduledRetentionRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledRetentionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled retention rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled retention rollup", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRetentionRollup {
        ScheduledRetentionRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            burst: self.burst,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled retention rollup in status created.
pub fn new_scheduled_retention_rollup(code: &str, label: &str) -> ScheduledRetentionRollup {
    ScheduledRetentionRollup {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        burst: 0,
        ratio: 0.0,
        status: ScheduledRetentionRollupStatus::Created,
    }
}

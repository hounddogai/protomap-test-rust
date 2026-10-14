use super::*;

use acme_kit::text;

/// Where a manual redaction rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualRedactionRollupStatus {
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

impl std::fmt::Display for ManualRedactionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual redaction rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionRollup {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub ends_at: i64,
    pub contact_email: String,
    pub weight: f64,
    pub quota_units: i64,
    pub ceiling: f64,
    pub cost: f64,
    pub status: ManualRedactionRollupStatus,
}

impl ManualRedactionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionRollupStatus) -> bool {
        match self.status {
            ManualRedactionRollupStatus::Planned => {
                return next == ManualRedactionRollupStatus::Cancelled;
            }
            ManualRedactionRollupStatus::Scheduled => {
                return next == ManualRedactionRollupStatus::InProgress
                    || next == ManualRedactionRollupStatus::Completed;
            }
            ManualRedactionRollupStatus::InProgress => {
                return next == ManualRedactionRollupStatus::Cancelled;
            }
            ManualRedactionRollupStatus::Completed => {
                match next {
                    ManualRedactionRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualRedactionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ends_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionRollup {
        ManualRedactionRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            weight: self.weight,
            quota_units: self.quota_units,
            ceiling: self.ceiling,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual redaction rollup in status planned.
pub fn new_manual_redaction_rollup(code: &str, title: &str) -> ManualRedactionRollup {
    ManualRedactionRollup {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        ends_at: 0,
        contact_email: "".to_string(),
        weight: 0.0,
        quota_units: 0,
        ceiling: 0.0,
        cost: 0.0,
        status: ManualRedactionRollupStatus::Planned,
    }
}

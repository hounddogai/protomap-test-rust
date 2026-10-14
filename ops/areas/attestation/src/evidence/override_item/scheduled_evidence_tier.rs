use super::*;

use acme_kit::text;

/// Where a scheduled evidence tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEvidenceTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledEvidenceTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledEvidenceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled evidence tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTier {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub ends_at: i64,
    pub floor_value: f64,
    pub weight: f64,
    pub archived: bool,
    pub capacity: i64,
    pub quota_units: i64,
    pub label: String,
    pub ceiling: f64,
    pub status: ScheduledEvidenceTierStatus,
}

impl ScheduledEvidenceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEvidenceTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEvidenceTierStatus) -> bool {
        match self.status {
            ScheduledEvidenceTierStatus::Created => {
                match next {
                    ScheduledEvidenceTierStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEvidenceTierStatus::Validated => {
                match next {
                    ScheduledEvidenceTierStatus::Published | ScheduledEvidenceTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEvidenceTierStatus::Published => {
                return next == ScheduledEvidenceTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledEvidenceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled evidence tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled evidence tier", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.retries as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledEvidenceTier {
        ScheduledEvidenceTier {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            weight: self.weight,
            archived: self.archived,
            capacity: self.capacity,
            quota_units: self.quota_units,
            label: self.label.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled evidence tier in status created.
pub fn new_scheduled_evidence_tier(code: &str, name: &str) -> ScheduledEvidenceTier {
    ScheduledEvidenceTier {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        ends_at: 0,
        floor_value: 0.0,
        weight: 0.0,
        archived: false,
        capacity: 0,
        quota_units: 0,
        label: "".to_string(),
        ceiling: 0.0,
        status: ScheduledEvidenceTierStatus::Created,
    }
}

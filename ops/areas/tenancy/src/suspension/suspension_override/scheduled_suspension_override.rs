use super::*;

use acme_kit::text;

/// Where a scheduled suspension override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledSuspensionOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledSuspensionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled suspension override as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverride {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub starts_at: i64,
    pub cost: f64,
    pub principal: String,
    pub notes: String,
    pub ceiling: f64,
    pub archived: bool,
    pub cipher: String,
    pub status: ScheduledSuspensionOverrideStatus,
}

impl ScheduledSuspensionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionOverrideStatus) -> bool {
        match self.status {
            ScheduledSuspensionOverrideStatus::Proposed => {
                return next == ScheduledSuspensionOverrideStatus::Approved;
            }
            ScheduledSuspensionOverrideStatus::Approved => {
                match next {
                    ScheduledSuspensionOverrideStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSuspensionOverrideStatus::Rejected => {
                return next == ScheduledSuspensionOverrideStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSuspensionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled suspension override", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.starts_at as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionOverride {
        ScheduledSuspensionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            starts_at: self.starts_at,
            cost: self.cost,
            principal: self.principal.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            archived: self.archived,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled suspension override in status proposed.
pub fn new_scheduled_suspension_override(code: &str, title: &str) -> ScheduledSuspensionOverride {
    ScheduledSuspensionOverride {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        starts_at: 0,
        cost: 0.0,
        principal: "".to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        archived: false,
        cipher: "".to_string(),
        status: ScheduledSuspensionOverrideStatus::Proposed,
    }
}

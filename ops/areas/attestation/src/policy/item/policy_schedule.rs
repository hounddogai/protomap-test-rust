use super::*;

use acme_kit::text;

/// Where a policy schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PolicyScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PolicyScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySchedule {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub source: String,
    pub limit: i64,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub status: PolicyScheduleStatus,
}

impl PolicySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyScheduleStatus) -> bool {
        match self.status {
            PolicyScheduleStatus::Draft => {
                match next {
                    PolicyScheduleStatus::Pending | PolicyScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyScheduleStatus::Pending => {
                return next == PolicyScheduleStatus::Active;
            }
            PolicyScheduleStatus::Active => {
                return next == PolicyScheduleStatus::Closed;
            }
            PolicyScheduleStatus::Suspended => {
                return next == PolicyScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicySchedule {
        PolicySchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            limit: self.limit,
            tags: self.tags.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a policy schedule in status draft.
pub fn new_policy_schedule(code: &str, label: &str) -> PolicySchedule {
    PolicySchedule {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        source: "".to_string(),
        limit: 0,
        tags: Vec::new(),
        pinned: false,
        status: PolicyScheduleStatus::Draft,
    }
}

use super::*;

use acme_kit::text;

/// Where a probe milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeMilestoneStatus {
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

impl std::fmt::Display for ProbeMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMilestone {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub sequence: i64,
    pub key_id: String,
    pub ends_at: i64,
    pub namespace: String,
    pub locale: String,
    pub grace_days: i64,
    pub status: ProbeMilestoneStatus,
}

impl ProbeMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeMilestoneStatus) -> bool {
        match self.status {
            ProbeMilestoneStatus::Draft => {
                match next {
                    ProbeMilestoneStatus::Active | ProbeMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeMilestoneStatus::Pending => {
                return next == ProbeMilestoneStatus::Closed;
            }
            ProbeMilestoneStatus::Active => {
                return next == ProbeMilestoneStatus::Closed;
            }
            ProbeMilestoneStatus::Suspended => {
                match next {
                    ProbeMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: ProbeMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe milestone {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeMilestone {
        ProbeMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            locale: self.locale.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a probe milestone in status draft.
pub fn new_probe_milestone(code: &str, name: &str) -> ProbeMilestone {
    ProbeMilestone {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        sequence: 0,
        key_id: "".to_string(),
        ends_at: 0,
        namespace: "".to_string(),
        locale: "".to_string(),
        grace_days: 0,
        status: ProbeMilestoneStatus::Draft,
    }
}

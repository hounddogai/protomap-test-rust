use super::*;

use acme_kit::text;

/// Where a allowlist milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistMilestoneStatus {
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

impl std::fmt::Display for AllowlistMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist milestone as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestone {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub ratio: f64,
    pub attempts: i64,
    pub scope: String,
    pub timezone: String,
    pub weight: f64,
    pub margin: f64,
    pub cipher: String,
    pub locale: String,
    pub status: AllowlistMilestoneStatus,
}

impl AllowlistMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistMilestoneStatus) -> bool {
        match self.status {
            AllowlistMilestoneStatus::Draft => {
                match next {
                    AllowlistMilestoneStatus::Pending | AllowlistMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistMilestoneStatus::Pending => {
                return next == AllowlistMilestoneStatus::Active || next == AllowlistMilestoneStatus::Closed;
            }
            AllowlistMilestoneStatus::Active => {
                match next {
                    AllowlistMilestoneStatus::Suspended | AllowlistMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistMilestoneStatus::Suspended => {
                return next == AllowlistMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist milestone", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ratio - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistMilestone {
        AllowlistMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            ratio: self.ratio,
            attempts: self.attempts,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            weight: self.weight,
            margin: self.margin,
            cipher: self.cipher.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a allowlist milestone in status draft.
pub fn new_allowlist_milestone(code: &str, name: &str) -> AllowlistMilestone {
    AllowlistMilestone {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        ratio: 0.0,
        attempts: 0,
        scope: "".to_string(),
        timezone: "".to_string(),
        weight: 0.0,
        margin: 0.0,
        cipher: "".to_string(),
        locale: "".to_string(),
        status: AllowlistMilestoneStatus::Draft,
    }
}

use super::*;

use acme_kit::text;

/// Where a regional runbook plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRunbookPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalRunbookPlanStatus {
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

impl std::fmt::Display for RegionalRunbookPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional runbook plan as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlan {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub version: i64,
    pub expires_at: i64,
    pub visible: bool,
    pub revision: i64,
    pub external_ref: String,
    pub owner_id: String,
    pub cost: f64,
    pub active: bool,
    pub status: RegionalRunbookPlanStatus,
}

impl RegionalRunbookPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRunbookPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRunbookPlanStatus) -> bool {
        match self.status {
            RegionalRunbookPlanStatus::Planned => {
                match next {
                    RegionalRunbookPlanStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookPlanStatus::Scheduled => {
                match next {
                    RegionalRunbookPlanStatus::InProgress | RegionalRunbookPlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookPlanStatus::InProgress => {
                return next == RegionalRunbookPlanStatus::Completed;
            }
            RegionalRunbookPlanStatus::Completed => {
                match next {
                    RegionalRunbookPlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalRunbookPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional runbook plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.revision as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRunbookPlan {
        RegionalRunbookPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            version: self.version,
            expires_at: self.expires_at,
            visible: self.visible,
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional runbook plan in status planned.
pub fn new_regional_runbook_plan(code: &str, label: &str) -> RegionalRunbookPlan {
    RegionalRunbookPlan {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        version: 0,
        expires_at: 0,
        visible: false,
        revision: 0,
        external_ref: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        active: false,
        status: RegionalRunbookPlanStatus::Planned,
    }
}

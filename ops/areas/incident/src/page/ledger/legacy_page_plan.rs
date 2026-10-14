use super::*;

use acme_kit::text;

/// Where a legacy page plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPagePlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyPagePlanStatus {
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

impl std::fmt::Display for LegacyPagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy page plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPagePlan {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub priority: i64,
    pub expires_at: i64,
    pub revision: i64,
    pub ends_at: i64,
    pub external_ref: String,
    pub ratio: f64,
    pub contact_email: String,
    pub amount: f64,
    pub status: LegacyPagePlanStatus,
}

impl LegacyPagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPagePlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPagePlanStatus) -> bool {
        match self.status {
            LegacyPagePlanStatus::Planned => {
                match next {
                    LegacyPagePlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPagePlanStatus::Scheduled => {
                match next {
                    LegacyPagePlanStatus::Completed | LegacyPagePlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPagePlanStatus::InProgress => {
                match next {
                    LegacyPagePlanStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPagePlanStatus::Completed => {
                match next {
                    LegacyPagePlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyPagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy page plan {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPagePlan {
        LegacyPagePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            priority: self.priority,
            expires_at: self.expires_at,
            revision: self.revision,
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy page plan in status planned.
pub fn new_legacy_page_plan(code: &str, name: &str) -> LegacyPagePlan {
    LegacyPagePlan {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        priority: 0,
        expires_at: 0,
        revision: 0,
        ends_at: 0,
        external_ref: "".to_string(),
        ratio: 0.0,
        contact_email: "".to_string(),
        amount: 0.0,
        status: LegacyPagePlanStatus::Planned,
    }
}

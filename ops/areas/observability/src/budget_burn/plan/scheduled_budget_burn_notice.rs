use super::*;

use acme_kit::text;

/// Where a scheduled budget burn notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledBudgetBurnNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledBudgetBurnNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledBudgetBurnNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled budget burn notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnNotice {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub contact_email: String,
    pub region: String,
    pub score: f64,
    pub status: ScheduledBudgetBurnNoticeStatus,
}

impl ScheduledBudgetBurnNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledBudgetBurnNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledBudgetBurnNoticeStatus) -> bool {
        match self.status {
            ScheduledBudgetBurnNoticeStatus::Idle => {
                return next == ScheduledBudgetBurnNoticeStatus::Warming
                    || next == ScheduledBudgetBurnNoticeStatus::Draining;
            }
            ScheduledBudgetBurnNoticeStatus::Warming => {
                match next {
                    ScheduledBudgetBurnNoticeStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBudgetBurnNoticeStatus::Ready => {
                match next {
                    ScheduledBudgetBurnNoticeStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBudgetBurnNoticeStatus::Draining => {
                return next == ScheduledBudgetBurnNoticeStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledBudgetBurnNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled budget burn notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled budget burn notice", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ScheduledBudgetBurnNotice {
        ScheduledBudgetBurnNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            region: self.region.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled budget burn notice in status idle.
pub fn new_scheduled_budget_burn_notice(code: &str, name: &str) -> ScheduledBudgetBurnNotice {
    ScheduledBudgetBurnNotice {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        region: "".to_string(),
        score: 0.0,
        status: ScheduledBudgetBurnNoticeStatus::Idle,
    }
}

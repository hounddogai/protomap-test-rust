use super::*;

use acme_kit::text;

/// Where a plan audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanAuditStatus {
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

impl std::fmt::Display for PlanAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAudit {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub quota_units: i64,
    pub threshold: f64,
    pub active: bool,
    pub status: PlanAuditStatus,
}

impl PlanAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanAuditStatus) -> bool {
        match self.status {
            PlanAuditStatus::Idle => {
                return next == PlanAuditStatus::Draining || next == PlanAuditStatus::Stopped;
            }
            PlanAuditStatus::Warming => {
                return next == PlanAuditStatus::Draining || next == PlanAuditStatus::Stopped;
            }
            PlanAuditStatus::Ready => {
                return next == PlanAuditStatus::Draining || next == PlanAuditStatus::Stopped;
            }
            PlanAuditStatus::Draining => {
                match next {
                    PlanAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quota_units as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanAudit {
        PlanAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            quota_units: self.quota_units,
            threshold: self.threshold,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan audit in status idle.
pub fn new_plan_audit(code: &str, name: &str) -> PlanAudit {
    PlanAudit {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        quota_units: 0,
        threshold: 0.0,
        active: false,
        status: PlanAuditStatus::Idle,
    }
}

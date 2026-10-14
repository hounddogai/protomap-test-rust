use super::*;

use acme_kit::text;

/// Where a manual rollout audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRolloutAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRolloutAuditStatus {
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

impl std::fmt::Display for ManualRolloutAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rollout audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutAudit {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub name: String,
    pub grace_days: i64,
    pub principal: String,
    pub status: ManualRolloutAuditStatus,
}

impl ManualRolloutAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRolloutAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRolloutAuditStatus) -> bool {
        match self.status {
            ManualRolloutAuditStatus::Idle => {
                return next == ManualRolloutAuditStatus::Warming || next == ManualRolloutAuditStatus::Stopped;
            }
            ManualRolloutAuditStatus::Warming => {
                match next {
                    ManualRolloutAuditStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRolloutAuditStatus::Ready => {
                return next == ManualRolloutAuditStatus::Draining || next == ManualRolloutAuditStatus::Stopped;
            }
            ManualRolloutAuditStatus::Draining => {
                return next == ManualRolloutAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRolloutAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollout audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRolloutAudit {
        ManualRolloutAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            name: self.name.clone(),
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual rollout audit in status idle.
pub fn new_manual_rollout_audit(code: &str, label: &str) -> ManualRolloutAudit {
    ManualRolloutAudit {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        name: "".to_string(),
        grace_days: 0,
        principal: "".to_string(),
        status: ManualRolloutAuditStatus::Idle,
    }
}

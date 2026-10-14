use super::*;

use acme_kit::text;

/// Where a pending health check rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingHealthCheckRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingHealthCheckRuleStatus {
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

impl std::fmt::Display for PendingHealthCheckRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending health check rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRule {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub segment: String,
    pub revision: i64,
    pub sequence: i64,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub status: PendingHealthCheckRuleStatus,
}

impl PendingHealthCheckRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingHealthCheckRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingHealthCheckRuleStatus) -> bool {
        match self.status {
            PendingHealthCheckRuleStatus::Idle => {
                return next == PendingHealthCheckRuleStatus::Draining || next == PendingHealthCheckRuleStatus::Stopped;
            }
            PendingHealthCheckRuleStatus::Warming => {
                return next == PendingHealthCheckRuleStatus::Ready;
            }
            PendingHealthCheckRuleStatus::Ready => {
                match next {
                    PendingHealthCheckRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckRuleStatus::Draining => {
                match next {
                    PendingHealthCheckRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingHealthCheckRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending health check rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingHealthCheckRule {
        PendingHealthCheckRule {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            revision: self.revision,
            sequence: self.sequence,
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending health check rule in status idle.
pub fn new_pending_health_check_rule(code: &str, label: &str) -> PendingHealthCheckRule {
    PendingHealthCheckRule {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        segment: "".to_string(),
        revision: 0,
        sequence: 0,
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        status: PendingHealthCheckRuleStatus::Idle,
    }
}

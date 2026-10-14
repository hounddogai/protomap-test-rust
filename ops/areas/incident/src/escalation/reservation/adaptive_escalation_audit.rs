use super::*;

use acme_kit::text;

/// Where a adaptive escalation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveEscalationAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveEscalationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveEscalationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive escalation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAudit {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub attempts: i64,
    pub amount: f64,
    pub status: AdaptiveEscalationAuditStatus,
}

impl AdaptiveEscalationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveEscalationAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveEscalationAuditStatus) -> bool {
        match self.status {
            AdaptiveEscalationAuditStatus::New => {
                match next {
                    AdaptiveEscalationAuditStatus::Running | AdaptiveEscalationAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveEscalationAuditStatus::Queued => {
                match next {
                    AdaptiveEscalationAuditStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveEscalationAuditStatus::Running => {
                return next == AdaptiveEscalationAuditStatus::Done || next == AdaptiveEscalationAuditStatus::Failed;
            }
            AdaptiveEscalationAuditStatus::Done => {
                match next {
                    AdaptiveEscalationAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveEscalationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive escalation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveEscalationAudit {
        AdaptiveEscalationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            attempts: self.attempts,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive escalation audit in status new.
pub fn new_adaptive_escalation_audit(code: &str, title: &str) -> AdaptiveEscalationAudit {
    AdaptiveEscalationAudit {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        limit: 0,
        attempts: 0,
        amount: 0.0,
        status: AdaptiveEscalationAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_escalation_audit("code-140", "title-936");
        assert_eq!(item.status, AdaptiveEscalationAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_escalation_audit("code-140", "title-936");
        if item.move_to(AdaptiveEscalationAuditStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveEscalationAuditStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_escalation_audit("code-140", "title-936");
        item.status = AdaptiveEscalationAuditStatus::Failed;
        assert!(
            item.move_to(AdaptiveEscalationAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

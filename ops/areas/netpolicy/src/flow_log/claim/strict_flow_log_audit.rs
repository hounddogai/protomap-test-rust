use super::*;

use acme_kit::text;

/// Where a strict flow log audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFlowLogAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictFlowLogAuditStatus {
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

impl std::fmt::Display for StrictFlowLogAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict flow log audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogAudit {
    pub code: String,
    pub name: String,
    pub category: String,
    pub retries: i64,
    pub floor_value: f64,
    pub version: i64,
    pub issuer: String,
    pub ends_at: i64,
    pub amount: f64,
    pub status: StrictFlowLogAuditStatus,
}

impl StrictFlowLogAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFlowLogAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFlowLogAuditStatus) -> bool {
        match self.status {
            StrictFlowLogAuditStatus::New => {
                return next == StrictFlowLogAuditStatus::Done;
            }
            StrictFlowLogAuditStatus::Queued => {
                match next {
                    StrictFlowLogAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFlowLogAuditStatus::Running => {
                return next == StrictFlowLogAuditStatus::Done || next == StrictFlowLogAuditStatus::Failed;
            }
            StrictFlowLogAuditStatus::Done => {
                return next == StrictFlowLogAuditStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFlowLogAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict flow log audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.amount * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFlowLogAudit {
        StrictFlowLogAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            retries: self.retries,
            floor_value: self.floor_value,
            version: self.version,
            issuer: self.issuer.clone(),
            ends_at: self.ends_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict flow log audit in status new.
pub fn new_strict_flow_log_audit(code: &str, name: &str) -> StrictFlowLogAudit {
    StrictFlowLogAudit {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        retries: 0,
        floor_value: 0.0,
        version: 0,
        issuer: "".to_string(),
        ends_at: 0,
        amount: 0.0,
        status: StrictFlowLogAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_flow_log_audit("code-683", "name-483");
        assert_eq!(item.status, StrictFlowLogAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_strict_flow_log_audit("code-683", "name-483");
        if item.move_to(StrictFlowLogAuditStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFlowLogAuditStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_flow_log_audit("code-683", "name-483");
        item.status = StrictFlowLogAuditStatus::Failed;
        assert!(
            item.move_to(StrictFlowLogAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

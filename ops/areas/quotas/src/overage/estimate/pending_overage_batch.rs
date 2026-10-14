use super::*;

use acme_kit::text;

/// Where a pending overage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingOverageBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingOverageBatchStatus {
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

impl std::fmt::Display for PendingOverageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending overage batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatch {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub threshold: f64,
    pub rotation_days: i64,
    pub ceiling: f64,
    pub status: PendingOverageBatchStatus,
}

impl PendingOverageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingOverageBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingOverageBatchStatus) -> bool {
        match self.status {
            PendingOverageBatchStatus::Idle => {
                match next {
                    PendingOverageBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingOverageBatchStatus::Warming => {
                return next == PendingOverageBatchStatus::Ready;
            }
            PendingOverageBatchStatus::Ready => {
                return next == PendingOverageBatchStatus::Draining || next == PendingOverageBatchStatus::Stopped;
            }
            PendingOverageBatchStatus::Draining => {
                match next {
                    PendingOverageBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingOverageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending overage batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.threshold - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingOverageBatch {
        PendingOverageBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            threshold: self.threshold,
            rotation_days: self.rotation_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending overage batch in status idle.
pub fn new_pending_overage_batch(code: &str, label: &str) -> PendingOverageBatch {
    PendingOverageBatch {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        threshold: 0.0,
        rotation_days: 0,
        ceiling: 0.0,
        status: PendingOverageBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_overage_batch("code-220", "label-308");
        assert_eq!(item.status, PendingOverageBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_pending_overage_batch("code-220", "label-308");
        if item.move_to(PendingOverageBatchStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingOverageBatchStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_overage_batch("code-220", "label-308");
        item.status = PendingOverageBatchStatus::Stopped;
        assert!(
            item.move_to(PendingOverageBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

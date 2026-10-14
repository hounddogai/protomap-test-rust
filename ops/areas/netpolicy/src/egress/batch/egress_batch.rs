use super::*;

use acme_kit::text;

/// Where a egress batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EgressBatchStatus {
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

impl std::fmt::Display for EgressBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressBatch {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub priority: i64,
    pub contact_email: String,
    pub amount: f64,
    pub attempts: i64,
    pub description: String,
    pub audience: String,
    pub margin: f64,
    pub status: EgressBatchStatus,
}

impl EgressBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressBatchStatus) -> bool {
        match self.status {
            EgressBatchStatus::Idle => {
                match next {
                    EgressBatchStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBatchStatus::Warming => {
                return next == EgressBatchStatus::Draining;
            }
            EgressBatchStatus::Ready => {
                match next {
                    EgressBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBatchStatus::Draining => {
                return next == EgressBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressBatch {
        EgressBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            priority: self.priority,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            attempts: self.attempts,
            description: self.description.clone(),
            audience: self.audience.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress batch in status idle.
pub fn new_egress_batch(code: &str, label: &str) -> EgressBatch {
    EgressBatch {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        priority: 0,
        contact_email: "".to_string(),
        amount: 0.0,
        attempts: 0,
        description: "".to_string(),
        audience: "".to_string(),
        margin: 0.0,
        status: EgressBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_egress_batch("code-484", "label-825");
        assert_eq!(item.status, EgressBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_egress_batch("code-484", "label-825");
        if item.move_to(EgressBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_egress_batch("code-484", "label-825");
        item.status = EgressBatchStatus::Stopped;
        assert!(item.move_to(EgressBatchStatus::Idle).is_err(), "expected move to fail");
    }
}

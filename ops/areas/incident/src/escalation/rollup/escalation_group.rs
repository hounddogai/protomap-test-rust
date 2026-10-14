use super::*;

use acme_kit::text;

/// Where a escalation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationGroupStatus {
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

impl std::fmt::Display for EscalationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationGroup {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub amount: f64,
    pub maximum: f64,
    pub status: EscalationGroupStatus,
}

impl EscalationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationGroupStatus) -> bool {
        match self.status {
            EscalationGroupStatus::New => {
                match next {
                    EscalationGroupStatus::Queued | EscalationGroupStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationGroupStatus::Queued => {
                match next {
                    EscalationGroupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationGroupStatus::Running => {
                return next == EscalationGroupStatus::Done;
            }
            EscalationGroupStatus::Done => {
                match next {
                    EscalationGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: EscalationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation group", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.discount + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationGroup {
        EscalationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            amount: self.amount,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation group in status new.
pub fn new_escalation_group(code: &str, label: &str) -> EscalationGroup {
    EscalationGroup {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        amount: 0.0,
        maximum: 0.0,
        status: EscalationGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_group("code-25", "label-527");
        assert_eq!(item.status, EscalationGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_escalation_group("code-25", "label-527");
        if item.move_to(EscalationGroupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationGroupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_group("code-25", "label-527");
        item.status = EscalationGroupStatus::Failed;
        assert!(
            item.move_to(EscalationGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

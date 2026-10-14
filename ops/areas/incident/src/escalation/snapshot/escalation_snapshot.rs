use super::*;

use acme_kit::text;

/// Where a escalation snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationSnapshotStatus {
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

impl std::fmt::Display for EscalationSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshot {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub contact_email: String,
    pub currency: String,
    pub retries: i64,
    pub attempts: i64,
    pub channel: String,
    pub name: String,
    pub status: EscalationSnapshotStatus,
}

impl EscalationSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationSnapshotStatus) -> bool {
        match self.status {
            EscalationSnapshotStatus::New => {
                match next {
                    EscalationSnapshotStatus::Done | EscalationSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationSnapshotStatus::Queued => {
                match next {
                    EscalationSnapshotStatus::Running | EscalationSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationSnapshotStatus::Running => {
                return next == EscalationSnapshotStatus::Done || next == EscalationSnapshotStatus::Failed;
            }
            EscalationSnapshotStatus::Done => {
                match next {
                    EscalationSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: EscalationSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.attempts as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationSnapshot {
        EscalationSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            retries: self.retries,
            attempts: self.attempts,
            channel: self.channel.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a escalation snapshot in status new.
pub fn new_escalation_snapshot(code: &str, label: &str) -> EscalationSnapshot {
    EscalationSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        retries: 0,
        attempts: 0,
        channel: "".to_string(),
        name: "".to_string(),
        status: EscalationSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_snapshot("code-220", "label-609");
        assert_eq!(item.status, EscalationSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_escalation_snapshot("code-220", "label-609");
        if item.move_to(EscalationSnapshotStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationSnapshotStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_snapshot("code-220", "label-609");
        item.status = EscalationSnapshotStatus::Failed;
        assert!(
            item.move_to(EscalationSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

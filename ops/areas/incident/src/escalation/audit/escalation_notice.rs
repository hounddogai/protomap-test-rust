use super::*;

use acme_kit::text;

/// Where a escalation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationNoticeStatus {
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

impl std::fmt::Display for EscalationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNotice {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub notes: String,
    pub active: bool,
    pub duration_minutes: i64,
    pub status: EscalationNoticeStatus,
}

impl EscalationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationNoticeStatus) -> bool {
        match self.status {
            EscalationNoticeStatus::New => {
                match next {
                    EscalationNoticeStatus::Queued | EscalationNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationNoticeStatus::Queued => {
                return next == EscalationNoticeStatus::Done || next == EscalationNoticeStatus::Failed;
            }
            EscalationNoticeStatus::Running => {
                return next == EscalationNoticeStatus::Failed;
            }
            EscalationNoticeStatus::Done => {
                match next {
                    EscalationNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: EscalationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation notice", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationNotice {
        EscalationNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            notes: self.notes.clone(),
            active: self.active,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation notice in status new.
pub fn new_escalation_notice(code: &str, label: &str) -> EscalationNotice {
    EscalationNotice {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        notes: "".to_string(),
        active: false,
        duration_minutes: 0,
        status: EscalationNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_notice("code-418", "label-165");
        assert_eq!(item.status, EscalationNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_escalation_notice("code-418", "label-165");
        if item.move_to(EscalationNoticeStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationNoticeStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_notice("code-418", "label-165");
        item.status = EscalationNoticeStatus::Failed;
        assert!(
            item.move_to(EscalationNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

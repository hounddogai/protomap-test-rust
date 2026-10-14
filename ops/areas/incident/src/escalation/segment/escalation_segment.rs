use super::*;

use acme_kit::text;

/// Where a escalation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationSegmentStatus {
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

impl std::fmt::Display for EscalationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSegment {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub host: String,
    pub principal: String,
    pub reviewer: String,
    pub priority: i64,
    pub visible: bool,
    pub duration_minutes: i64,
    pub channel: String,
    pub currency: String,
    pub status: EscalationSegmentStatus,
}

impl EscalationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationSegmentStatus) -> bool {
        match self.status {
            EscalationSegmentStatus::New => {
                return next == EscalationSegmentStatus::Done;
            }
            EscalationSegmentStatus::Queued => {
                return next == EscalationSegmentStatus::Running || next == EscalationSegmentStatus::Done;
            }
            EscalationSegmentStatus::Running => {
                return next == EscalationSegmentStatus::Done || next == EscalationSegmentStatus::Failed;
            }
            EscalationSegmentStatus::Done => {
                return next == EscalationSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.revision as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationSegment {
        EscalationSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            host: self.host.clone(),
            principal: self.principal.clone(),
            reviewer: self.reviewer.clone(),
            priority: self.priority,
            visible: self.visible,
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a escalation segment in status new.
pub fn new_escalation_segment(code: &str, title: &str) -> EscalationSegment {
    EscalationSegment {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        host: "".to_string(),
        principal: "".to_string(),
        reviewer: "".to_string(),
        priority: 0,
        visible: false,
        duration_minutes: 0,
        channel: "".to_string(),
        currency: "".to_string(),
        status: EscalationSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_segment("code-632", "title-255");
        assert_eq!(item.status, EscalationSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_escalation_segment("code-632", "title-255");
        if item.move_to(EscalationSegmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationSegmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_segment("code-632", "title-255");
        item.status = EscalationSegmentStatus::Failed;
        assert!(
            item.move_to(EscalationSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

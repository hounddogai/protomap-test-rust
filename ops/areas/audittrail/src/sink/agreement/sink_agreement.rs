use super::*;

use acme_kit::text;

/// Where a sink agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkAgreementStatus {
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

impl std::fmt::Display for SinkAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreement {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub owner_id: String,
    pub rate: f64,
    pub threshold: f64,
    pub notes: String,
    pub status: SinkAgreementStatus,
}

impl SinkAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkAgreementStatus) -> bool {
        match self.status {
            SinkAgreementStatus::New => {
                return next == SinkAgreementStatus::Done;
            }
            SinkAgreementStatus::Queued => {
                match next {
                    SinkAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAgreementStatus::Running => {
                match next {
                    SinkAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkAgreementStatus::Done => {
                match next {
                    SinkAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: SinkAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkAgreement {
        SinkAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            owner_id: self.owner_id.clone(),
            rate: self.rate,
            threshold: self.threshold,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a sink agreement in status new.
pub fn new_sink_agreement(code: &str, name: &str) -> SinkAgreement {
    SinkAgreement {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        owner_id: "".to_string(),
        rate: 0.0,
        threshold: 0.0,
        notes: "".to_string(),
        status: SinkAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_sink_agreement("code-312", "name-35");
        assert_eq!(item.status, SinkAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_sink_agreement("code-312", "name-35");
        if item.move_to(SinkAgreementStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkAgreementStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_sink_agreement("code-312", "name-35");
        item.status = SinkAgreementStatus::Failed;
        assert!(item.move_to(SinkAgreementStatus::New).is_err(), "expected move to fail");
    }
}

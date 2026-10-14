use super::*;

use acme_kit::text;

/// Where a scheduled retention proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRetentionProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledRetentionProposalStatus {
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

impl std::fmt::Display for ScheduledRetentionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled retention proposal as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionProposal {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub threshold: f64,
    pub description: String,
    pub issuer: String,
    pub sequence: i64,
    pub source: String,
    pub port: i64,
    pub status: ScheduledRetentionProposalStatus,
}

impl ScheduledRetentionProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRetentionProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRetentionProposalStatus) -> bool {
        match self.status {
            ScheduledRetentionProposalStatus::New => {
                return next == ScheduledRetentionProposalStatus::Running
                    || next == ScheduledRetentionProposalStatus::Done;
            }
            ScheduledRetentionProposalStatus::Queued => {
                match next {
                    ScheduledRetentionProposalStatus::Running | ScheduledRetentionProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRetentionProposalStatus::Running => {
                match next {
                    ScheduledRetentionProposalStatus::Done | ScheduledRetentionProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRetentionProposalStatus::Done => {
                match next {
                    ScheduledRetentionProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledRetentionProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled retention proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRetentionProposal {
        ScheduledRetentionProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            threshold: self.threshold,
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled retention proposal in status new.
pub fn new_scheduled_retention_proposal(code: &str, title: &str) -> ScheduledRetentionProposal {
    ScheduledRetentionProposal {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        threshold: 0.0,
        description: "".to_string(),
        issuer: "".to_string(),
        sequence: 0,
        source: "".to_string(),
        port: 0,
        status: ScheduledRetentionProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_retention_proposal("code-18", "title-956");
        assert_eq!(item.status, ScheduledRetentionProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_scheduled_retention_proposal("code-18", "title-956");
        if item.move_to(ScheduledRetentionProposalStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRetentionProposalStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_retention_proposal("code-18", "title-956");
        item.status = ScheduledRetentionProposalStatus::Failed;
        assert!(
            item.move_to(ScheduledRetentionProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

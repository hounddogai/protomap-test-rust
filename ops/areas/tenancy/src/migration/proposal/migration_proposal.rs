use super::*;

use acme_kit::text;

/// Where a migration proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MigrationProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for MigrationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProposal {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub expires_at: i64,
    pub source: String,
    pub status: MigrationProposalStatus,
}

impl MigrationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationProposalStatus) -> bool {
        match self.status {
            MigrationProposalStatus::Planned => {
                match next {
                    MigrationProposalStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationProposalStatus::Scheduled => {
                match next {
                    MigrationProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationProposalStatus::InProgress => {
                return next == MigrationProposalStatus::Completed;
            }
            MigrationProposalStatus::Completed => {
                return next == MigrationProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationProposal {
        MigrationProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a migration proposal in status planned.
pub fn new_migration_proposal(code: &str, title: &str) -> MigrationProposal {
    MigrationProposal {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        expires_at: 0,
        source: "".to_string(),
        status: MigrationProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_migration_proposal("code-433", "title-969");
        assert_eq!(item.status, MigrationProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_migration_proposal("code-433", "title-969");
        if item.move_to(MigrationProposalStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationProposalStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_migration_proposal("code-433", "title-969");
        item.status = MigrationProposalStatus::Cancelled;
        assert!(
            item.move_to(MigrationProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

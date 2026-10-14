use super::*;

use acme_kit::text;

/// Where a global retention proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRetentionProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalRetentionProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalRetentionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global retention proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionProposal {
    pub code: String,
    pub title: String,
    pub source: String,
    pub port: i64,
    pub grace_days: i64,
    pub reviewer: String,
    pub cipher: String,
    pub status: GlobalRetentionProposalStatus,
}

impl GlobalRetentionProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRetentionProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRetentionProposalStatus) -> bool {
        match self.status {
            GlobalRetentionProposalStatus::Draft => {
                return next == GlobalRetentionProposalStatus::Closed;
            }
            GlobalRetentionProposalStatus::Pending => {
                match next {
                    GlobalRetentionProposalStatus::Suspended | GlobalRetentionProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionProposalStatus::Active => {
                return next == GlobalRetentionProposalStatus::Suspended
                    || next == GlobalRetentionProposalStatus::Closed;
            }
            GlobalRetentionProposalStatus::Suspended => {
                match next {
                    GlobalRetentionProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalRetentionProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global retention proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global retention proposal", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRetentionProposal {
        GlobalRetentionProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            port: self.port,
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global retention proposal in status draft.
pub fn new_global_retention_proposal(code: &str, title: &str) -> GlobalRetentionProposal {
    GlobalRetentionProposal {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        port: 0,
        grace_days: 0,
        reviewer: "".to_string(),
        cipher: "".to_string(),
        status: GlobalRetentionProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_retention_proposal("code-559", "title-490");
        assert_eq!(item.status, GlobalRetentionProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_global_retention_proposal("code-559", "title-490");
        if item.move_to(GlobalRetentionProposalStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRetentionProposalStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_retention_proposal("code-559", "title-490");
        item.status = GlobalRetentionProposalStatus::Closed;
        assert!(
            item.move_to(GlobalRetentionProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

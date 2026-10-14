use super::*;

use acme_kit::text;

/// Where a strict page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPageProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictPageProposalStatus {
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

impl std::fmt::Display for StrictPageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict page proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageProposal {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub category: String,
    pub checksum: String,
    pub floor_value: f64,
    pub expires_at: i64,
    pub created_at: i64,
    pub archived: bool,
    pub cipher: String,
    pub status: StrictPageProposalStatus,
}

impl StrictPageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPageProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPageProposalStatus) -> bool {
        match self.status {
            StrictPageProposalStatus::New => {
                match next {
                    StrictPageProposalStatus::Done | StrictPageProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageProposalStatus::Queued => {
                match next {
                    StrictPageProposalStatus::Running | StrictPageProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageProposalStatus::Running => {
                return next == StrictPageProposalStatus::Done || next == StrictPageProposalStatus::Failed;
            }
            StrictPageProposalStatus::Done => {
                return next == StrictPageProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict page proposal {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.floor_value * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPageProposal {
        StrictPageProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            category: self.category.clone(),
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            created_at: self.created_at,
            archived: self.archived,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict page proposal in status new.
pub fn new_strict_page_proposal(code: &str, title: &str) -> StrictPageProposal {
    StrictPageProposal {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        category: "".to_string(),
        checksum: "".to_string(),
        floor_value: 0.0,
        expires_at: 0,
        created_at: 0,
        archived: false,
        cipher: "".to_string(),
        status: StrictPageProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_page_proposal("code-99", "title-506");
        assert_eq!(item.status, StrictPageProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_strict_page_proposal("code-99", "title-506");
        if item.move_to(StrictPageProposalStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPageProposalStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_page_proposal("code-99", "title-506");
        item.status = StrictPageProposalStatus::Failed;
        assert!(
            item.move_to(StrictPageProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a postmortem proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemProposalStatus {
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

impl std::fmt::Display for PostmortemProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProposal {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub cost: f64,
    pub pinned: bool,
    pub country: String,
    pub score: f64,
    pub burst: i64,
    pub tags: Vec<String>,
    pub margin: f64,
    pub status: PostmortemProposalStatus,
}

impl PostmortemProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemProposalStatus) -> bool {
        match self.status {
            PostmortemProposalStatus::New => {
                return next == PostmortemProposalStatus::Failed;
            }
            PostmortemProposalStatus::Queued => {
                match next {
                    PostmortemProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemProposalStatus::Running => {
                return next == PostmortemProposalStatus::Done || next == PostmortemProposalStatus::Failed;
            }
            PostmortemProposalStatus::Done => {
                return next == PostmortemProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem proposal", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemProposal {
        PostmortemProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            pinned: self.pinned,
            country: self.country.clone(),
            score: self.score,
            burst: self.burst,
            tags: self.tags.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a postmortem proposal in status new.
pub fn new_postmortem_proposal(code: &str, label: &str) -> PostmortemProposal {
    PostmortemProposal {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        contact_email: "".to_string(),
        cost: 0.0,
        pinned: false,
        country: "".to_string(),
        score: 0.0,
        burst: 0,
        tags: Vec::new(),
        margin: 0.0,
        status: PostmortemProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_postmortem_proposal("code-943", "label-817");
        assert_eq!(item.status, PostmortemProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_postmortem_proposal("code-943", "label-817");
        if item.move_to(PostmortemProposalStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemProposalStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_postmortem_proposal("code-943", "label-817");
        item.status = PostmortemProposalStatus::Failed;
        assert!(
            item.move_to(PostmortemProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

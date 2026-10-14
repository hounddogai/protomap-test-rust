use super::*;

use acme_kit::text;

/// Where a default retention proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRetentionProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultRetentionProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultRetentionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default retention proposal as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposal {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub grace_days: i64,
    pub score: f64,
    pub host: String,
    pub floor_value: f64,
    pub limit: i64,
    pub created_at: i64,
    pub locale: String,
    pub status: DefaultRetentionProposalStatus,
}

impl DefaultRetentionProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRetentionProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRetentionProposalStatus) -> bool {
        match self.status {
            DefaultRetentionProposalStatus::Proposed => {
                match next {
                    DefaultRetentionProposalStatus::Rejected | DefaultRetentionProposalStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRetentionProposalStatus::Approved => {
                return next == DefaultRetentionProposalStatus::Rejected
                    || next == DefaultRetentionProposalStatus::Withdrawn;
            }
            DefaultRetentionProposalStatus::Rejected => {
                match next {
                    DefaultRetentionProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultRetentionProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default retention proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.limit as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRetentionProposal {
        DefaultRetentionProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            score: self.score,
            host: self.host.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            created_at: self.created_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default retention proposal in status proposed.
pub fn new_default_retention_proposal(code: &str, title: &str) -> DefaultRetentionProposal {
    DefaultRetentionProposal {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        grace_days: 0,
        score: 0.0,
        host: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        created_at: 0,
        locale: "".to_string(),
        status: DefaultRetentionProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_retention_proposal("code-160", "title-208");
        assert_eq!(item.status, DefaultRetentionProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_retention_proposal("code-160", "title-208");
        if item.move_to(DefaultRetentionProposalStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRetentionProposalStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_retention_proposal("code-160", "title-208");
        item.status = DefaultRetentionProposalStatus::Withdrawn;
        assert!(
            item.move_to(DefaultRetentionProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

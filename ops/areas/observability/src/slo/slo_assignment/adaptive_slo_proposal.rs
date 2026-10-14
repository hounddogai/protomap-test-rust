use super::*;

use acme_kit::text;

/// Where a adaptive slo proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSloProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveSloProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveSloProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive slo proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloProposal {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub tags: Vec<String>,
    pub reviewer: String,
    pub ratio: f64,
    pub category: String,
    pub label: String,
    pub port: i64,
    pub owner_id: String,
    pub ends_at: i64,
    pub status: AdaptiveSloProposalStatus,
}

impl AdaptiveSloProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSloProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSloProposalStatus) -> bool {
        match self.status {
            AdaptiveSloProposalStatus::Open => {
                match next {
                    AdaptiveSloProposalStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloProposalStatus::InReview => {
                return next == AdaptiveSloProposalStatus::Archived;
            }
            AdaptiveSloProposalStatus::Resolved => {
                match next {
                    AdaptiveSloProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloProposalStatus::Reopened => {
                match next {
                    AdaptiveSloProposalStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveSloProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive slo proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ends_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSloProposal {
        AdaptiveSloProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            category: self.category.clone(),
            label: self.label.clone(),
            port: self.port,
            owner_id: self.owner_id.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive slo proposal in status open.
pub fn new_adaptive_slo_proposal(code: &str, title: &str) -> AdaptiveSloProposal {
    AdaptiveSloProposal {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        tags: Vec::new(),
        reviewer: "".to_string(),
        ratio: 0.0,
        category: "".to_string(),
        label: "".to_string(),
        port: 0,
        owner_id: "".to_string(),
        ends_at: 0,
        status: AdaptiveSloProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_slo_proposal("code-191", "title-819");
        assert_eq!(item.status, AdaptiveSloProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_slo_proposal("code-191", "title-819");
        if item.move_to(AdaptiveSloProposalStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSloProposalStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_slo_proposal("code-191", "title-819");
        item.status = AdaptiveSloProposalStatus::Archived;
        assert!(
            item.move_to(AdaptiveSloProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

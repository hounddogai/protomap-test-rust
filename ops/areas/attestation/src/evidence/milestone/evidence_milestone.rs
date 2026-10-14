use super::*;

use acme_kit::text;

/// Where a evidence milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EvidenceMilestoneStatus {
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

impl std::fmt::Display for EvidenceMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestone {
    pub code: String,
    pub title: String,
    pub country: String,
    pub host: String,
    pub score: f64,
    pub threshold: f64,
    pub summary: String,
    pub contact_email: String,
    pub currency: String,
    pub port: i64,
    pub status: EvidenceMilestoneStatus,
}

impl EvidenceMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceMilestoneStatus) -> bool {
        match self.status {
            EvidenceMilestoneStatus::Open => {
                match next {
                    EvidenceMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceMilestoneStatus::InReview => {
                return next == EvidenceMilestoneStatus::Resolved || next == EvidenceMilestoneStatus::Archived;
            }
            EvidenceMilestoneStatus::Resolved => {
                return next == EvidenceMilestoneStatus::Reopened || next == EvidenceMilestoneStatus::Archived;
            }
            EvidenceMilestoneStatus::Reopened => {
                return next == EvidenceMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceMilestone {
        EvidenceMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            host: self.host.clone(),
            score: self.score,
            threshold: self.threshold,
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a evidence milestone in status open.
pub fn new_evidence_milestone(code: &str, title: &str) -> EvidenceMilestone {
    EvidenceMilestone {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        host: "".to_string(),
        score: 0.0,
        threshold: 0.0,
        summary: "".to_string(),
        contact_email: "".to_string(),
        currency: "".to_string(),
        port: 0,
        status: EvidenceMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_evidence_milestone("code-883", "title-731");
        assert_eq!(item.status, EvidenceMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_evidence_milestone("code-883", "title-731");
        if item.move_to(EvidenceMilestoneStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceMilestoneStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_evidence_milestone("code-883", "title-731");
        item.status = EvidenceMilestoneStatus::Archived;
        assert!(
            item.move_to(EvidenceMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

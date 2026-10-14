use super::*;

use acme_kit::text;

/// Where a egress digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EgressDigestStatus {
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

impl std::fmt::Display for EgressDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress digest as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigest {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub principal: String,
    pub segment: String,
    pub score: f64,
    pub status: EgressDigestStatus,
}

impl EgressDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressDigestStatus) -> bool {
        match self.status {
            EgressDigestStatus::Open => {
                return next == EgressDigestStatus::InReview || next == EgressDigestStatus::Resolved;
            }
            EgressDigestStatus::InReview => {
                return next == EgressDigestStatus::Resolved || next == EgressDigestStatus::Archived;
            }
            EgressDigestStatus::Resolved => {
                return next == EgressDigestStatus::Archived;
            }
            EgressDigestStatus::Reopened => {
                return next == EgressDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> EgressDigest {
        EgressDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            segment: self.segment.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a egress digest in status open.
pub fn new_egress_digest(code: &str, title: &str) -> EgressDigest {
    EgressDigest {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        principal: "".to_string(),
        segment: "".to_string(),
        score: 0.0,
        status: EgressDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_egress_digest("code-371", "title-544");
        assert_eq!(item.status, EgressDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_egress_digest("code-371", "title-544");
        if item.move_to(EgressDigestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressDigestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_egress_digest("code-371", "title-544");
        item.status = EgressDigestStatus::Archived;
        assert!(item.move_to(EgressDigestStatus::Open).is_err(), "expected move to fail");
    }
}

use super::*;

use acme_kit::text;

/// Where a seasonal evidence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEvidenceBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalEvidenceBundleStatus {
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

impl std::fmt::Display for SeasonalEvidenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal evidence bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceBundle {
    pub code: String,
    pub name: String,
    pub country: String,
    pub notes: String,
    pub capacity: i64,
    pub issuer: String,
    pub approved: bool,
    pub ratio: f64,
    pub principal: String,
    pub batch_size: i64,
    pub status: SeasonalEvidenceBundleStatus,
}

impl SeasonalEvidenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEvidenceBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEvidenceBundleStatus) -> bool {
        match self.status {
            SeasonalEvidenceBundleStatus::Open => {
                match next {
                    SeasonalEvidenceBundleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEvidenceBundleStatus::InReview => {
                return next == SeasonalEvidenceBundleStatus::Resolved
                    || next == SeasonalEvidenceBundleStatus::Reopened;
            }
            SeasonalEvidenceBundleStatus::Resolved => {
                return next == SeasonalEvidenceBundleStatus::Archived;
            }
            SeasonalEvidenceBundleStatus::Reopened => {
                match next {
                    SeasonalEvidenceBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalEvidenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal evidence bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal evidence bundle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEvidenceBundle {
        SeasonalEvidenceBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            approved: self.approved,
            ratio: self.ratio,
            principal: self.principal.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal evidence bundle in status open.
pub fn new_seasonal_evidence_bundle(code: &str, name: &str) -> SeasonalEvidenceBundle {
    SeasonalEvidenceBundle {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        capacity: 0,
        issuer: "".to_string(),
        approved: false,
        ratio: 0.0,
        principal: "".to_string(),
        batch_size: 0,
        status: SeasonalEvidenceBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_evidence_bundle("code-926", "name-99");
        assert_eq!(item.status, SeasonalEvidenceBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_evidence_bundle("code-926", "name-99");
        if item.move_to(SeasonalEvidenceBundleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEvidenceBundleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_evidence_bundle("code-926", "name-99");
        item.status = SeasonalEvidenceBundleStatus::Archived;
        assert!(
            item.move_to(SeasonalEvidenceBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a fallback evidence agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEvidenceAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackEvidenceAgreementStatus {
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

impl std::fmt::Display for FallbackEvidenceAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback evidence agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreement {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub timezone: String,
    pub active: bool,
    pub budget: f64,
    pub maximum: f64,
    pub quota_units: i64,
    pub starts_at: i64,
    pub status: FallbackEvidenceAgreementStatus,
}

impl FallbackEvidenceAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEvidenceAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEvidenceAgreementStatus) -> bool {
        match self.status {
            FallbackEvidenceAgreementStatus::Open => {
                match next {
                    FallbackEvidenceAgreementStatus::Reopened | FallbackEvidenceAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEvidenceAgreementStatus::InReview => {
                match next {
                    FallbackEvidenceAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEvidenceAgreementStatus::Resolved => {
                return next == FallbackEvidenceAgreementStatus::Reopened;
            }
            FallbackEvidenceAgreementStatus::Reopened => {
                return next == FallbackEvidenceAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackEvidenceAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback evidence agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.budget * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackEvidenceAgreement {
        FallbackEvidenceAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            active: self.active,
            budget: self.budget,
            maximum: self.maximum,
            quota_units: self.quota_units,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback evidence agreement in status open.
pub fn new_fallback_evidence_agreement(code: &str, title: &str) -> FallbackEvidenceAgreement {
    FallbackEvidenceAgreement {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        timezone: "".to_string(),
        active: false,
        budget: 0.0,
        maximum: 0.0,
        quota_units: 0,
        starts_at: 0,
        status: FallbackEvidenceAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_evidence_agreement("code-55", "title-797");
        assert_eq!(item.status, FallbackEvidenceAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_fallback_evidence_agreement("code-55", "title-797");
        if item.move_to(FallbackEvidenceAgreementStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEvidenceAgreementStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_evidence_agreement("code-55", "title-797");
        item.status = FallbackEvidenceAgreementStatus::Archived;
        assert!(
            item.move_to(FallbackEvidenceAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

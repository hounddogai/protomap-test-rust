use super::*;

use acme_kit::text;

/// Where a adaptive release agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveReleaseAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveReleaseAgreementStatus {
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

impl std::fmt::Display for AdaptiveReleaseAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive release agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseAgreement {
    pub code: String,
    pub label: String,
    pub category: String,
    pub audience: String,
    pub limit: i64,
    pub contact_email: String,
    pub pinned: bool,
    pub amount: f64,
    pub burst: i64,
    pub status: AdaptiveReleaseAgreementStatus,
}

impl AdaptiveReleaseAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveReleaseAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveReleaseAgreementStatus) -> bool {
        match self.status {
            AdaptiveReleaseAgreementStatus::New => {
                return next == AdaptiveReleaseAgreementStatus::Queued;
            }
            AdaptiveReleaseAgreementStatus::Queued => {
                return next == AdaptiveReleaseAgreementStatus::Done || next == AdaptiveReleaseAgreementStatus::Failed;
            }
            AdaptiveReleaseAgreementStatus::Running => {
                match next {
                    AdaptiveReleaseAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveReleaseAgreementStatus::Done => {
                match next {
                    AdaptiveReleaseAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveReleaseAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive release agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.burst as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveReleaseAgreement {
        AdaptiveReleaseAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            audience: self.audience.clone(),
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            pinned: self.pinned,
            amount: self.amount,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive release agreement in status new.
pub fn new_adaptive_release_agreement(code: &str, label: &str) -> AdaptiveReleaseAgreement {
    AdaptiveReleaseAgreement {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        audience: "".to_string(),
        limit: 0,
        contact_email: "".to_string(),
        pinned: false,
        amount: 0.0,
        burst: 0,
        status: AdaptiveReleaseAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_release_agreement("code-179", "label-42");
        assert_eq!(item.status, AdaptiveReleaseAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_adaptive_release_agreement("code-179", "label-42");
        if item.move_to(AdaptiveReleaseAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveReleaseAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_release_agreement("code-179", "label-42");
        item.status = AdaptiveReleaseAgreementStatus::Failed;
        assert!(
            item.move_to(AdaptiveReleaseAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

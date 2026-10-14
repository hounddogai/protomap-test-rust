use super::*;

use acme_kit::text;

/// Where a lenient federation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFederationAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientFederationAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientFederationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient federation agreement as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationAgreement {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub archived: bool,
    pub source: String,
    pub status: LenientFederationAgreementStatus,
}

impl LenientFederationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFederationAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFederationAgreementStatus) -> bool {
        match self.status {
            LenientFederationAgreementStatus::Planned => {
                return next == LenientFederationAgreementStatus::Cancelled;
            }
            LenientFederationAgreementStatus::Scheduled => {
                return next == LenientFederationAgreementStatus::InProgress
                    || next == LenientFederationAgreementStatus::Completed;
            }
            LenientFederationAgreementStatus::InProgress => {
                return next == LenientFederationAgreementStatus::Cancelled;
            }
            LenientFederationAgreementStatus::Completed => {
                return next == LenientFederationAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFederationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient federation agreement {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> LenientFederationAgreement {
        LenientFederationAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            archived: self.archived,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient federation agreement in status planned.
pub fn new_lenient_federation_agreement(code: &str, title: &str) -> LenientFederationAgreement {
    LenientFederationAgreement {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        archived: false,
        source: "".to_string(),
        status: LenientFederationAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_federation_agreement("code-993", "title-118");
        assert_eq!(item.status, LenientFederationAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_lenient_federation_agreement("code-993", "title-118");
        if item.move_to(LenientFederationAgreementStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFederationAgreementStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_federation_agreement("code-993", "title-118");
        item.status = LenientFederationAgreementStatus::Cancelled;
        assert!(
            item.move_to(LenientFederationAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

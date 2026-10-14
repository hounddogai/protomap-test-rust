use super::*;

use acme_kit::text;

/// Where a release agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ReleaseAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release agreement as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAgreement {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub scope: String,
    pub limit: i64,
    pub notes: String,
    pub retries: i64,
    pub quota_units: i64,
    pub status: ReleaseAgreementStatus,
}

impl ReleaseAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseAgreementStatus) -> bool {
        match self.status {
            ReleaseAgreementStatus::Draft => {
                return next == ReleaseAgreementStatus::Active || next == ReleaseAgreementStatus::Suspended;
            }
            ReleaseAgreementStatus::Pending => {
                return next == ReleaseAgreementStatus::Active;
            }
            ReleaseAgreementStatus::Active => {
                return next == ReleaseAgreementStatus::Closed;
            }
            ReleaseAgreementStatus::Suspended => {
                return next == ReleaseAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseAgreement {
        ReleaseAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            retries: self.retries,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a release agreement in status draft.
pub fn new_release_agreement(code: &str, title: &str) -> ReleaseAgreement {
    ReleaseAgreement {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        scope: "".to_string(),
        limit: 0,
        notes: "".to_string(),
        retries: 0,
        quota_units: 0,
        status: ReleaseAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_agreement("code-669", "title-824");
        assert_eq!(item.status, ReleaseAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_release_agreement("code-669", "title-824");
        if item.move_to(ReleaseAgreementStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseAgreementStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_agreement("code-669", "title-824");
        item.status = ReleaseAgreementStatus::Closed;
        assert!(
            item.move_to(ReleaseAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

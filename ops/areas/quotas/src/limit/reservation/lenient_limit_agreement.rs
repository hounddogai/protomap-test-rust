use super::*;

use acme_kit::text;

/// Where a lenient limit agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientLimitAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientLimitAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientLimitAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient limit agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitAgreement {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub updated_at: i64,
    pub version: i64,
    pub rotation_days: i64,
    pub status: LenientLimitAgreementStatus,
}

impl LenientLimitAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientLimitAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientLimitAgreementStatus) -> bool {
        match self.status {
            LenientLimitAgreementStatus::Proposed => {
                return next == LenientLimitAgreementStatus::Approved || next == LenientLimitAgreementStatus::Rejected;
            }
            LenientLimitAgreementStatus::Approved => {
                return next == LenientLimitAgreementStatus::Rejected || next == LenientLimitAgreementStatus::Withdrawn;
            }
            LenientLimitAgreementStatus::Rejected => {
                return next == LenientLimitAgreementStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientLimitAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient limit agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientLimitAgreement {
        LenientLimitAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            updated_at: self.updated_at,
            version: self.version,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient limit agreement in status proposed.
pub fn new_lenient_limit_agreement(code: &str, title: &str) -> LenientLimitAgreement {
    LenientLimitAgreement {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        external_ref: "".to_string(),
        ttl_seconds: 0,
        updated_at: 0,
        version: 0,
        rotation_days: 0,
        status: LenientLimitAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_limit_agreement("code-798", "title-658");
        assert_eq!(item.status, LenientLimitAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_limit_agreement("code-798", "title-658");
        if item.move_to(LenientLimitAgreementStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientLimitAgreementStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_limit_agreement("code-798", "title-658");
        item.status = LenientLimitAgreementStatus::Withdrawn;
        assert!(
            item.move_to(LenientLimitAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

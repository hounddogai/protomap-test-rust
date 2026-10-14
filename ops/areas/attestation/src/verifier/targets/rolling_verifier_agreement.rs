use super::*;

use acme_kit::text;

/// Where a rolling verifier agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingVerifierAgreementStatus {
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

impl std::fmt::Display for RollingVerifierAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling verifier agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreement {
    pub code: String,
    pub title: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub locale: String,
    pub checksum: String,
    pub amount: f64,
    pub pinned: bool,
    pub approved: bool,
    pub rate: f64,
    pub status: RollingVerifierAgreementStatus,
}

impl RollingVerifierAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierAgreementStatus) -> bool {
        match self.status {
            RollingVerifierAgreementStatus::Draft => {
                match next {
                    RollingVerifierAgreementStatus::Pending | RollingVerifierAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierAgreementStatus::Pending => {
                return next == RollingVerifierAgreementStatus::Active;
            }
            RollingVerifierAgreementStatus::Active => {
                return next == RollingVerifierAgreementStatus::Suspended
                    || next == RollingVerifierAgreementStatus::Closed;
            }
            RollingVerifierAgreementStatus::Suspended => {
                return next == RollingVerifierAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingVerifierAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rate - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierAgreement {
        RollingVerifierAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            pinned: self.pinned,
            approved: self.approved,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling verifier agreement in status draft.
pub fn new_rolling_verifier_agreement(code: &str, title: &str) -> RollingVerifierAgreement {
    RollingVerifierAgreement {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        ttl_seconds: 0,
        revision: 0,
        locale: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        pinned: false,
        approved: false,
        rate: 0.0,
        status: RollingVerifierAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_verifier_agreement("code-152", "title-966");
        assert_eq!(item.status, RollingVerifierAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_verifier_agreement("code-152", "title-966");
        if item.move_to(RollingVerifierAgreementStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingVerifierAgreementStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_verifier_agreement("code-152", "title-966");
        item.status = RollingVerifierAgreementStatus::Closed;
        assert!(
            item.move_to(RollingVerifierAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

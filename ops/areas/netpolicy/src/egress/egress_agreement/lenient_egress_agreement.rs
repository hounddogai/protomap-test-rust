use super::*;

use acme_kit::text;

/// Where a lenient egress agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientEgressAgreementStatus {
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

impl std::fmt::Display for LenientEgressAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient egress agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressAgreement {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub timezone: String,
    pub region: String,
    pub external_ref: String,
    pub checksum: String,
    pub amount: f64,
    pub status: LenientEgressAgreementStatus,
}

impl LenientEgressAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressAgreementStatus) -> bool {
        match self.status {
            LenientEgressAgreementStatus::Draft => {
                return next == LenientEgressAgreementStatus::Pending;
            }
            LenientEgressAgreementStatus::Pending => {
                return next == LenientEgressAgreementStatus::Active || next == LenientEgressAgreementStatus::Closed;
            }
            LenientEgressAgreementStatus::Active => {
                return next == LenientEgressAgreementStatus::Suspended || next == LenientEgressAgreementStatus::Closed;
            }
            LenientEgressAgreementStatus::Suspended => {
                return next == LenientEgressAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEgressAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> LenientEgressAgreement {
        LenientEgressAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient egress agreement in status draft.
pub fn new_lenient_egress_agreement(code: &str, title: &str) -> LenientEgressAgreement {
    LenientEgressAgreement {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        timezone: "".to_string(),
        region: "".to_string(),
        external_ref: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        status: LenientEgressAgreementStatus::Draft,
    }
}

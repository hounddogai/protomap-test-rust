use super::*;

use acme_kit::text;

/// Where a pending region pin agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRegionPinAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingRegionPinAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingRegionPinAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending region pin agreement as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreement {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub category: String,
    pub country: String,
    pub amount: f64,
    pub batch_size: i64,
    pub host: String,
    pub rate: f64,
    pub status: PendingRegionPinAgreementStatus,
}

impl PendingRegionPinAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRegionPinAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRegionPinAgreementStatus) -> bool {
        match self.status {
            PendingRegionPinAgreementStatus::Requested => {
                match next {
                    PendingRegionPinAgreementStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRegionPinAgreementStatus::Granted => {
                return next == PendingRegionPinAgreementStatus::Revoked;
            }
            PendingRegionPinAgreementStatus::Revoked => {
                return next == PendingRegionPinAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRegionPinAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending region pin agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.amount - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRegionPinAgreement {
        PendingRegionPinAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            amount: self.amount,
            batch_size: self.batch_size,
            host: self.host.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending region pin agreement in status requested.
pub fn new_pending_region_pin_agreement(code: &str, title: &str) -> PendingRegionPinAgreement {
    PendingRegionPinAgreement {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        category: "".to_string(),
        country: "".to_string(),
        amount: 0.0,
        batch_size: 0,
        host: "".to_string(),
        rate: 0.0,
        status: PendingRegionPinAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_region_pin_agreement("code-507", "title-228");
        assert_eq!(item.status, PendingRegionPinAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_region_pin_agreement("code-507", "title-228");
        if item.move_to(PendingRegionPinAgreementStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRegionPinAgreementStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_region_pin_agreement("code-507", "title-228");
        item.status = PendingRegionPinAgreementStatus::Expired;
        assert!(
            item.move_to(PendingRegionPinAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

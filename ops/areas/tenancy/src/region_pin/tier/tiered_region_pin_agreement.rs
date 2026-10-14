use super::*;

use acme_kit::text;

/// Where a tiered region pin agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRegionPinAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredRegionPinAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredRegionPinAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered region pin agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreement {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub cluster: String,
    pub attempts: i64,
    pub checksum: String,
    pub status: TieredRegionPinAgreementStatus,
}

impl TieredRegionPinAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRegionPinAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRegionPinAgreementStatus) -> bool {
        match self.status {
            TieredRegionPinAgreementStatus::Requested => {
                return next == TieredRegionPinAgreementStatus::Revoked;
            }
            TieredRegionPinAgreementStatus::Granted => {
                return next == TieredRegionPinAgreementStatus::Revoked;
            }
            TieredRegionPinAgreementStatus::Revoked => {
                match next {
                    TieredRegionPinAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: TieredRegionPinAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered region pin agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> TieredRegionPinAgreement {
        TieredRegionPinAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered region pin agreement in status requested.
pub fn new_tiered_region_pin_agreement(code: &str, title: &str) -> TieredRegionPinAgreement {
    TieredRegionPinAgreement {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        cluster: "".to_string(),
        attempts: 0,
        checksum: "".to_string(),
        status: TieredRegionPinAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_region_pin_agreement("code-69", "title-351");
        assert_eq!(item.status, TieredRegionPinAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tiered_region_pin_agreement("code-69", "title-351");
        if item.move_to(TieredRegionPinAgreementStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRegionPinAgreementStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_region_pin_agreement("code-69", "title-351");
        item.status = TieredRegionPinAgreementStatus::Expired;
        assert!(
            item.move_to(TieredRegionPinAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a seasonal delegation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDelegationProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalDelegationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalDelegationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal delegation profile as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDelegationProfile {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub currency: String,
    pub score: f64,
    pub margin: f64,
    pub checksum: String,
    pub limit: i64,
    pub minimum: f64,
    pub segment: String,
    pub status: SeasonalDelegationProfileStatus,
}

impl SeasonalDelegationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDelegationProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDelegationProfileStatus) -> bool {
        match self.status {
            SeasonalDelegationProfileStatus::Requested => {
                return next == SeasonalDelegationProfileStatus::Expired;
            }
            SeasonalDelegationProfileStatus::Granted => {
                return next == SeasonalDelegationProfileStatus::Revoked
                    || next == SeasonalDelegationProfileStatus::Expired;
            }
            SeasonalDelegationProfileStatus::Revoked => {
                match next {
                    SeasonalDelegationProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalDelegationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal delegation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.minimum - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDelegationProfile {
        SeasonalDelegationProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            score: self.score,
            margin: self.margin,
            checksum: self.checksum.clone(),
            limit: self.limit,
            minimum: self.minimum,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal delegation profile in status requested.
pub fn new_seasonal_delegation_profile(code: &str, label: &str) -> SeasonalDelegationProfile {
    SeasonalDelegationProfile {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        score: 0.0,
        margin: 0.0,
        checksum: "".to_string(),
        limit: 0,
        minimum: 0.0,
        segment: "".to_string(),
        status: SeasonalDelegationProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_delegation_profile("code-999", "label-3");
        assert_eq!(item.status, SeasonalDelegationProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_delegation_profile("code-999", "label-3");
        if item.move_to(SeasonalDelegationProfileStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDelegationProfileStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_delegation_profile("code-999", "label-3");
        item.status = SeasonalDelegationProfileStatus::Expired;
        assert!(
            item.move_to(SeasonalDelegationProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

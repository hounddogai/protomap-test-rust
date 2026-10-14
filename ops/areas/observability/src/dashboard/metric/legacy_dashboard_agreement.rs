use super::*;

use acme_kit::text;

/// Where a legacy dashboard agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDashboardAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyDashboardAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyDashboardAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy dashboard agreement as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardAgreement {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub budget: f64,
    pub channel: String,
    pub audience: String,
    pub description: String,
    pub ratio: f64,
    pub timezone: String,
    pub status: LegacyDashboardAgreementStatus,
}

impl LegacyDashboardAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDashboardAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDashboardAgreementStatus) -> bool {
        match self.status {
            LegacyDashboardAgreementStatus::Requested => {
                return next == LegacyDashboardAgreementStatus::Revoked
                    || next == LegacyDashboardAgreementStatus::Expired;
            }
            LegacyDashboardAgreementStatus::Granted => {
                match next {
                    LegacyDashboardAgreementStatus::Revoked | LegacyDashboardAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDashboardAgreementStatus::Revoked => {
                return next == LegacyDashboardAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDashboardAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dashboard agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.discount - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDashboardAgreement {
        LegacyDashboardAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            budget: self.budget,
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            description: self.description.clone(),
            ratio: self.ratio,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a legacy dashboard agreement in status requested.
pub fn new_legacy_dashboard_agreement(code: &str, name: &str) -> LegacyDashboardAgreement {
    LegacyDashboardAgreement {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        budget: 0.0,
        channel: "".to_string(),
        audience: "".to_string(),
        description: "".to_string(),
        ratio: 0.0,
        timezone: "".to_string(),
        status: LegacyDashboardAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_dashboard_agreement("code-349", "name-888");
        assert_eq!(item.status, LegacyDashboardAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_dashboard_agreement("code-349", "name-888");
        if item.move_to(LegacyDashboardAgreementStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDashboardAgreementStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_dashboard_agreement("code-349", "name-888");
        item.status = LegacyDashboardAgreementStatus::Expired;
        assert!(
            item.move_to(LegacyDashboardAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

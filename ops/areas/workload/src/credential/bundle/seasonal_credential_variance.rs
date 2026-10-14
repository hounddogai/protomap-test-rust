use super::*;

use acme_kit::text;

/// Where a seasonal credential variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCredentialVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalCredentialVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalCredentialVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal credential variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialVariance {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub reviewer: String,
    pub score: f64,
    pub currency: String,
    pub timezone: String,
    pub scope: String,
    pub pinned: bool,
    pub contact_email: String,
    pub grace_days: i64,
    pub status: SeasonalCredentialVarianceStatus,
}

impl SeasonalCredentialVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCredentialVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCredentialVarianceStatus) -> bool {
        match self.status {
            SeasonalCredentialVarianceStatus::Proposed => {
                match next {
                    SeasonalCredentialVarianceStatus::Approved | SeasonalCredentialVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialVarianceStatus::Approved => {
                return next == SeasonalCredentialVarianceStatus::Rejected
                    || next == SeasonalCredentialVarianceStatus::Withdrawn;
            }
            SeasonalCredentialVarianceStatus::Rejected => {
                match next {
                    SeasonalCredentialVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalCredentialVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal credential variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.score + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCredentialVariance {
        SeasonalCredentialVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            reviewer: self.reviewer.clone(),
            score: self.score,
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            scope: self.scope.clone(),
            pinned: self.pinned,
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal credential variance in status proposed.
pub fn new_seasonal_credential_variance(code: &str, title: &str) -> SeasonalCredentialVariance {
    SeasonalCredentialVariance {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        reviewer: "".to_string(),
        score: 0.0,
        currency: "".to_string(),
        timezone: "".to_string(),
        scope: "".to_string(),
        pinned: false,
        contact_email: "".to_string(),
        grace_days: 0,
        status: SeasonalCredentialVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_credential_variance("code-944", "title-857");
        assert_eq!(item.status, SeasonalCredentialVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_credential_variance("code-944", "title-857");
        if item.move_to(SeasonalCredentialVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCredentialVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_credential_variance("code-944", "title-857");
        item.status = SeasonalCredentialVarianceStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalCredentialVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

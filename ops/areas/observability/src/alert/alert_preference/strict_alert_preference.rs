use super::*;

use acme_kit::text;

/// Where a strict alert preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAlertPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictAlertPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictAlertPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict alert preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreference {
    pub code: String,
    pub name: String,
    pub category: String,
    pub budget: f64,
    pub burst: i64,
    pub external_ref: String,
    pub rate: f64,
    pub margin: f64,
    pub status: StrictAlertPreferenceStatus,
}

impl StrictAlertPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAlertPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAlertPreferenceStatus) -> bool {
        match self.status {
            StrictAlertPreferenceStatus::Requested => {
                return next == StrictAlertPreferenceStatus::Granted;
            }
            StrictAlertPreferenceStatus::Granted => {
                match next {
                    StrictAlertPreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAlertPreferenceStatus::Revoked => {
                return next == StrictAlertPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictAlertPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict alert preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict alert preference", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rate - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAlertPreference {
        StrictAlertPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            budget: self.budget,
            burst: self.burst,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict alert preference in status requested.
pub fn new_strict_alert_preference(code: &str, name: &str) -> StrictAlertPreference {
    StrictAlertPreference {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        budget: 0.0,
        burst: 0,
        external_ref: "".to_string(),
        rate: 0.0,
        margin: 0.0,
        status: StrictAlertPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_alert_preference("code-343", "name-872");
        assert_eq!(item.status, StrictAlertPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_strict_alert_preference("code-343", "name-872");
        if item.move_to(StrictAlertPreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAlertPreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_alert_preference("code-343", "name-872");
        item.status = StrictAlertPreferenceStatus::Expired;
        assert!(
            item.move_to(StrictAlertPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

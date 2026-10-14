use super::*;

use acme_kit::text;

/// Where a federation preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FederationPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FederationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreference {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub phone: String,
    pub grace_days: i64,
    pub status: FederationPreferenceStatus,
}

impl FederationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationPreferenceStatus) -> bool {
        match self.status {
            FederationPreferenceStatus::Requested => {
                return next == FederationPreferenceStatus::Granted || next == FederationPreferenceStatus::Expired;
            }
            FederationPreferenceStatus::Granted => {
                return next == FederationPreferenceStatus::Revoked || next == FederationPreferenceStatus::Expired;
            }
            FederationPreferenceStatus::Revoked => {
                match next {
                    FederationPreferenceStatus::Expired => {
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

    pub fn move_to(&mut self, next: FederationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> FederationPreference {
        FederationPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a federation preference in status requested.
pub fn new_federation_preference(code: &str, title: &str) -> FederationPreference {
    FederationPreference {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        phone: "".to_string(),
        grace_days: 0,
        status: FederationPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_federation_preference("code-208", "title-948");
        assert_eq!(item.status, FederationPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_federation_preference("code-208", "title-948");
        if item.move_to(FederationPreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationPreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_federation_preference("code-208", "title-948");
        item.status = FederationPreferenceStatus::Expired;
        assert!(
            item.move_to(FederationPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

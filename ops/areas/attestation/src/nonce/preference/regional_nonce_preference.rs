use super::*;

use acme_kit::text;

/// Where a regional nonce preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalNoncePreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalNoncePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalNoncePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional nonce preference as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNoncePreference {
    pub code: String,
    pub title: String,
    pub country: String,
    pub timezone: String,
    pub capacity: i64,
    pub status: RegionalNoncePreferenceStatus,
}

impl RegionalNoncePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalNoncePreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalNoncePreferenceStatus) -> bool {
        match self.status {
            RegionalNoncePreferenceStatus::Requested => {
                return next == RegionalNoncePreferenceStatus::Revoked;
            }
            RegionalNoncePreferenceStatus::Granted => {
                match next {
                    RegionalNoncePreferenceStatus::Revoked | RegionalNoncePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalNoncePreferenceStatus::Revoked => {
                return next == RegionalNoncePreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalNoncePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional nonce preference {} cannot move from {} to {}",
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
        self.capacity as f64
    }

    pub fn copy(&self) -> RegionalNoncePreference {
        RegionalNoncePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a regional nonce preference in status requested.
pub fn new_regional_nonce_preference(code: &str, title: &str) -> RegionalNoncePreference {
    RegionalNoncePreference {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        timezone: "".to_string(),
        capacity: 0,
        status: RegionalNoncePreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_nonce_preference("code-93", "title-286");
        assert_eq!(item.status, RegionalNoncePreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_regional_nonce_preference("code-93", "title-286");
        if item.move_to(RegionalNoncePreferenceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalNoncePreferenceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_nonce_preference("code-93", "title-286");
        item.status = RegionalNoncePreferenceStatus::Expired;
        assert!(
            item.move_to(RegionalNoncePreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

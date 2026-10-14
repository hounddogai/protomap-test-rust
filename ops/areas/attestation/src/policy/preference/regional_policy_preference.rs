use super::*;

use acme_kit::text;

/// Where a regional policy preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPolicyPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalPolicyPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalPolicyPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional policy preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreference {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub starts_at: i64,
    pub window_days: i64,
    pub reviewer: String,
    pub threshold: f64,
    pub audience: String,
    pub status: RegionalPolicyPreferenceStatus,
}

impl RegionalPolicyPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPolicyPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPolicyPreferenceStatus) -> bool {
        match self.status {
            RegionalPolicyPreferenceStatus::Requested => {
                match next {
                    RegionalPolicyPreferenceStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPolicyPreferenceStatus::Granted => {
                match next {
                    RegionalPolicyPreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPolicyPreferenceStatus::Revoked => {
                match next {
                    RegionalPolicyPreferenceStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalPolicyPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional policy preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional policy preference", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.window_days as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPolicyPreference {
        RegionalPolicyPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            starts_at: self.starts_at,
            window_days: self.window_days,
            reviewer: self.reviewer.clone(),
            threshold: self.threshold,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional policy preference in status requested.
pub fn new_regional_policy_preference(code: &str, label: &str) -> RegionalPolicyPreference {
    RegionalPolicyPreference {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        sequence: 0,
        starts_at: 0,
        window_days: 0,
        reviewer: "".to_string(),
        threshold: 0.0,
        audience: "".to_string(),
        status: RegionalPolicyPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_policy_preference("code-322", "label-351");
        assert_eq!(item.status, RegionalPolicyPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_policy_preference("code-322", "label-351");
        if item.move_to(RegionalPolicyPreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPolicyPreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_policy_preference("code-322", "label-351");
        item.status = RegionalPolicyPreferenceStatus::Expired;
        assert!(
            item.move_to(RegionalPolicyPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

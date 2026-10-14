use super::*;

use acme_kit::text;

/// Where a overage preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OveragePreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl OveragePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for OveragePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePreference {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub locale: String,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub status: OveragePreferenceStatus,
}

impl OveragePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OveragePreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OveragePreferenceStatus) -> bool {
        match self.status {
            OveragePreferenceStatus::Requested => {
                match next {
                    OveragePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OveragePreferenceStatus::Granted => {
                return next == OveragePreferenceStatus::Revoked || next == OveragePreferenceStatus::Expired;
            }
            OveragePreferenceStatus::Revoked => {
                match next {
                    OveragePreferenceStatus::Expired => {
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

    pub fn move_to(&mut self, next: OveragePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "overage preference", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rotation_days as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OveragePreference {
        OveragePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage preference in status requested.
pub fn new_overage_preference(code: &str, title: &str) -> OveragePreference {
    OveragePreference {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        locale: "".to_string(),
        floor_value: 0.0,
        duration_minutes: 0,
        external_ref: "".to_string(),
        status: OveragePreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_overage_preference("code-61", "title-338");
        assert_eq!(item.status, OveragePreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_overage_preference("code-61", "title-338");
        if item.move_to(OveragePreferenceStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OveragePreferenceStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_overage_preference("code-61", "title-338");
        item.status = OveragePreferenceStatus::Expired;
        assert!(
            item.move_to(OveragePreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

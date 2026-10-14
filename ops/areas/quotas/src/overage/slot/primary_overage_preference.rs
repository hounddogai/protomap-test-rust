use super::*;

use acme_kit::text;

/// Where a primary overage preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryOveragePreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryOveragePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryOveragePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary overage preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreference {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub starts_at: i64,
    pub currency: String,
    pub host: String,
    pub margin: f64,
    pub score: f64,
    pub status: PrimaryOveragePreferenceStatus,
}

impl PrimaryOveragePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryOveragePreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryOveragePreferenceStatus) -> bool {
        match self.status {
            PrimaryOveragePreferenceStatus::Requested => {
                match next {
                    PrimaryOveragePreferenceStatus::Revoked | PrimaryOveragePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryOveragePreferenceStatus::Granted => {
                match next {
                    PrimaryOveragePreferenceStatus::Revoked | PrimaryOveragePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryOveragePreferenceStatus::Revoked => {
                return next == PrimaryOveragePreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryOveragePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary overage preference {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "primary overage preference", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.cost * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryOveragePreference {
        PrimaryOveragePreference {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            host: self.host.clone(),
            margin: self.margin,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary overage preference in status requested.
pub fn new_primary_overage_preference(code: &str, name: &str) -> PrimaryOveragePreference {
    PrimaryOveragePreference {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        starts_at: 0,
        currency: "".to_string(),
        host: "".to_string(),
        margin: 0.0,
        score: 0.0,
        status: PrimaryOveragePreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_overage_preference("code-410", "name-371");
        assert_eq!(item.status, PrimaryOveragePreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_overage_preference("code-410", "name-371");
        if item.move_to(PrimaryOveragePreferenceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryOveragePreferenceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_overage_preference("code-410", "name-371");
        item.status = PrimaryOveragePreferenceStatus::Expired;
        assert!(
            item.move_to(PrimaryOveragePreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

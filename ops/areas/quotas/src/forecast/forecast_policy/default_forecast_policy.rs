use super::*;

use acme_kit::text;

/// Where a default forecast policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultForecastPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultForecastPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultForecastPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default forecast policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicy {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub contact_email: String,
    pub scope: String,
    pub key_id: String,
    pub expires_at: i64,
    pub threshold: f64,
    pub sequence: i64,
    pub status: DefaultForecastPolicyStatus,
}

impl DefaultForecastPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultForecastPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultForecastPolicyStatus) -> bool {
        match self.status {
            DefaultForecastPolicyStatus::Requested => {
                return next == DefaultForecastPolicyStatus::Granted;
            }
            DefaultForecastPolicyStatus::Granted => {
                match next {
                    DefaultForecastPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastPolicyStatus::Revoked => {
                match next {
                    DefaultForecastPolicyStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultForecastPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default forecast policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default forecast policy", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.sequence as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultForecastPolicy {
        DefaultForecastPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            key_id: self.key_id.clone(),
            expires_at: self.expires_at,
            threshold: self.threshold,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default forecast policy in status requested.
pub fn new_default_forecast_policy(code: &str, name: &str) -> DefaultForecastPolicy {
    DefaultForecastPolicy {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        contact_email: "".to_string(),
        scope: "".to_string(),
        key_id: "".to_string(),
        expires_at: 0,
        threshold: 0.0,
        sequence: 0,
        status: DefaultForecastPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_forecast_policy("code-96", "name-271");
        assert_eq!(item.status, DefaultForecastPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_forecast_policy("code-96", "name-271");
        if item.move_to(DefaultForecastPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultForecastPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_forecast_policy("code-96", "name-271");
        item.status = DefaultForecastPolicyStatus::Expired;
        assert!(
            item.move_to(DefaultForecastPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

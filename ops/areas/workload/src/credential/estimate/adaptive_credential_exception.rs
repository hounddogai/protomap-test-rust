use super::*;

use acme_kit::text;

/// Where a adaptive credential exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCredentialExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveCredentialExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveCredentialExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive credential exception as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialException {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub port: i64,
    pub capacity: i64,
    pub category: String,
    pub ends_at: i64,
    pub country: String,
    pub budget: f64,
    pub status: AdaptiveCredentialExceptionStatus,
}

impl AdaptiveCredentialException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCredentialExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCredentialExceptionStatus) -> bool {
        match self.status {
            AdaptiveCredentialExceptionStatus::Requested => {
                return next == AdaptiveCredentialExceptionStatus::Granted;
            }
            AdaptiveCredentialExceptionStatus::Granted => {
                return next == AdaptiveCredentialExceptionStatus::Revoked;
            }
            AdaptiveCredentialExceptionStatus::Revoked => {
                return next == AdaptiveCredentialExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveCredentialExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive credential exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.capacity as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveCredentialException {
        AdaptiveCredentialException {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            capacity: self.capacity,
            category: self.category.clone(),
            ends_at: self.ends_at,
            country: self.country.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive credential exception in status requested.
pub fn new_adaptive_credential_exception(code: &str, label: &str) -> AdaptiveCredentialException {
    AdaptiveCredentialException {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        port: 0,
        capacity: 0,
        category: "".to_string(),
        ends_at: 0,
        country: "".to_string(),
        budget: 0.0,
        status: AdaptiveCredentialExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_credential_exception("code-122", "label-44");
        assert_eq!(item.status, AdaptiveCredentialExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_adaptive_credential_exception("code-122", "label-44");
        if item.move_to(AdaptiveCredentialExceptionStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveCredentialExceptionStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_credential_exception("code-122", "label-44");
        item.status = AdaptiveCredentialExceptionStatus::Expired;
        assert!(
            item.move_to(AdaptiveCredentialExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

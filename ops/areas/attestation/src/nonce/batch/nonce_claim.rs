use super::*;

use acme_kit::text;

/// Where a nonce claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce claim as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceClaim {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub category: String,
    pub ends_at: i64,
    pub status: NonceClaimStatus,
}

impl NonceClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceClaimStatus) -> bool {
        match self.status {
            NonceClaimStatus::Requested => {
                return next == NonceClaimStatus::Expired;
            }
            NonceClaimStatus::Granted => {
                match next {
                    NonceClaimStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceClaimStatus::Revoked => {
                match next {
                    NonceClaimStatus::Expired => {
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

    pub fn move_to(&mut self, next: NonceClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce claim {} cannot move from {} to {}",
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
        format!("{} {}", "nonce claim", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceClaim {
        NonceClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a nonce claim in status requested.
pub fn new_nonce_claim(code: &str, label: &str) -> NonceClaim {
    NonceClaim {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        category: "".to_string(),
        ends_at: 0,
        status: NonceClaimStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_nonce_claim("code-856", "label-470");
        assert_eq!(item.status, NonceClaimStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_nonce_claim("code-856", "label-470");
        if item.move_to(NonceClaimStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceClaimStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_nonce_claim("code-856", "label-470");
        item.status = NonceClaimStatus::Expired;
        assert!(
            item.move_to(NonceClaimStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

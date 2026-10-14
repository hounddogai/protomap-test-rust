use super::*;

use acme_kit::text;

/// Where a nonce preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoncePreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NoncePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NoncePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce preference as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NoncePreference {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub namespace: String,
    pub region: String,
    pub expires_at: i64,
    pub phone: String,
    pub host: String,
    pub floor_value: f64,
    pub discount: f64,
    pub burst: i64,
    pub status: NoncePreferenceStatus,
}

impl NoncePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NoncePreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NoncePreferenceStatus) -> bool {
        match self.status {
            NoncePreferenceStatus::Requested => {
                return next == NoncePreferenceStatus::Granted || next == NoncePreferenceStatus::Revoked;
            }
            NoncePreferenceStatus::Granted => {
                match next {
                    NoncePreferenceStatus::Revoked | NoncePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NoncePreferenceStatus::Revoked => {
                match next {
                    NoncePreferenceStatus::Expired => {
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

    pub fn move_to(&mut self, next: NoncePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce preference {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NoncePreference {
        NoncePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            region: self.region.clone(),
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            host: self.host.clone(),
            floor_value: self.floor_value,
            discount: self.discount,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a nonce preference in status requested.
pub fn new_nonce_preference(code: &str, title: &str) -> NoncePreference {
    NoncePreference {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        region: "".to_string(),
        expires_at: 0,
        phone: "".to_string(),
        host: "".to_string(),
        floor_value: 0.0,
        discount: 0.0,
        burst: 0,
        status: NoncePreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_nonce_preference("code-89", "title-170");
        assert_eq!(item.status, NoncePreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_nonce_preference("code-89", "title-170");
        if item.move_to(NoncePreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NoncePreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_nonce_preference("code-89", "title-170");
        item.status = NoncePreferenceStatus::Expired;
        assert!(
            item.move_to(NoncePreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a nonce profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl NonceProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for NonceProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce profile as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfile {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub sequence: i64,
    pub host: String,
    pub cost: f64,
    pub expires_at: i64,
    pub active: bool,
    pub status: NonceProfileStatus,
}

impl NonceProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceProfileStatus) -> bool {
        match self.status {
            NonceProfileStatus::Draft => {
                return next == NonceProfileStatus::Suspended;
            }
            NonceProfileStatus::Pending => {
                return next == NonceProfileStatus::Active;
            }
            NonceProfileStatus::Active => {
                match next {
                    NonceProfileStatus::Suspended | NonceProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceProfileStatus::Suspended => {
                match next {
                    NonceProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: NonceProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce profile {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceProfile {
        NonceProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            sequence: self.sequence,
            host: self.host.clone(),
            cost: self.cost,
            expires_at: self.expires_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce profile in status draft.
pub fn new_nonce_profile(code: &str, name: &str) -> NonceProfile {
    NonceProfile {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        sequence: 0,
        host: "".to_string(),
        cost: 0.0,
        expires_at: 0,
        active: false,
        status: NonceProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_nonce_profile("code-379", "name-15");
        assert_eq!(item.status, NonceProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_nonce_profile("code-379", "name-15");
        if item.move_to(NonceProfileStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceProfileStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_nonce_profile("code-379", "name-15");
        item.status = NonceProfileStatus::Closed;
        assert!(
            item.move_to(NonceProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

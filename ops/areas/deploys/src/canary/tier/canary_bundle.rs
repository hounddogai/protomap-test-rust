use super::*;

use acme_kit::text;

/// Where a canary bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CanaryBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CanaryBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary bundle as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundle {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub issuer: String,
    pub notes: String,
    pub locale: String,
    pub sequence: i64,
    pub source: String,
    pub updated_at: i64,
    pub title: String,
    pub status: CanaryBundleStatus,
}

impl CanaryBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryBundleStatus) -> bool {
        match self.status {
            CanaryBundleStatus::Requested => {
                match next {
                    CanaryBundleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryBundleStatus::Granted => {
                return next == CanaryBundleStatus::Revoked || next == CanaryBundleStatus::Expired;
            }
            CanaryBundleStatus::Revoked => {
                return next == CanaryBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryBundle {
        CanaryBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            locale: self.locale.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            updated_at: self.updated_at,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a canary bundle in status requested.
pub fn new_canary_bundle(code: &str, name: &str) -> CanaryBundle {
    CanaryBundle {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        issuer: "".to_string(),
        notes: "".to_string(),
        locale: "".to_string(),
        sequence: 0,
        source: "".to_string(),
        updated_at: 0,
        title: "".to_string(),
        status: CanaryBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_canary_bundle("code-487", "name-362");
        assert_eq!(item.status, CanaryBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_canary_bundle("code-487", "name-362");
        if item.move_to(CanaryBundleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryBundleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_canary_bundle("code-487", "name-362");
        item.status = CanaryBundleStatus::Expired;
        assert!(
            item.move_to(CanaryBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a allowlist override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist override as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverride {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub version: i64,
    pub ends_at: i64,
    pub region: String,
    pub archived: bool,
    pub status: AllowlistOverrideStatus,
}

impl AllowlistOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistOverrideStatus) -> bool {
        match self.status {
            AllowlistOverrideStatus::Requested => {
                match next {
                    AllowlistOverrideStatus::Revoked | AllowlistOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistOverrideStatus::Granted => {
                return next == AllowlistOverrideStatus::Expired;
            }
            AllowlistOverrideStatus::Revoked => {
                return next == AllowlistOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.version as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistOverride {
        AllowlistOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            version: self.version,
            ends_at: self.ends_at,
            region: self.region.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a allowlist override in status requested.
pub fn new_allowlist_override(code: &str, title: &str) -> AllowlistOverride {
    AllowlistOverride {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        version: 0,
        ends_at: 0,
        region: "".to_string(),
        archived: false,
        status: AllowlistOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_override("code-223", "title-800");
        assert_eq!(item.status, AllowlistOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_allowlist_override("code-223", "title-800");
        if item.move_to(AllowlistOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_override("code-223", "title-800");
        item.status = AllowlistOverrideStatus::Expired;
        assert!(
            item.move_to(AllowlistOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

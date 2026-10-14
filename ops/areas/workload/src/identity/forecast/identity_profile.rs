use super::*;

use acme_kit::text;

/// Where a identity profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityProfile {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub checksum: String,
    pub external_ref: String,
    pub locale: String,
    pub timezone: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub tags: Vec<String>,
    pub status: IdentityProfileStatus,
}

impl IdentityProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityProfileStatus) -> bool {
        match self.status {
            IdentityProfileStatus::Requested => {
                match next {
                    IdentityProfileStatus::Granted | IdentityProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityProfileStatus::Granted => {
                match next {
                    IdentityProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityProfileStatus::Revoked => {
                match next {
                    IdentityProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: IdentityProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity profile {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityProfile {
        IdentityProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            timezone: self.timezone.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a identity profile in status requested.
pub fn new_identity_profile(code: &str, name: &str) -> IdentityProfile {
    IdentityProfile {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        checksum: "".to_string(),
        external_ref: "".to_string(),
        locale: "".to_string(),
        timezone: "".to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        tags: Vec::new(),
        status: IdentityProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_profile("code-409", "name-873");
        assert_eq!(item.status, IdentityProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_identity_profile("code-409", "name-873");
        if item.move_to(IdentityProfileStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityProfileStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_profile("code-409", "name-873");
        item.status = IdentityProfileStatus::Expired;
        assert!(
            item.move_to(IdentityProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

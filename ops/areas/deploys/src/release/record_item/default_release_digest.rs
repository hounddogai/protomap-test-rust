use super::*;

use acme_kit::text;

/// Where a default release digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultReleaseDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultReleaseDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default release digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigest {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub starts_at: i64,
    pub region: String,
    pub status: DefaultReleaseDigestStatus,
}

impl DefaultReleaseDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseDigestStatus) -> bool {
        match self.status {
            DefaultReleaseDigestStatus::Requested => {
                match next {
                    DefaultReleaseDigestStatus::Revoked | DefaultReleaseDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseDigestStatus::Granted => {
                match next {
                    DefaultReleaseDigestStatus::Revoked | DefaultReleaseDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseDigestStatus::Revoked => {
                match next {
                    DefaultReleaseDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultReleaseDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release digest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.limit as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseDigest {
        DefaultReleaseDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            starts_at: self.starts_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default release digest in status requested.
pub fn new_default_release_digest(code: &str, name: &str) -> DefaultReleaseDigest {
    DefaultReleaseDigest {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        starts_at: 0,
        region: "".to_string(),
        status: DefaultReleaseDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_release_digest("code-69", "name-117");
        assert_eq!(item.status, DefaultReleaseDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_release_digest("code-69", "name-117");
        if item.move_to(DefaultReleaseDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultReleaseDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_release_digest("code-69", "name-117");
        item.status = DefaultReleaseDigestStatus::Expired;
        assert!(
            item.move_to(DefaultReleaseDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

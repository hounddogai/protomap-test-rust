use super::*;

use acme_kit::text;

/// Where a global tenant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTenantDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalTenantDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for GlobalTenantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global tenant digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigest {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub margin: f64,
    pub ends_at: i64,
    pub score: f64,
    pub channel: String,
    pub status: GlobalTenantDigestStatus,
}

impl GlobalTenantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTenantDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTenantDigestStatus) -> bool {
        match self.status {
            GlobalTenantDigestStatus::Open => {
                match next {
                    GlobalTenantDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTenantDigestStatus::InReview => {
                match next {
                    GlobalTenantDigestStatus::Resolved | GlobalTenantDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTenantDigestStatus::Resolved => {
                match next {
                    GlobalTenantDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTenantDigestStatus::Reopened => {
                return next == GlobalTenantDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTenantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global tenant digest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTenantDigest {
        GlobalTenantDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            margin: self.margin,
            ends_at: self.ends_at,
            score: self.score,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a global tenant digest in status open.
pub fn new_global_tenant_digest(code: &str, name: &str) -> GlobalTenantDigest {
    GlobalTenantDigest {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        margin: 0.0,
        ends_at: 0,
        score: 0.0,
        channel: "".to_string(),
        status: GlobalTenantDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_tenant_digest("code-940", "name-127");
        assert_eq!(item.status, GlobalTenantDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_global_tenant_digest("code-940", "name-127");
        if item.move_to(GlobalTenantDigestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTenantDigestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_tenant_digest("code-940", "name-127");
        item.status = GlobalTenantDigestStatus::Archived;
        assert!(
            item.move_to(GlobalTenantDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled tenant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledTenantDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledTenantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled tenant digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigest {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub score: f64,
    pub title: String,
    pub version: i64,
    pub capacity: i64,
    pub ends_at: i64,
    pub batch_size: i64,
    pub status: ScheduledTenantDigestStatus,
}

impl ScheduledTenantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantDigestStatus) -> bool {
        match self.status {
            ScheduledTenantDigestStatus::Requested => {
                match next {
                    ScheduledTenantDigestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantDigestStatus::Granted => {
                return next == ScheduledTenantDigestStatus::Revoked;
            }
            ScheduledTenantDigestStatus::Revoked => {
                match next {
                    ScheduledTenantDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledTenantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant digest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled tenant digest", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantDigest {
        ScheduledTenantDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            approved: self.approved,
            score: self.score,
            title: self.title.clone(),
            version: self.version,
            capacity: self.capacity,
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled tenant digest in status requested.
pub fn new_scheduled_tenant_digest(code: &str, label: &str) -> ScheduledTenantDigest {
    ScheduledTenantDigest {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        approved: false,
        score: 0.0,
        title: "".to_string(),
        version: 0,
        capacity: 0,
        ends_at: 0,
        batch_size: 0,
        status: ScheduledTenantDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_tenant_digest("code-922", "label-580");
        assert_eq!(item.status, ScheduledTenantDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_scheduled_tenant_digest("code-922", "label-580");
        if item.move_to(ScheduledTenantDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_tenant_digest("code-922", "label-580");
        item.status = ScheduledTenantDigestStatus::Expired;
        assert!(
            item.move_to(ScheduledTenantDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

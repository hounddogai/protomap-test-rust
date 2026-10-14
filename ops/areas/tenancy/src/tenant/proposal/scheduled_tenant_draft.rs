use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled tenant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledTenantDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledTenantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled tenant draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDraft {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub capacity: i64,
    pub status: ScheduledTenantDraftStatus,
}

impl ScheduledTenantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantDraftStatus) -> bool {
        match self.status {
            ScheduledTenantDraftStatus::Requested => {
                return next == ScheduledTenantDraftStatus::Revoked || next == ScheduledTenantDraftStatus::Expired;
            }
            ScheduledTenantDraftStatus::Granted => {
                return next == ScheduledTenantDraftStatus::Expired;
            }
            ScheduledTenantDraftStatus::Revoked => {
                match next {
                    ScheduledTenantDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledTenantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled tenant draft", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.cost - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantDraft {
        ScheduledTenantDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled tenant draft in status requested.
pub fn new_scheduled_tenant_draft(code: &str, name: &str) -> ScheduledTenantDraft {
    ScheduledTenantDraft {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        capacity: 0,
        status: ScheduledTenantDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_tenant_draft("code-805", "name-342");
        assert_eq!(item.status, ScheduledTenantDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_scheduled_tenant_draft("code-805", "name-342");
        if item.move_to(ScheduledTenantDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_tenant_draft("code-805", "name-342");
        item.status = ScheduledTenantDraftStatus::Expired;
        assert!(
            item.move_to(ScheduledTenantDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

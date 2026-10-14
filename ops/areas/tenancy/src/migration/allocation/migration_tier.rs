use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a migration tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MigrationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTier {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub maximum: f64,
    pub reviewer: String,
    pub expires_at: i64,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub status: MigrationTierStatus,
}

impl MigrationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationTierStatus) -> bool {
        match self.status {
            MigrationTierStatus::New => {
                return next == MigrationTierStatus::Done || next == MigrationTierStatus::Failed;
            }
            MigrationTierStatus::Queued => {
                match next {
                    MigrationTierStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationTierStatus::Running => {
                return next == MigrationTierStatus::Done || next == MigrationTierStatus::Failed;
            }
            MigrationTierStatus::Done => {
                return next == MigrationTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration tier {} cannot move from {} to {}",
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
        format!("{} {}", "migration tier", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.expires_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationTier {
        MigrationTier {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration tier in status new.
pub fn new_migration_tier(code: &str, name: &str) -> MigrationTier {
    MigrationTier {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        maximum: 0.0,
        reviewer: "".to_string(),
        expires_at: 0,
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        status: MigrationTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_migration_tier("code-273", "name-455");
        assert_eq!(item.status, MigrationTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_migration_tier("code-273", "name-455");
        if item.move_to(MigrationTierStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationTierStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_migration_tier("code-273", "name-455");
        item.status = MigrationTierStatus::Failed;
        assert!(item.move_to(MigrationTierStatus::New).is_err(), "expected move to fail");
    }
}

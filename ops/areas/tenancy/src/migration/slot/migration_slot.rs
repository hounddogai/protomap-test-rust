use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a migration slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlot {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub active: bool,
    pub minimum: f64,
    pub weight: f64,
    pub timezone: String,
    pub currency: String,
    pub namespace: String,
    pub status: MigrationSlotStatus,
}

impl MigrationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationSlotStatus) -> bool {
        match self.status {
            MigrationSlotStatus::Created => {
                match next {
                    MigrationSlotStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationSlotStatus::Validated => {
                match next {
                    MigrationSlotStatus::Published | MigrationSlotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationSlotStatus::Published => {
                return next == MigrationSlotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationSlot {
        MigrationSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            active: self.active,
            minimum: self.minimum,
            weight: self.weight,
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a migration slot in status created.
pub fn new_migration_slot(code: &str, title: &str) -> MigrationSlot {
    MigrationSlot {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        active: false,
        minimum: 0.0,
        weight: 0.0,
        timezone: "".to_string(),
        currency: "".to_string(),
        namespace: "".to_string(),
        status: MigrationSlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_slot("code-417", "title-744");
        assert_eq!(item.status, MigrationSlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_migration_slot("code-417", "title-744");
        if item.move_to(MigrationSlotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationSlotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_slot("code-417", "title-744");
        item.status = MigrationSlotStatus::Retired;
        assert!(
            item.move_to(MigrationSlotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

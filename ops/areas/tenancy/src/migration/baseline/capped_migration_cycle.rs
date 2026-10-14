use super::*;

use acme_kit::text;

/// Where a capped migration cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedMigrationCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedMigrationCycleStatus {
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

impl std::fmt::Display for CappedMigrationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped migration cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationCycle {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub source: String,
    pub ends_at: i64,
    pub status: CappedMigrationCycleStatus,
}

impl CappedMigrationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedMigrationCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedMigrationCycleStatus) -> bool {
        match self.status {
            CappedMigrationCycleStatus::Draft => {
                match next {
                    CappedMigrationCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedMigrationCycleStatus::Pending => {
                match next {
                    CappedMigrationCycleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedMigrationCycleStatus::Active => {
                return next == CappedMigrationCycleStatus::Suspended || next == CappedMigrationCycleStatus::Closed;
            }
            CappedMigrationCycleStatus::Suspended => {
                match next {
                    CappedMigrationCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedMigrationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped migration cycle {} cannot move from {} to {}",
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
        format!("{} {}", "capped migration cycle", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> CappedMigrationCycle {
        CappedMigrationCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            source: self.source.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped migration cycle in status draft.
pub fn new_capped_migration_cycle(code: &str, name: &str) -> CappedMigrationCycle {
    CappedMigrationCycle {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        source: "".to_string(),
        ends_at: 0,
        status: CappedMigrationCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_migration_cycle("code-551", "name-566");
        assert_eq!(item.status, CappedMigrationCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_capped_migration_cycle("code-551", "name-566");
        if item.move_to(CappedMigrationCycleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedMigrationCycleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_migration_cycle("code-551", "name-566");
        item.status = CappedMigrationCycleStatus::Closed;
        assert!(
            item.move_to(CappedMigrationCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

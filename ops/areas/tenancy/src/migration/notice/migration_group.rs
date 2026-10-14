use super::*;

use acme_kit::text;

/// Where a migration group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroup {
    pub code: String,
    pub title: String,
    pub description: String,
    pub rotation_days: i64,
    pub label: String,
    pub cipher: String,
    pub status: MigrationGroupStatus,
}

impl MigrationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationGroupStatus) -> bool {
        match self.status {
            MigrationGroupStatus::Created => {
                return next == MigrationGroupStatus::Validated || next == MigrationGroupStatus::Published;
            }
            MigrationGroupStatus::Validated => {
                return next == MigrationGroupStatus::Published || next == MigrationGroupStatus::Retired;
            }
            MigrationGroupStatus::Published => {
                match next {
                    MigrationGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: MigrationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> MigrationGroup {
        MigrationGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a migration group in status created.
pub fn new_migration_group(code: &str, title: &str) -> MigrationGroup {
    MigrationGroup {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        rotation_days: 0,
        label: "".to_string(),
        cipher: "".to_string(),
        status: MigrationGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_group("code-475", "title-476");
        assert_eq!(item.status, MigrationGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_migration_group("code-475", "title-476");
        if item.move_to(MigrationGroupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationGroupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_group("code-475", "title-476");
        item.status = MigrationGroupStatus::Retired;
        assert!(
            item.move_to(MigrationGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

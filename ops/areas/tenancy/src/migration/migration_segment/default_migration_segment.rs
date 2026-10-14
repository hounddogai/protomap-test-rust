use super::*;

use acme_kit::text;

/// Where a default migration segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMigrationSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultMigrationSegmentStatus {
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

impl std::fmt::Display for DefaultMigrationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default migration segment as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSegment {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub version: i64,
    pub contact_email: String,
    pub cipher: String,
    pub window_days: i64,
    pub limit: i64,
    pub ceiling: f64,
    pub status: DefaultMigrationSegmentStatus,
}

impl DefaultMigrationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMigrationSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMigrationSegmentStatus) -> bool {
        match self.status {
            DefaultMigrationSegmentStatus::Draft => {
                return next == DefaultMigrationSegmentStatus::Pending;
            }
            DefaultMigrationSegmentStatus::Pending => {
                return next == DefaultMigrationSegmentStatus::Suspended;
            }
            DefaultMigrationSegmentStatus::Active => {
                match next {
                    DefaultMigrationSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationSegmentStatus::Suspended => {
                match next {
                    DefaultMigrationSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultMigrationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default migration segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.limit as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMigrationSegment {
        DefaultMigrationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            version: self.version,
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            window_days: self.window_days,
            limit: self.limit,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default migration segment in status draft.
pub fn new_default_migration_segment(code: &str, name: &str) -> DefaultMigrationSegment {
    DefaultMigrationSegment {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        version: 0,
        contact_email: "".to_string(),
        cipher: "".to_string(),
        window_days: 0,
        limit: 0,
        ceiling: 0.0,
        status: DefaultMigrationSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_migration_segment("code-36", "name-406");
        assert_eq!(item.status, DefaultMigrationSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_migration_segment("code-36", "name-406");
        if item.move_to(DefaultMigrationSegmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultMigrationSegmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_migration_segment("code-36", "name-406");
        item.status = DefaultMigrationSegmentStatus::Closed;
        assert!(
            item.move_to(DefaultMigrationSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a migration audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MigrationAuditStatus {
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

impl std::fmt::Display for MigrationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAudit {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub owner_id: String,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub visible: bool,
    pub status: MigrationAuditStatus,
}

impl MigrationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationAuditStatus) -> bool {
        match self.status {
            MigrationAuditStatus::Open => {
                return next == MigrationAuditStatus::Reopened;
            }
            MigrationAuditStatus::InReview => {
                return next == MigrationAuditStatus::Resolved || next == MigrationAuditStatus::Archived;
            }
            MigrationAuditStatus::Resolved => {
                match next {
                    MigrationAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationAuditStatus::Reopened => {
                match next {
                    MigrationAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: MigrationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationAudit {
        MigrationAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a migration audit in status open.
pub fn new_migration_audit(code: &str, label: &str) -> MigrationAudit {
    MigrationAudit {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        grace_days: 0,
        owner_id: "".to_string(),
        ttl_seconds: 0,
        minimum: 0.0,
        visible: false,
        status: MigrationAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_migration_audit("code-220", "label-157");
        assert_eq!(item.status, MigrationAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_migration_audit("code-220", "label-157");
        if item.move_to(MigrationAuditStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationAuditStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_migration_audit("code-220", "label-157");
        item.status = MigrationAuditStatus::Archived;
        assert!(
            item.move_to(MigrationAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

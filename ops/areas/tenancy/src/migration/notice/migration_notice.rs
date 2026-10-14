use super::*;

use acme_kit::text;

/// Where a migration notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNotice {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub source: String,
    pub ratio: f64,
    pub maximum: f64,
    pub region: String,
    pub status: MigrationNoticeStatus,
}

impl MigrationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationNoticeStatus) -> bool {
        match self.status {
            MigrationNoticeStatus::Proposed => {
                match next {
                    MigrationNoticeStatus::Approved | MigrationNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationNoticeStatus::Approved => {
                match next {
                    MigrationNoticeStatus::Rejected | MigrationNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationNoticeStatus::Rejected => {
                return next == MigrationNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quantity as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationNotice {
        MigrationNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            tags: self.tags.clone(),
            quantity: self.quantity,
            source: self.source.clone(),
            ratio: self.ratio,
            maximum: self.maximum,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a migration notice in status proposed.
pub fn new_migration_notice(code: &str, title: &str) -> MigrationNotice {
    MigrationNotice {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        tags: Vec::new(),
        quantity: 0,
        source: "".to_string(),
        ratio: 0.0,
        maximum: 0.0,
        region: "".to_string(),
        status: MigrationNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_notice("code-9", "title-199");
        assert_eq!(item.status, MigrationNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_notice("code-9", "title-199");
        if item.move_to(MigrationNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_notice("code-9", "title-199");
        item.status = MigrationNoticeStatus::Withdrawn;
        assert!(
            item.move_to(MigrationNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

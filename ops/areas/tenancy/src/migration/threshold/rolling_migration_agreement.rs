use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling migration agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingMigrationAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingMigrationAgreementStatus {
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

impl std::fmt::Display for RollingMigrationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling migration agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationAgreement {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub maximum: f64,
    pub pinned: bool,
    pub ceiling: f64,
    pub created_at: i64,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub budget: f64,
    pub status: RollingMigrationAgreementStatus,
}

impl RollingMigrationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingMigrationAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingMigrationAgreementStatus) -> bool {
        match self.status {
            RollingMigrationAgreementStatus::New => {
                match next {
                    RollingMigrationAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingMigrationAgreementStatus::Queued => {
                return next == RollingMigrationAgreementStatus::Done;
            }
            RollingMigrationAgreementStatus::Running => {
                match next {
                    RollingMigrationAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingMigrationAgreementStatus::Done => {
                return next == RollingMigrationAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingMigrationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling migration agreement {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.budget - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingMigrationAgreement {
        RollingMigrationAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            visible: self.visible,
            maximum: self.maximum,
            pinned: self.pinned,
            ceiling: self.ceiling,
            created_at: self.created_at,
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling migration agreement in status new.
pub fn new_rolling_migration_agreement(code: &str, title: &str) -> RollingMigrationAgreement {
    RollingMigrationAgreement {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        visible: false,
        maximum: 0.0,
        pinned: false,
        ceiling: 0.0,
        created_at: 0,
        starts_at: 0,
        tags: Vec::new(),
        budget: 0.0,
        status: RollingMigrationAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_migration_agreement("code-136", "title-346");
        assert_eq!(item.status, RollingMigrationAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_migration_agreement("code-136", "title-346");
        if item.move_to(RollingMigrationAgreementStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingMigrationAgreementStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_migration_agreement("code-136", "title-346");
        item.status = RollingMigrationAgreementStatus::Failed;
        assert!(
            item.move_to(RollingMigrationAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

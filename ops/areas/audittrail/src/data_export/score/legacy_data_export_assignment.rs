use super::*;

use acme_kit::text;

/// Where a legacy data export assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDataExportAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyDataExportAssignmentStatus {
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

impl std::fmt::Display for LegacyDataExportAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy data export assignment as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAssignment {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub created_at: i64,
    pub revision: i64,
    pub rate: f64,
    pub discount: f64,
    pub status: LegacyDataExportAssignmentStatus,
}

impl LegacyDataExportAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDataExportAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDataExportAssignmentStatus) -> bool {
        match self.status {
            LegacyDataExportAssignmentStatus::Open => {
                return next == LegacyDataExportAssignmentStatus::Reopened
                    || next == LegacyDataExportAssignmentStatus::Archived;
            }
            LegacyDataExportAssignmentStatus::InReview => {
                match next {
                    LegacyDataExportAssignmentStatus::Resolved | LegacyDataExportAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDataExportAssignmentStatus::Resolved => {
                match next {
                    LegacyDataExportAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDataExportAssignmentStatus::Reopened => {
                match next {
                    LegacyDataExportAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyDataExportAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy data export assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDataExportAssignment {
        LegacyDataExportAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            created_at: self.created_at,
            revision: self.revision,
            rate: self.rate,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy data export assignment in status open.
pub fn new_legacy_data_export_assignment(code: &str, name: &str) -> LegacyDataExportAssignment {
    LegacyDataExportAssignment {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        created_at: 0,
        revision: 0,
        rate: 0.0,
        discount: 0.0,
        status: LegacyDataExportAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_data_export_assignment("code-563", "name-896");
        assert_eq!(item.status, LegacyDataExportAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_data_export_assignment("code-563", "name-896");
        if item.move_to(LegacyDataExportAssignmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDataExportAssignmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_data_export_assignment("code-563", "name-896");
        item.status = LegacyDataExportAssignmentStatus::Archived;
        assert!(
            item.move_to(LegacyDataExportAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

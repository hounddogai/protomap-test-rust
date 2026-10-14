use super::*;

use acme_kit::text;

/// Where a migration segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSegment {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub budget: f64,
    pub principal: String,
    pub issuer: String,
    pub expires_at: i64,
    pub ceiling: f64,
    pub tenant_id: String,
    pub status: MigrationSegmentStatus,
}

impl MigrationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationSegmentStatus) -> bool {
        match self.status {
            MigrationSegmentStatus::Created => {
                match next {
                    MigrationSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationSegmentStatus::Validated => {
                return next == MigrationSegmentStatus::Retired;
            }
            MigrationSegmentStatus::Published => {
                return next == MigrationSegmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration segment {} cannot move from {} to {}",
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
        format!("{} {}", "migration segment", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.expires_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationSegment {
        MigrationSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            budget: self.budget,
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a migration segment in status created.
pub fn new_migration_segment(code: &str, title: &str) -> MigrationSegment {
    MigrationSegment {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        budget: 0.0,
        principal: "".to_string(),
        issuer: "".to_string(),
        expires_at: 0,
        ceiling: 0.0,
        tenant_id: "".to_string(),
        status: MigrationSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_segment("code-155", "title-271");
        assert_eq!(item.status, MigrationSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_migration_segment("code-155", "title-271");
        if item.move_to(MigrationSegmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationSegmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_segment("code-155", "title-271");
        item.status = MigrationSegmentStatus::Retired;
        assert!(
            item.move_to(MigrationSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a migration summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSummary {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub label: String,
    pub revision: i64,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub minimum: f64,
    pub port: i64,
    pub created_at: i64,
    pub status: MigrationSummaryStatus,
}

impl MigrationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationSummaryStatus) -> bool {
        match self.status {
            MigrationSummaryStatus::Proposed => {
                match next {
                    MigrationSummaryStatus::Approved | MigrationSummaryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationSummaryStatus::Approved => {
                match next {
                    MigrationSummaryStatus::Rejected | MigrationSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationSummaryStatus::Rejected => {
                match next {
                    MigrationSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration summary", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationSummary {
        MigrationSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            label: self.label.clone(),
            revision: self.revision,
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            minimum: self.minimum,
            port: self.port,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration summary in status proposed.
pub fn new_migration_summary(code: &str, name: &str) -> MigrationSummary {
    MigrationSummary {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        label: "".to_string(),
        revision: 0,
        duration_minutes: 0,
        expires_at: 0,
        minimum: 0.0,
        port: 0,
        created_at: 0,
        status: MigrationSummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_summary("code-327", "name-149");
        assert_eq!(item.status, MigrationSummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_summary("code-327", "name-149");
        if item.move_to(MigrationSummaryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationSummaryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_summary("code-327", "name-149");
        item.status = MigrationSummaryStatus::Withdrawn;
        assert!(
            item.move_to(MigrationSummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

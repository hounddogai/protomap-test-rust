use super::*;

use acme_kit::text;

/// Where a runbook target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTarget {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub priority: i64,
    pub quota_units: i64,
    pub region: String,
    pub status: RunbookTargetStatus,
}

impl RunbookTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookTargetStatus) -> bool {
        match self.status {
            RunbookTargetStatus::Created => {
                return next == RunbookTargetStatus::Published || next == RunbookTargetStatus::Retired;
            }
            RunbookTargetStatus::Validated => {
                match next {
                    RunbookTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookTargetStatus::Published => {
                match next {
                    RunbookTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: RunbookTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook target", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookTarget {
        RunbookTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            priority: self.priority,
            quota_units: self.quota_units,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a runbook target in status created.
pub fn new_runbook_target(code: &str, title: &str) -> RunbookTarget {
    RunbookTarget {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        priority: 0,
        quota_units: 0,
        region: "".to_string(),
        status: RunbookTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_target("code-663", "title-293");
        assert_eq!(item.status, RunbookTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_runbook_target("code-663", "title-293");
        if item.move_to(RunbookTargetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookTargetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_target("code-663", "title-293");
        item.status = RunbookTargetStatus::Retired;
        assert!(
            item.move_to(RunbookTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a runbook template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RunbookTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RunbookTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplate {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub margin: f64,
    pub checksum: String,
    pub priority: i64,
    pub reviewer: String,
    pub title: String,
    pub external_ref: String,
    pub country: String,
    pub status: RunbookTemplateStatus,
}

impl RunbookTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookTemplateStatus) -> bool {
        match self.status {
            RunbookTemplateStatus::Proposed => {
                match next {
                    RunbookTemplateStatus::Approved | RunbookTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookTemplateStatus::Approved => {
                return next == RunbookTemplateStatus::Rejected || next == RunbookTemplateStatus::Withdrawn;
            }
            RunbookTemplateStatus::Rejected => {
                return next == RunbookTemplateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook template", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.priority as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookTemplate {
        RunbookTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            margin: self.margin,
            checksum: self.checksum.clone(),
            priority: self.priority,
            reviewer: self.reviewer.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a runbook template in status proposed.
pub fn new_runbook_template(code: &str, name: &str) -> RunbookTemplate {
    RunbookTemplate {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        margin: 0.0,
        checksum: "".to_string(),
        priority: 0,
        reviewer: "".to_string(),
        title: "".to_string(),
        external_ref: "".to_string(),
        country: "".to_string(),
        status: RunbookTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_runbook_template("code-956", "name-657");
        assert_eq!(item.status, RunbookTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_runbook_template("code-956", "name-657");
        if item.move_to(RunbookTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_runbook_template("code-956", "name-657");
        item.status = RunbookTemplateStatus::Withdrawn;
        assert!(
            item.move_to(RunbookTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

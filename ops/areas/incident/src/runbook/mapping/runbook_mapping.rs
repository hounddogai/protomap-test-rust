use super::*;

use acme_kit::text;

/// Where a runbook mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMapping {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub threshold: f64,
    pub issuer: String,
    pub expires_at: i64,
    pub description: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub approved: bool,
    pub status: RunbookMappingStatus,
}

impl RunbookMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookMappingStatus) -> bool {
        match self.status {
            RunbookMappingStatus::Created => {
                match next {
                    RunbookMappingStatus::Validated | RunbookMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookMappingStatus::Validated => {
                return next == RunbookMappingStatus::Retired;
            }
            RunbookMappingStatus::Published => {
                match next {
                    RunbookMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: RunbookMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook mapping {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookMapping {
        RunbookMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            description: self.description.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a runbook mapping in status created.
pub fn new_runbook_mapping(code: &str, label: &str) -> RunbookMapping {
    RunbookMapping {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        threshold: 0.0,
        issuer: "".to_string(),
        expires_at: 0,
        description: "".to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        approved: false,
        status: RunbookMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_mapping("code-199", "label-848");
        assert_eq!(item.status, RunbookMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_runbook_mapping("code-199", "label-848");
        if item.move_to(RunbookMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_mapping("code-199", "label-848");
        item.status = RunbookMappingStatus::Retired;
        assert!(
            item.move_to(RunbookMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

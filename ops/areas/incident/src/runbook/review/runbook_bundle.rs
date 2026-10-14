use super::*;

use acme_kit::text;

/// Where a runbook bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RunbookBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RunbookBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundle {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub country: String,
    pub key_id: String,
    pub archived: bool,
    pub region: String,
    pub timezone: String,
    pub created_at: i64,
    pub host: String,
    pub tags: Vec<String>,
    pub status: RunbookBundleStatus,
}

impl RunbookBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookBundleStatus) -> bool {
        match self.status {
            RunbookBundleStatus::Proposed => {
                match next {
                    RunbookBundleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookBundleStatus::Approved => {
                match next {
                    RunbookBundleStatus::Rejected | RunbookBundleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookBundleStatus::Rejected => {
                match next {
                    RunbookBundleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RunbookBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook bundle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> RunbookBundle {
        RunbookBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            archived: self.archived,
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            created_at: self.created_at,
            host: self.host.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook bundle in status proposed.
pub fn new_runbook_bundle(code: &str, label: &str) -> RunbookBundle {
    RunbookBundle {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        country: "".to_string(),
        key_id: "".to_string(),
        archived: false,
        region: "".to_string(),
        timezone: "".to_string(),
        created_at: 0,
        host: "".to_string(),
        tags: Vec::new(),
        status: RunbookBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_runbook_bundle("code-596", "label-626");
        assert_eq!(item.status, RunbookBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_runbook_bundle("code-596", "label-626");
        if item.move_to(RunbookBundleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookBundleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_runbook_bundle("code-596", "label-626");
        item.status = RunbookBundleStatus::Withdrawn;
        assert!(
            item.move_to(RunbookBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

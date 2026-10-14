use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a runbook digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RunbookDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RunbookDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook digest as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigest {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub currency: String,
    pub score: f64,
    pub threshold: f64,
    pub visible: bool,
    pub audience: String,
    pub port: i64,
    pub status: RunbookDigestStatus,
}

impl RunbookDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookDigestStatus) -> bool {
        match self.status {
            RunbookDigestStatus::Proposed => {
                match next {
                    RunbookDigestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookDigestStatus::Approved => {
                match next {
                    RunbookDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookDigestStatus::Rejected => {
                return next == RunbookDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook digest", self.code)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookDigest {
        RunbookDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            score: self.score,
            threshold: self.threshold,
            visible: self.visible,
            audience: self.audience.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook digest in status proposed.
pub fn new_runbook_digest(code: &str, title: &str) -> RunbookDigest {
    RunbookDigest {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        currency: "".to_string(),
        score: 0.0,
        threshold: 0.0,
        visible: false,
        audience: "".to_string(),
        port: 0,
        status: RunbookDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_runbook_digest("code-426", "title-481");
        assert_eq!(item.status, RunbookDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_runbook_digest("code-426", "title-481");
        if item.move_to(RunbookDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_runbook_digest("code-426", "title-481");
        item.status = RunbookDigestStatus::Withdrawn;
        assert!(
            item.move_to(RunbookDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

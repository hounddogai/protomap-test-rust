use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default runbook assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRunbookAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRunbookAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DefaultRunbookAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default runbook assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignment {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub capacity: i64,
    pub notes: String,
    pub segment: String,
    pub external_ref: String,
    pub status: DefaultRunbookAssignmentStatus,
}

impl DefaultRunbookAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRunbookAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRunbookAssignmentStatus) -> bool {
        match self.status {
            DefaultRunbookAssignmentStatus::Idle => {
                return next == DefaultRunbookAssignmentStatus::Warming;
            }
            DefaultRunbookAssignmentStatus::Warming => {
                match next {
                    DefaultRunbookAssignmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRunbookAssignmentStatus::Ready => {
                match next {
                    DefaultRunbookAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRunbookAssignmentStatus::Draining => {
                return next == DefaultRunbookAssignmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRunbookAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default runbook assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.capacity as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRunbookAssignment {
        DefaultRunbookAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            capacity: self.capacity,
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default runbook assignment in status idle.
pub fn new_default_runbook_assignment(code: &str, name: &str) -> DefaultRunbookAssignment {
    DefaultRunbookAssignment {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        capacity: 0,
        notes: "".to_string(),
        segment: "".to_string(),
        external_ref: "".to_string(),
        status: DefaultRunbookAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_runbook_assignment("code-646", "name-962");
        assert_eq!(item.status, DefaultRunbookAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_runbook_assignment("code-646", "name-962");
        if item.move_to(DefaultRunbookAssignmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRunbookAssignmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_runbook_assignment("code-646", "name-962");
        item.status = DefaultRunbookAssignmentStatus::Stopped;
        assert!(
            item.move_to(DefaultRunbookAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

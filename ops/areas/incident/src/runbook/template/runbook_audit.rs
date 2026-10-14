use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a runbook audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RunbookAuditStatus {
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

impl std::fmt::Display for RunbookAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAudit {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub maximum: f64,
    pub status: RunbookAuditStatus,
}

impl RunbookAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookAuditStatus) -> bool {
        match self.status {
            RunbookAuditStatus::Idle => {
                return next == RunbookAuditStatus::Ready || next == RunbookAuditStatus::Draining;
            }
            RunbookAuditStatus::Warming => {
                match next {
                    RunbookAuditStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAuditStatus::Ready => {
                return next == RunbookAuditStatus::Draining || next == RunbookAuditStatus::Stopped;
            }
            RunbookAuditStatus::Draining => {
                return next == RunbookAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> RunbookAudit {
        RunbookAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a runbook audit in status idle.
pub fn new_runbook_audit(code: &str, label: &str) -> RunbookAudit {
    RunbookAudit {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        maximum: 0.0,
        status: RunbookAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_runbook_audit("code-706", "label-582");
        assert_eq!(item.status, RunbookAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_runbook_audit("code-706", "label-582");
        if item.move_to(RunbookAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_runbook_audit("code-706", "label-582");
        item.status = RunbookAuditStatus::Stopped;
        assert!(item.move_to(RunbookAuditStatus::Idle).is_err(), "expected move to fail");
    }
}

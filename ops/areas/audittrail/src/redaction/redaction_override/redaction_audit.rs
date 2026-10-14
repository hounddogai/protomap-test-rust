use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a redaction audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RedactionAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAudit {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub port: i64,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub status: RedactionAuditStatus,
}

impl RedactionAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionAuditStatus) -> bool {
        match self.status {
            RedactionAuditStatus::Planned => {
                match next {
                    RedactionAuditStatus::Completed | RedactionAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAuditStatus::Scheduled => {
                match next {
                    RedactionAuditStatus::InProgress | RedactionAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAuditStatus::InProgress => {
                return next == RedactionAuditStatus::Completed;
            }
            RedactionAuditStatus::Completed => {
                match next {
                    RedactionAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RedactionAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionAudit {
        RedactionAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            port: self.port,
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction audit in status planned.
pub fn new_redaction_audit(code: &str, title: &str) -> RedactionAudit {
    RedactionAudit {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        port: 0,
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        status: RedactionAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_audit("code-956", "title-143");
        assert_eq!(item.status, RedactionAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_redaction_audit("code-956", "title-143");
        if item.move_to(RedactionAuditStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionAuditStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_audit("code-956", "title-143");
        item.status = RedactionAuditStatus::Cancelled;
        assert!(
            item.move_to(RedactionAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

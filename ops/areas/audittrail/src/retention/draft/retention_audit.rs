use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a retention audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RetentionAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAudit {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub issuer: String,
    pub attributes: BTreeMap<String, String>,
    pub audience: String,
    pub ends_at: i64,
    pub archived: bool,
    pub quantity: i64,
    pub margin: f64,
    pub active: bool,
    pub status: RetentionAuditStatus,
}

impl RetentionAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionAuditStatus) -> bool {
        match self.status {
            RetentionAuditStatus::New => {
                match next {
                    RetentionAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAuditStatus::Queued => {
                match next {
                    RetentionAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAuditStatus::Running => {
                return next == RetentionAuditStatus::Failed;
            }
            RetentionAuditStatus::Done => {
                return next == RetentionAuditStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionAudit {
        RetentionAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            issuer: self.issuer.clone(),
            attributes: self.attributes.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            archived: self.archived,
            quantity: self.quantity,
            margin: self.margin,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention audit in status new.
pub fn new_retention_audit(code: &str, label: &str) -> RetentionAudit {
    RetentionAudit {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        issuer: "".to_string(),
        attributes: BTreeMap::new(),
        audience: "".to_string(),
        ends_at: 0,
        archived: false,
        quantity: 0,
        margin: 0.0,
        active: false,
        status: RetentionAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_audit("code-784", "label-591");
        assert_eq!(item.status, RetentionAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_retention_audit("code-784", "label-591");
        if item.move_to(RetentionAuditStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionAuditStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_audit("code-784", "label-591");
        item.status = RetentionAuditStatus::Failed;
        assert!(
            item.move_to(RetentionAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

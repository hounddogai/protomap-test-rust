use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FlowLogAuditStatus {
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

impl std::fmt::Display for FlowLogAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAudit {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub floor_value: f64,
    pub budget: f64,
    pub external_ref: String,
    pub active: bool,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub category: String,
    pub status: FlowLogAuditStatus,
}

impl FlowLogAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogAuditStatus) -> bool {
        match self.status {
            FlowLogAuditStatus::New => {
                return next == FlowLogAuditStatus::Failed;
            }
            FlowLogAuditStatus::Queued => {
                return next == FlowLogAuditStatus::Running;
            }
            FlowLogAuditStatus::Running => {
                match next {
                    FlowLogAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogAuditStatus::Done => {
                match next {
                    FlowLogAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: FlowLogAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn weighted_exposure(&self) -> f64 {
        (self.floor_value * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogAudit {
        FlowLogAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            floor_value: self.floor_value,
            budget: self.budget,
            external_ref: self.external_ref.clone(),
            active: self.active,
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a flow log audit in status new.
pub fn new_flow_log_audit(code: &str, name: &str) -> FlowLogAudit {
    FlowLogAudit {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        expires_at: 0,
        floor_value: 0.0,
        budget: 0.0,
        external_ref: "".to_string(),
        active: false,
        attributes: BTreeMap::new(),
        threshold: 0.0,
        category: "".to_string(),
        status: FlowLogAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_flow_log_audit("code-220", "name-787");
        assert_eq!(item.status, FlowLogAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_flow_log_audit("code-220", "name-787");
        if item.move_to(FlowLogAuditStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogAuditStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_flow_log_audit("code-220", "name-787");
        item.status = FlowLogAuditStatus::Failed;
        assert!(item.move_to(FlowLogAuditStatus::New).is_err(), "expected move to fail");
    }
}

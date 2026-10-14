use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAudit {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub threshold: f64,
    pub limit: i64,
    pub window_days: i64,
    pub score: f64,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub principal: String,
    pub source: String,
    pub status: IsolationAuditStatus,
}

impl IsolationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationAuditStatus) -> bool {
        match self.status {
            IsolationAuditStatus::Created => {
                return next == IsolationAuditStatus::Validated || next == IsolationAuditStatus::Published;
            }
            IsolationAuditStatus::Validated => {
                match next {
                    IsolationAuditStatus::Published | IsolationAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationAuditStatus::Published => {
                return next == IsolationAuditStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn effective_value(&self) -> f64 {
        ((self.score + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationAudit {
        IsolationAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            threshold: self.threshold,
            limit: self.limit,
            window_days: self.window_days,
            score: self.score,
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a isolation audit in status created.
pub fn new_isolation_audit(code: &str, name: &str) -> IsolationAudit {
    IsolationAudit {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        threshold: 0.0,
        limit: 0,
        window_days: 0,
        score: 0.0,
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        status: IsolationAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_audit("code-924", "name-400");
        assert_eq!(item.status, IsolationAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_isolation_audit("code-924", "name-400");
        if item.move_to(IsolationAuditStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationAuditStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_audit("code-924", "name-400");
        item.status = IsolationAuditStatus::Retired;
        assert!(
            item.move_to(IsolationAuditStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

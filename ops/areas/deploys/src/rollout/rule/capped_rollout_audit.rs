use super::*;

use acme_kit::text;

/// Where a capped rollout audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRolloutAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedRolloutAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedRolloutAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped rollout audit as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAudit {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub tags: Vec<String>,
    pub audience: String,
    pub status: CappedRolloutAuditStatus,
}

impl CappedRolloutAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRolloutAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRolloutAuditStatus) -> bool {
        match self.status {
            CappedRolloutAuditStatus::Created => {
                match next {
                    CappedRolloutAuditStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRolloutAuditStatus::Validated => {
                match next {
                    CappedRolloutAuditStatus::Published | CappedRolloutAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRolloutAuditStatus::Published => {
                match next {
                    CappedRolloutAuditStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedRolloutAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped rollout audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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
        self.port as f64
    }

    pub fn copy(&self) -> CappedRolloutAudit {
        CappedRolloutAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped rollout audit in status created.
pub fn new_capped_rollout_audit(code: &str, name: &str) -> CappedRolloutAudit {
    CappedRolloutAudit {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        tags: Vec::new(),
        audience: "".to_string(),
        status: CappedRolloutAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_rollout_audit("code-923", "name-794");
        assert_eq!(item.status, CappedRolloutAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_rollout_audit("code-923", "name-794");
        if item.move_to(CappedRolloutAuditStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRolloutAuditStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_rollout_audit("code-923", "name-794");
        item.status = CappedRolloutAuditStatus::Retired;
        assert!(
            item.move_to(CappedRolloutAuditStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

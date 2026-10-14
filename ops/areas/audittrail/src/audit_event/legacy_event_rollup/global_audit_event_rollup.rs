use super::*;

use acme_kit::text;

/// Where a global audit event rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAuditEventRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalAuditEventRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalAuditEventRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global audit event rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollup {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub tags: Vec<String>,
    pub scope: String,
    pub status: GlobalAuditEventRollupStatus,
}

impl GlobalAuditEventRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAuditEventRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAuditEventRollupStatus) -> bool {
        match self.status {
            GlobalAuditEventRollupStatus::Created => {
                match next {
                    GlobalAuditEventRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventRollupStatus::Validated => {
                match next {
                    GlobalAuditEventRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventRollupStatus::Published => {
                return next == GlobalAuditEventRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalAuditEventRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global audit event rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.margin
    }

    pub fn copy(&self) -> GlobalAuditEventRollup {
        GlobalAuditEventRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            tags: self.tags.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global audit event rollup in status created.
pub fn new_global_audit_event_rollup(code: &str, label: &str) -> GlobalAuditEventRollup {
    GlobalAuditEventRollup {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        tags: Vec::new(),
        scope: "".to_string(),
        status: GlobalAuditEventRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_audit_event_rollup("code-513", "label-193");
        assert_eq!(item.status, GlobalAuditEventRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_global_audit_event_rollup("code-513", "label-193");
        if item.move_to(GlobalAuditEventRollupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAuditEventRollupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_audit_event_rollup("code-513", "label-193");
        item.status = GlobalAuditEventRollupStatus::Retired;
        assert!(
            item.move_to(GlobalAuditEventRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

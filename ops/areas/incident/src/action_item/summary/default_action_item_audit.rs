use super::*;

use acme_kit::text;

/// Where a default action item audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultActionItemAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultActionItemAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default action item audit as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAudit {
    pub code: String,
    pub name: String,
    pub description: String,
    pub threshold: f64,
    pub ends_at: i64,
    pub rate: f64,
    pub retries: i64,
    pub batch_size: i64,
    pub status: DefaultActionItemAuditStatus,
}

impl DefaultActionItemAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemAuditStatus) -> bool {
        match self.status {
            DefaultActionItemAuditStatus::Created => {
                match next {
                    DefaultActionItemAuditStatus::Published | DefaultActionItemAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemAuditStatus::Validated => {
                match next {
                    DefaultActionItemAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemAuditStatus::Published => {
                return next == DefaultActionItemAuditStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultActionItemAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.rate * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultActionItemAudit {
        DefaultActionItemAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            threshold: self.threshold,
            ends_at: self.ends_at,
            rate: self.rate,
            retries: self.retries,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default action item audit in status created.
pub fn new_default_action_item_audit(code: &str, name: &str) -> DefaultActionItemAudit {
    DefaultActionItemAudit {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        threshold: 0.0,
        ends_at: 0,
        rate: 0.0,
        retries: 0,
        batch_size: 0,
        status: DefaultActionItemAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_action_item_audit("code-523", "name-90");
        assert_eq!(item.status, DefaultActionItemAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_default_action_item_audit("code-523", "name-90");
        if item.move_to(DefaultActionItemAuditStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultActionItemAuditStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_action_item_audit("code-523", "name-90");
        item.status = DefaultActionItemAuditStatus::Retired;
        assert!(
            item.move_to(DefaultActionItemAuditStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

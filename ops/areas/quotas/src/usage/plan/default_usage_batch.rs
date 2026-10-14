use super::*;

use acme_kit::text;

/// Where a default usage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultUsageBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultUsageBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultUsageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default usage batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageBatch {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub title: String,
    pub scope: String,
    pub expires_at: i64,
    pub notes: String,
    pub status: DefaultUsageBatchStatus,
}

impl DefaultUsageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultUsageBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultUsageBatchStatus) -> bool {
        match self.status {
            DefaultUsageBatchStatus::Created => {
                match next {
                    DefaultUsageBatchStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageBatchStatus::Validated => {
                return next == DefaultUsageBatchStatus::Retired;
            }
            DefaultUsageBatchStatus::Published => {
                return next == DefaultUsageBatchStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultUsageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default usage batch {} cannot move from {} to {}",
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
        format!("{} {}", "default usage batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> DefaultUsageBatch {
        DefaultUsageBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            expires_at: self.expires_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default usage batch in status created.
pub fn new_default_usage_batch(code: &str, name: &str) -> DefaultUsageBatch {
    DefaultUsageBatch {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        title: "".to_string(),
        scope: "".to_string(),
        expires_at: 0,
        notes: "".to_string(),
        status: DefaultUsageBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_usage_batch("code-412", "name-406");
        assert_eq!(item.status, DefaultUsageBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_default_usage_batch("code-412", "name-406");
        if item.move_to(DefaultUsageBatchStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultUsageBatchStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_usage_batch("code-412", "name-406");
        item.status = DefaultUsageBatchStatus::Retired;
        assert!(
            item.move_to(DefaultUsageBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

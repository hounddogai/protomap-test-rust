use super::*;

use acme_kit::text;

/// Where a default action item exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultActionItemExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultActionItemExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default action item exception as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemException {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub ends_at: i64,
    pub batch_size: i64,
    pub external_ref: String,
    pub category: String,
    pub pinned: bool,
    pub status: DefaultActionItemExceptionStatus,
}

impl DefaultActionItemException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemExceptionStatus) -> bool {
        match self.status {
            DefaultActionItemExceptionStatus::Created => {
                return next == DefaultActionItemExceptionStatus::Published
                    || next == DefaultActionItemExceptionStatus::Retired;
            }
            DefaultActionItemExceptionStatus::Validated => {
                return next == DefaultActionItemExceptionStatus::Published
                    || next == DefaultActionItemExceptionStatus::Retired;
            }
            DefaultActionItemExceptionStatus::Published => {
                return next == DefaultActionItemExceptionStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultActionItemExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item exception {} cannot move from {} to {}",
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
        format!("{} {}", "default action item exception", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultActionItemException {
        DefaultActionItemException {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default action item exception in status created.
pub fn new_default_action_item_exception(code: &str, name: &str) -> DefaultActionItemException {
    DefaultActionItemException {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        ends_at: 0,
        batch_size: 0,
        external_ref: "".to_string(),
        category: "".to_string(),
        pinned: false,
        status: DefaultActionItemExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_action_item_exception("code-623", "name-883");
        assert_eq!(item.status, DefaultActionItemExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_default_action_item_exception("code-623", "name-883");
        if item.move_to(DefaultActionItemExceptionStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultActionItemExceptionStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_action_item_exception("code-623", "name-883");
        item.status = DefaultActionItemExceptionStatus::Retired;
        assert!(
            item.move_to(DefaultActionItemExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

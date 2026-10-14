use super::*;

use acme_kit::text;

/// Where a usage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage batch as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatch {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub visible: bool,
    pub timezone: String,
    pub quantity: i64,
    pub status: UsageBatchStatus,
}

impl UsageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageBatchStatus) -> bool {
        match self.status {
            UsageBatchStatus::Proposed => {
                match next {
                    UsageBatchStatus::Approved | UsageBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageBatchStatus::Approved => {
                return next == UsageBatchStatus::Withdrawn;
            }
            UsageBatchStatus::Rejected => {
                return next == UsageBatchStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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
        self.quantity as f64
    }

    pub fn copy(&self) -> UsageBatch {
        UsageBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a usage batch in status proposed.
pub fn new_usage_batch(code: &str, name: &str) -> UsageBatch {
    UsageBatch {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        visible: false,
        timezone: "".to_string(),
        quantity: 0,
        status: UsageBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_batch("code-657", "name-820");
        assert_eq!(item.status, UsageBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_usage_batch("code-657", "name-820");
        if item.move_to(UsageBatchStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageBatchStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_batch("code-657", "name-820");
        item.status = UsageBatchStatus::Withdrawn;
        assert!(
            item.move_to(UsageBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

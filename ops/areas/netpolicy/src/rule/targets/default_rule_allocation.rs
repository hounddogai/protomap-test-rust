use super::*;

use acme_kit::text;

/// Where a default rule allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRuleAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRuleAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultRuleAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default rule allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleAllocation {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub weight: f64,
    pub starts_at: i64,
    pub sequence: i64,
    pub status: DefaultRuleAllocationStatus,
}

impl DefaultRuleAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRuleAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRuleAllocationStatus) -> bool {
        match self.status {
            DefaultRuleAllocationStatus::Open => {
                return next == DefaultRuleAllocationStatus::Resolved || next == DefaultRuleAllocationStatus::Archived;
            }
            DefaultRuleAllocationStatus::InReview => {
                match next {
                    DefaultRuleAllocationStatus::Reopened | DefaultRuleAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleAllocationStatus::Resolved => {
                return next == DefaultRuleAllocationStatus::Archived;
            }
            DefaultRuleAllocationStatus::Reopened => {
                return next == DefaultRuleAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRuleAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rule allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default rule allocation", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.weight * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRuleAllocation {
        DefaultRuleAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            weight: self.weight,
            starts_at: self.starts_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default rule allocation in status open.
pub fn new_default_rule_allocation(code: &str, title: &str) -> DefaultRuleAllocation {
    DefaultRuleAllocation {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        weight: 0.0,
        starts_at: 0,
        sequence: 0,
        status: DefaultRuleAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_rule_allocation("code-511", "title-254");
        assert_eq!(item.status, DefaultRuleAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_rule_allocation("code-511", "title-254");
        if item.move_to(DefaultRuleAllocationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRuleAllocationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_rule_allocation("code-511", "title-254");
        item.status = DefaultRuleAllocationStatus::Archived;
        assert!(
            item.move_to(DefaultRuleAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

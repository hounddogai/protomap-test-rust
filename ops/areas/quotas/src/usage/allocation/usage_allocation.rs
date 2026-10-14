use super::*;

use acme_kit::text;

/// Where a usage allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAllocation {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub sequence: i64,
    pub category: String,
    pub status: UsageAllocationStatus,
}

impl UsageAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageAllocationStatus) -> bool {
        match self.status {
            UsageAllocationStatus::Proposed => {
                match next {
                    UsageAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageAllocationStatus::Approved => {
                return next == UsageAllocationStatus::Rejected || next == UsageAllocationStatus::Withdrawn;
            }
            UsageAllocationStatus::Rejected => {
                return next == UsageAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> UsageAllocation {
        UsageAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a usage allocation in status proposed.
pub fn new_usage_allocation(code: &str, title: &str) -> UsageAllocation {
    UsageAllocation {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        sequence: 0,
        category: "".to_string(),
        status: UsageAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_allocation("code-641", "title-668");
        assert_eq!(item.status, UsageAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_usage_allocation("code-641", "title-668");
        if item.move_to(UsageAllocationStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageAllocationStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_allocation("code-641", "title-668");
        item.status = UsageAllocationStatus::Withdrawn;
        assert!(
            item.move_to(UsageAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

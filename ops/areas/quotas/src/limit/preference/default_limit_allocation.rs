use super::*;

use acme_kit::text;

/// Where a default limit allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultLimitAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultLimitAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultLimitAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default limit allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAllocation {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub locale: String,
    pub owner_id: String,
    pub source: String,
    pub quantity: i64,
    pub status: DefaultLimitAllocationStatus,
}

impl DefaultLimitAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultLimitAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultLimitAllocationStatus) -> bool {
        match self.status {
            DefaultLimitAllocationStatus::Planned => {
                match next {
                    DefaultLimitAllocationStatus::Scheduled | DefaultLimitAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultLimitAllocationStatus::Scheduled => {
                match next {
                    DefaultLimitAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultLimitAllocationStatus::InProgress => {
                return next == DefaultLimitAllocationStatus::Completed
                    || next == DefaultLimitAllocationStatus::Cancelled;
            }
            DefaultLimitAllocationStatus::Completed => {
                match next {
                    DefaultLimitAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultLimitAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default limit allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> DefaultLimitAllocation {
        DefaultLimitAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default limit allocation in status planned.
pub fn new_default_limit_allocation(code: &str, name: &str) -> DefaultLimitAllocation {
    DefaultLimitAllocation {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        locale: "".to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        status: DefaultLimitAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_limit_allocation("code-544", "name-172");
        assert_eq!(item.status, DefaultLimitAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_default_limit_allocation("code-544", "name-172");
        if item.move_to(DefaultLimitAllocationStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultLimitAllocationStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_limit_allocation("code-544", "name-172");
        item.status = DefaultLimitAllocationStatus::Cancelled;
        assert!(
            item.move_to(DefaultLimitAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

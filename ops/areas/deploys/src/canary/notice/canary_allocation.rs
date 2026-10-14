use super::*;

use acme_kit::text;

/// Where a canary allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanaryAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CanaryAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocation {
    pub code: String,
    pub label: String,
    pub title: String,
    pub retries: i64,
    pub limit: i64,
    pub status: CanaryAllocationStatus,
}

impl CanaryAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryAllocationStatus) -> bool {
        match self.status {
            CanaryAllocationStatus::Draft => {
                match next {
                    CanaryAllocationStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAllocationStatus::Pending => {
                return next == CanaryAllocationStatus::Suspended || next == CanaryAllocationStatus::Closed;
            }
            CanaryAllocationStatus::Active => {
                return next == CanaryAllocationStatus::Closed;
            }
            CanaryAllocationStatus::Suspended => {
                match next {
                    CanaryAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: CanaryAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "canary allocation", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryAllocation {
        CanaryAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            retries: self.retries,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a canary allocation in status draft.
pub fn new_canary_allocation(code: &str, label: &str) -> CanaryAllocation {
    CanaryAllocation {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        retries: 0,
        limit: 0,
        status: CanaryAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_allocation("code-886", "label-427");
        assert_eq!(item.status, CanaryAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_canary_allocation("code-886", "label-427");
        if item.move_to(CanaryAllocationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryAllocationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_allocation("code-886", "label-427");
        item.status = CanaryAllocationStatus::Closed;
        assert!(
            item.move_to(CanaryAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

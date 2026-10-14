use super::*;

use acme_kit::text;

/// Where a pending federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingFederationAllocationStatus {
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

impl std::fmt::Display for PendingFederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending federation allocation as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocation {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub cost: f64,
    pub threshold: f64,
    pub priority: i64,
    pub capacity: i64,
    pub status: PendingFederationAllocationStatus,
}

impl PendingFederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationAllocationStatus) -> bool {
        match self.status {
            PendingFederationAllocationStatus::Draft => {
                return next == PendingFederationAllocationStatus::Closed;
            }
            PendingFederationAllocationStatus::Pending => {
                return next == PendingFederationAllocationStatus::Active;
            }
            PendingFederationAllocationStatus::Active => {
                return next == PendingFederationAllocationStatus::Suspended
                    || next == PendingFederationAllocationStatus::Closed;
            }
            PendingFederationAllocationStatus::Suspended => {
                match next {
                    PendingFederationAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingFederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending federation allocation", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.priority as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationAllocation {
        PendingFederationAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            threshold: self.threshold,
            priority: self.priority,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending federation allocation in status draft.
pub fn new_pending_federation_allocation(code: &str, label: &str) -> PendingFederationAllocation {
    PendingFederationAllocation {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        cost: 0.0,
        threshold: 0.0,
        priority: 0,
        capacity: 0,
        status: PendingFederationAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_federation_allocation("code-831", "label-541");
        assert_eq!(item.status, PendingFederationAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_pending_federation_allocation("code-831", "label-541");
        if item.move_to(PendingFederationAllocationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFederationAllocationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_federation_allocation("code-831", "label-541");
        item.status = PendingFederationAllocationStatus::Closed;
        assert!(
            item.move_to(PendingFederationAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

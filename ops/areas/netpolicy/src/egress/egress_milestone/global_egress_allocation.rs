use super::*;

use acme_kit::text;

/// Where a global egress allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEgressAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalEgressAllocationStatus {
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

impl std::fmt::Display for GlobalEgressAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global egress allocation as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocation {
    pub code: String,
    pub name: String,
    pub host: String,
    pub contact_email: String,
    pub owner_id: String,
    pub created_at: i64,
    pub phone: String,
    pub summary: String,
    pub sequence: i64,
    pub country: String,
    pub status: GlobalEgressAllocationStatus,
}

impl GlobalEgressAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEgressAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEgressAllocationStatus) -> bool {
        match self.status {
            GlobalEgressAllocationStatus::Draft => {
                return next == GlobalEgressAllocationStatus::Pending
                    || next == GlobalEgressAllocationStatus::Suspended;
            }
            GlobalEgressAllocationStatus::Pending => {
                match next {
                    GlobalEgressAllocationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEgressAllocationStatus::Active => {
                return next == GlobalEgressAllocationStatus::Closed;
            }
            GlobalEgressAllocationStatus::Suspended => {
                return next == GlobalEgressAllocationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalEgressAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global egress allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.sequence as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEgressAllocation {
        GlobalEgressAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            sequence: self.sequence,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global egress allocation in status draft.
pub fn new_global_egress_allocation(code: &str, name: &str) -> GlobalEgressAllocation {
    GlobalEgressAllocation {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        created_at: 0,
        phone: "".to_string(),
        summary: "".to_string(),
        sequence: 0,
        country: "".to_string(),
        status: GlobalEgressAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_egress_allocation("code-371", "name-358");
        assert_eq!(item.status, GlobalEgressAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_egress_allocation("code-371", "name-358");
        if item.move_to(GlobalEgressAllocationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEgressAllocationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_egress_allocation("code-371", "name-358");
        item.status = GlobalEgressAllocationStatus::Closed;
        assert!(
            item.move_to(GlobalEgressAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

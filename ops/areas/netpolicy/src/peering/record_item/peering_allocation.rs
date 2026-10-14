use super::*;

use acme_kit::text;

/// Where a peering allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocation {
    pub code: String,
    pub name: String,
    pub label: String,
    pub reviewer: String,
    pub updated_at: i64,
    pub batch_size: i64,
    pub notes: String,
    pub status: PeeringAllocationStatus,
}

impl PeeringAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringAllocationStatus) -> bool {
        match self.status {
            PeeringAllocationStatus::Requested => {
                return next == PeeringAllocationStatus::Granted || next == PeeringAllocationStatus::Expired;
            }
            PeeringAllocationStatus::Granted => {
                match next {
                    PeeringAllocationStatus::Revoked | PeeringAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringAllocationStatus::Revoked => {
                return next == PeeringAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringAllocation {
        PeeringAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            batch_size: self.batch_size,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering allocation in status requested.
pub fn new_peering_allocation(code: &str, name: &str) -> PeeringAllocation {
    PeeringAllocation {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        reviewer: "".to_string(),
        updated_at: 0,
        batch_size: 0,
        notes: "".to_string(),
        status: PeeringAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_peering_allocation("code-43", "name-632");
        assert_eq!(item.status, PeeringAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_peering_allocation("code-43", "name-632");
        if item.move_to(PeeringAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_peering_allocation("code-43", "name-632");
        item.status = PeeringAllocationStatus::Expired;
        assert!(
            item.move_to(PeeringAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

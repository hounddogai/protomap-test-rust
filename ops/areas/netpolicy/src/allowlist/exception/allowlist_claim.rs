use super::*;

use acme_kit::text;

/// Where a allowlist claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AllowlistClaimStatus {
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

impl std::fmt::Display for AllowlistClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaim {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub capacity: i64,
    pub amount: f64,
    pub ceiling: f64,
    pub cluster: String,
    pub region: String,
    pub notes: String,
    pub quantity: i64,
    pub status: AllowlistClaimStatus,
}

impl AllowlistClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistClaimStatus) -> bool {
        match self.status {
            AllowlistClaimStatus::Planned => {
                match next {
                    AllowlistClaimStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistClaimStatus::Scheduled => {
                match next {
                    AllowlistClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistClaimStatus::InProgress => {
                match next {
                    AllowlistClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistClaimStatus::Completed => {
                return next == AllowlistClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist claim", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistClaim {
        AllowlistClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            amount: self.amount,
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a allowlist claim in status planned.
pub fn new_allowlist_claim(code: &str, label: &str) -> AllowlistClaim {
    AllowlistClaim {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        capacity: 0,
        amount: 0.0,
        ceiling: 0.0,
        cluster: "".to_string(),
        region: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: AllowlistClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_allowlist_claim("code-877", "label-330");
        assert_eq!(item.status, AllowlistClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_allowlist_claim("code-877", "label-330");
        if item.move_to(AllowlistClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_allowlist_claim("code-877", "label-330");
        item.status = AllowlistClaimStatus::Cancelled;
        assert!(
            item.move_to(AllowlistClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

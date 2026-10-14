use super::*;

use acme_kit::text;

/// Where a strict approval adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictApprovalAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictApprovalAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict approval adjustment as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustment {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub amount: f64,
    pub grace_days: i64,
    pub visible: bool,
    pub created_at: i64,
    pub country: String,
    pub audience: String,
    pub status: StrictApprovalAdjustmentStatus,
}

impl StrictApprovalAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalAdjustmentStatus) -> bool {
        match self.status {
            StrictApprovalAdjustmentStatus::Created => {
                match next {
                    StrictApprovalAdjustmentStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalAdjustmentStatus::Validated => {
                match next {
                    StrictApprovalAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalAdjustmentStatus::Published => {
                match next {
                    StrictApprovalAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictApprovalAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval adjustment {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.amount - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictApprovalAdjustment {
        StrictApprovalAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            amount: self.amount,
            grace_days: self.grace_days,
            visible: self.visible,
            created_at: self.created_at,
            country: self.country.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict approval adjustment in status created.
pub fn new_strict_approval_adjustment(code: &str, name: &str) -> StrictApprovalAdjustment {
    StrictApprovalAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        amount: 0.0,
        grace_days: 0,
        visible: false,
        created_at: 0,
        country: "".to_string(),
        audience: "".to_string(),
        status: StrictApprovalAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_approval_adjustment("code-725", "name-535");
        assert_eq!(item.status, StrictApprovalAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_approval_adjustment("code-725", "name-535");
        if item.move_to(StrictApprovalAdjustmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalAdjustmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_approval_adjustment("code-725", "name-535");
        item.status = StrictApprovalAdjustmentStatus::Retired;
        assert!(
            item.move_to(StrictApprovalAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

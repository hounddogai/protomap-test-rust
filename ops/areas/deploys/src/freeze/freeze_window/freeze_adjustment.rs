use super::*;

use acme_kit::text;

/// Where a freeze adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustment {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub port: i64,
    pub principal: String,
    pub name: String,
    pub status: FreezeAdjustmentStatus,
}

impl FreezeAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeAdjustmentStatus) -> bool {
        match self.status {
            FreezeAdjustmentStatus::Proposed => {
                match next {
                    FreezeAdjustmentStatus::Approved | FreezeAdjustmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeAdjustmentStatus::Approved => {
                return next == FreezeAdjustmentStatus::Rejected || next == FreezeAdjustmentStatus::Withdrawn;
            }
            FreezeAdjustmentStatus::Rejected => {
                match next {
                    FreezeAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FreezeAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeAdjustment {
        FreezeAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            port: self.port,
            principal: self.principal.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze adjustment in status proposed.
pub fn new_freeze_adjustment(code: &str, title: &str) -> FreezeAdjustment {
    FreezeAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        port: 0,
        principal: "".to_string(),
        name: "".to_string(),
        status: FreezeAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_adjustment("code-612", "title-71");
        assert_eq!(item.status, FreezeAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_freeze_adjustment("code-612", "title-71");
        if item.move_to(FreezeAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_adjustment("code-612", "title-71");
        item.status = FreezeAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(FreezeAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

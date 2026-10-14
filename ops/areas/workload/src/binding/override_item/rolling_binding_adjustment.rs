use super::*;

use acme_kit::text;

/// Where a rolling binding adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBindingAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingBindingAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingBindingAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling binding adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustment {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub locale: String,
    pub retries: i64,
    pub name: String,
    pub status: RollingBindingAdjustmentStatus,
}

impl RollingBindingAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBindingAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBindingAdjustmentStatus) -> bool {
        match self.status {
            RollingBindingAdjustmentStatus::Proposed => {
                match next {
                    RollingBindingAdjustmentStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBindingAdjustmentStatus::Approved => {
                match next {
                    RollingBindingAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBindingAdjustmentStatus::Rejected => {
                match next {
                    RollingBindingAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingBindingAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling binding adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling binding adjustment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> RollingBindingAdjustment {
        RollingBindingAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            retries: self.retries,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling binding adjustment in status proposed.
pub fn new_rolling_binding_adjustment(code: &str, title: &str) -> RollingBindingAdjustment {
    RollingBindingAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        locale: "".to_string(),
        retries: 0,
        name: "".to_string(),
        status: RollingBindingAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_binding_adjustment("code-621", "title-777");
        assert_eq!(item.status, RollingBindingAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_binding_adjustment("code-621", "title-777");
        if item.move_to(RollingBindingAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBindingAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_binding_adjustment("code-621", "title-777");
        item.status = RollingBindingAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(RollingBindingAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a overage window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl OverageWindowStatus {
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

impl std::fmt::Display for OverageWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindow {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub locale: String,
    pub cost: f64,
    pub status: OverageWindowStatus,
}

impl OverageWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageWindowStatus) -> bool {
        match self.status {
            OverageWindowStatus::Draft => {
                match next {
                    OverageWindowStatus::Pending | OverageWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageWindowStatus::Pending => {
                return next == OverageWindowStatus::Suspended || next == OverageWindowStatus::Closed;
            }
            OverageWindowStatus::Active => {
                match next {
                    OverageWindowStatus::Suspended | OverageWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageWindowStatus::Suspended => {
                return next == OverageWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.weight * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageWindow {
        OverageWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            locale: self.locale.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage window in status draft.
pub fn new_overage_window(code: &str, label: &str) -> OverageWindow {
    OverageWindow {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        locale: "".to_string(),
        cost: 0.0,
        status: OverageWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_overage_window("code-426", "label-102");
        assert_eq!(item.status, OverageWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_overage_window("code-426", "label-102");
        if item.move_to(OverageWindowStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageWindowStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_overage_window("code-426", "label-102");
        item.status = OverageWindowStatus::Closed;
        assert!(
            item.move_to(OverageWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a regional token slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTokenSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalTokenSlotStatus {
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

impl std::fmt::Display for RegionalTokenSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional token slot as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlot {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub weight: f64,
    pub description: String,
    pub status: RegionalTokenSlotStatus,
}

impl RegionalTokenSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTokenSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTokenSlotStatus) -> bool {
        match self.status {
            RegionalTokenSlotStatus::Draft => {
                return next == RegionalTokenSlotStatus::Pending;
            }
            RegionalTokenSlotStatus::Pending => {
                return next == RegionalTokenSlotStatus::Suspended || next == RegionalTokenSlotStatus::Closed;
            }
            RegionalTokenSlotStatus::Active => {
                match next {
                    RegionalTokenSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTokenSlotStatus::Suspended => {
                return next == RegionalTokenSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalTokenSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional token slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.weight * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTokenSlot {
        RegionalTokenSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            weight: self.weight,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional token slot in status draft.
pub fn new_regional_token_slot(code: &str, label: &str) -> RegionalTokenSlot {
    RegionalTokenSlot {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        weight: 0.0,
        description: "".to_string(),
        status: RegionalTokenSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_token_slot("code-985", "label-984");
        assert_eq!(item.status, RegionalTokenSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_regional_token_slot("code-985", "label-984");
        if item.move_to(RegionalTokenSlotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTokenSlotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_token_slot("code-985", "label-984");
        item.status = RegionalTokenSlotStatus::Closed;
        assert!(
            item.move_to(RegionalTokenSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

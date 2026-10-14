use super::*;

use acme_kit::text;

/// Where a global page slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPageSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalPageSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalPageSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global page slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlot {
    pub code: String,
    pub title: String,
    pub label: String,
    pub name: String,
    pub principal: String,
    pub description: String,
    pub window_days: i64,
    pub status: GlobalPageSlotStatus,
}

impl GlobalPageSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPageSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPageSlotStatus) -> bool {
        match self.status {
            GlobalPageSlotStatus::Proposed => {
                return next == GlobalPageSlotStatus::Approved;
            }
            GlobalPageSlotStatus::Approved => {
                match next {
                    GlobalPageSlotStatus::Rejected | GlobalPageSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPageSlotStatus::Rejected => {
                return next == GlobalPageSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPageSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global page slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> GlobalPageSlot {
        GlobalPageSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            description: self.description.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global page slot in status proposed.
pub fn new_global_page_slot(code: &str, title: &str) -> GlobalPageSlot {
    GlobalPageSlot {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        name: "".to_string(),
        principal: "".to_string(),
        description: "".to_string(),
        window_days: 0,
        status: GlobalPageSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_page_slot("code-85", "title-194");
        assert_eq!(item.status, GlobalPageSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_global_page_slot("code-85", "title-194");
        if item.move_to(GlobalPageSlotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPageSlotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_page_slot("code-85", "title-194");
        item.status = GlobalPageSlotStatus::Withdrawn;
        assert!(
            item.move_to(GlobalPageSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a page slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PageSlotStatus {
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

impl std::fmt::Display for PageSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSlot {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub quantity: i64,
    pub weight: f64,
    pub issuer: String,
    pub region: String,
    pub unit: String,
    pub retries: i64,
    pub principal: String,
    pub status: PageSlotStatus,
}

impl PageSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageSlotStatus) -> bool {
        match self.status {
            PageSlotStatus::Planned => {
                return next == PageSlotStatus::Completed || next == PageSlotStatus::Cancelled;
            }
            PageSlotStatus::Scheduled => {
                return next == PageSlotStatus::InProgress || next == PageSlotStatus::Completed;
            }
            PageSlotStatus::InProgress => {
                return next == PageSlotStatus::Completed || next == PageSlotStatus::Cancelled;
            }
            PageSlotStatus::Completed => {
                return next == PageSlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "page slot", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageSlot {
        PageSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            quantity: self.quantity,
            weight: self.weight,
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            unit: self.unit.clone(),
            retries: self.retries,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a page slot in status planned.
pub fn new_page_slot(code: &str, label: &str) -> PageSlot {
    PageSlot {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        quantity: 0,
        weight: 0.0,
        issuer: "".to_string(),
        region: "".to_string(),
        unit: "".to_string(),
        retries: 0,
        principal: "".to_string(),
        status: PageSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_slot("code-3", "label-255");
        assert_eq!(item.status, PageSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_page_slot("code-3", "label-255");
        if item.move_to(PageSlotStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageSlotStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_slot("code-3", "label-255");
        item.status = PageSlotStatus::Cancelled;
        assert!(item.move_to(PageSlotStatus::Planned).is_err(), "expected move to fail");
    }
}

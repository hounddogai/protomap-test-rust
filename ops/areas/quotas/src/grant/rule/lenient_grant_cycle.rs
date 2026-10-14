use super::*;

use acme_kit::text;

/// Where a lenient grant cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientGrantCycleStatus {
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

impl std::fmt::Display for LenientGrantCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient grant cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantCycle {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub archived: bool,
    pub quota_units: i64,
    pub visible: bool,
    pub principal: String,
    pub discount: f64,
    pub label: String,
    pub cipher: String,
    pub capacity: i64,
    pub status: LenientGrantCycleStatus,
}

impl LenientGrantCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantCycleStatus) -> bool {
        match self.status {
            LenientGrantCycleStatus::Planned => {
                return next == LenientGrantCycleStatus::Completed;
            }
            LenientGrantCycleStatus::Scheduled => {
                return next == LenientGrantCycleStatus::Cancelled;
            }
            LenientGrantCycleStatus::InProgress => {
                return next == LenientGrantCycleStatus::Completed || next == LenientGrantCycleStatus::Cancelled;
            }
            LenientGrantCycleStatus::Completed => {
                return next == LenientGrantCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientGrantCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.discount * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantCycle {
        LenientGrantCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            archived: self.archived,
            quota_units: self.quota_units,
            visible: self.visible,
            principal: self.principal.clone(),
            discount: self.discount,
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient grant cycle in status planned.
pub fn new_lenient_grant_cycle(code: &str, title: &str) -> LenientGrantCycle {
    LenientGrantCycle {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        archived: false,
        quota_units: 0,
        visible: false,
        principal: "".to_string(),
        discount: 0.0,
        label: "".to_string(),
        cipher: "".to_string(),
        capacity: 0,
        status: LenientGrantCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_grant_cycle("code-495", "title-513");
        assert_eq!(item.status, LenientGrantCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_grant_cycle("code-495", "title-513");
        if item.move_to(LenientGrantCycleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientGrantCycleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_grant_cycle("code-495", "title-513");
        item.status = LenientGrantCycleStatus::Cancelled;
        assert!(
            item.move_to(LenientGrantCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

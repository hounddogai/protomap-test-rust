use super::*;

use acme_kit::text;

/// Where a capped postmortem cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPostmortemCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedPostmortemCycleStatus {
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

impl std::fmt::Display for CappedPostmortemCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped postmortem cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCycle {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub source: String,
    pub duration_minutes: i64,
    pub pinned: bool,
    pub notes: String,
    pub floor_value: f64,
    pub budget: f64,
    pub status: CappedPostmortemCycleStatus,
}

impl CappedPostmortemCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPostmortemCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPostmortemCycleStatus) -> bool {
        match self.status {
            CappedPostmortemCycleStatus::Draft => {
                return next == CappedPostmortemCycleStatus::Active;
            }
            CappedPostmortemCycleStatus::Pending => {
                return next == CappedPostmortemCycleStatus::Suspended;
            }
            CappedPostmortemCycleStatus::Active => {
                return next == CappedPostmortemCycleStatus::Closed;
            }
            CappedPostmortemCycleStatus::Suspended => {
                return next == CappedPostmortemCycleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPostmortemCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped postmortem cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.floor_value - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPostmortemCycle {
        CappedPostmortemCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            source: self.source.clone(),
            duration_minutes: self.duration_minutes,
            pinned: self.pinned,
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped postmortem cycle in status draft.
pub fn new_capped_postmortem_cycle(code: &str, label: &str) -> CappedPostmortemCycle {
    CappedPostmortemCycle {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        source: "".to_string(),
        duration_minutes: 0,
        pinned: false,
        notes: "".to_string(),
        floor_value: 0.0,
        budget: 0.0,
        status: CappedPostmortemCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_postmortem_cycle("code-947", "label-307");
        assert_eq!(item.status, CappedPostmortemCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_postmortem_cycle("code-947", "label-307");
        if item.move_to(CappedPostmortemCycleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPostmortemCycleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_postmortem_cycle("code-947", "label-307");
        item.status = CappedPostmortemCycleStatus::Closed;
        assert!(
            item.move_to(CappedPostmortemCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

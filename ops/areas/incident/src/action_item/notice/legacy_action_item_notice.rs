use super::*;

use acme_kit::text;

/// Where a legacy action item notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyActionItemNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyActionItemNoticeStatus {
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

impl std::fmt::Display for LegacyActionItemNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy action item notice as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemNotice {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub phone: String,
    pub port: i64,
    pub batch_size: i64,
    pub status: LegacyActionItemNoticeStatus,
}

impl LegacyActionItemNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyActionItemNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyActionItemNoticeStatus) -> bool {
        match self.status {
            LegacyActionItemNoticeStatus::Planned => {
                return next == LegacyActionItemNoticeStatus::Scheduled
                    || next == LegacyActionItemNoticeStatus::InProgress;
            }
            LegacyActionItemNoticeStatus::Scheduled => {
                match next {
                    LegacyActionItemNoticeStatus::InProgress | LegacyActionItemNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyActionItemNoticeStatus::InProgress => {
                match next {
                    LegacyActionItemNoticeStatus::Completed | LegacyActionItemNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyActionItemNoticeStatus::Completed => {
                return next == LegacyActionItemNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyActionItemNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy action item notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.margin * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyActionItemNotice {
        LegacyActionItemNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            phone: self.phone.clone(),
            port: self.port,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy action item notice in status planned.
pub fn new_legacy_action_item_notice(code: &str, name: &str) -> LegacyActionItemNotice {
    LegacyActionItemNotice {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        phone: "".to_string(),
        port: 0,
        batch_size: 0,
        status: LegacyActionItemNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_action_item_notice("code-985", "name-12");
        assert_eq!(item.status, LegacyActionItemNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_action_item_notice("code-985", "name-12");
        if item.move_to(LegacyActionItemNoticeStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyActionItemNoticeStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_action_item_notice("code-985", "name-12");
        item.status = LegacyActionItemNoticeStatus::Cancelled;
        assert!(
            item.move_to(LegacyActionItemNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

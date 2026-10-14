use super::*;

use acme_kit::text;

/// Where a legacy retention window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRetentionWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyRetentionWindowStatus {
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

impl std::fmt::Display for LegacyRetentionWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy retention window as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionWindow {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub version: i64,
    pub segment: String,
    pub status: LegacyRetentionWindowStatus,
}

impl LegacyRetentionWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRetentionWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRetentionWindowStatus) -> bool {
        match self.status {
            LegacyRetentionWindowStatus::Planned => {
                return next == LegacyRetentionWindowStatus::Cancelled;
            }
            LegacyRetentionWindowStatus::Scheduled => {
                return next == LegacyRetentionWindowStatus::InProgress
                    || next == LegacyRetentionWindowStatus::Completed;
            }
            LegacyRetentionWindowStatus::InProgress => {
                return next == LegacyRetentionWindowStatus::Cancelled;
            }
            LegacyRetentionWindowStatus::Completed => {
                match next {
                    LegacyRetentionWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyRetentionWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy retention window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRetentionWindow {
        LegacyRetentionWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            version: self.version,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy retention window in status planned.
pub fn new_legacy_retention_window(code: &str, label: &str) -> LegacyRetentionWindow {
    LegacyRetentionWindow {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        version: 0,
        segment: "".to_string(),
        status: LegacyRetentionWindowStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_retention_window("code-397", "label-968");
        assert_eq!(item.status, LegacyRetentionWindowStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_legacy_retention_window("code-397", "label-968");
        if item.move_to(LegacyRetentionWindowStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRetentionWindowStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_retention_window("code-397", "label-968");
        item.status = LegacyRetentionWindowStatus::Cancelled;
        assert!(
            item.move_to(LegacyRetentionWindowStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

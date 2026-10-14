use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy freeze exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFreezeExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyFreezeExceptionStatus {
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

impl std::fmt::Display for LegacyFreezeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy freeze exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeException {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub notes: String,
    pub attributes: BTreeMap<String, String>,
    pub host: String,
    pub external_ref: String,
    pub name: String,
    pub pinned: bool,
    pub status: LegacyFreezeExceptionStatus,
}

impl LegacyFreezeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFreezeExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFreezeExceptionStatus) -> bool {
        match self.status {
            LegacyFreezeExceptionStatus::Planned => {
                return next == LegacyFreezeExceptionStatus::Scheduled;
            }
            LegacyFreezeExceptionStatus::Scheduled => {
                return next == LegacyFreezeExceptionStatus::InProgress
                    || next == LegacyFreezeExceptionStatus::Completed;
            }
            LegacyFreezeExceptionStatus::InProgress => {
                match next {
                    LegacyFreezeExceptionStatus::Completed | LegacyFreezeExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFreezeExceptionStatus::Completed => {
                return next == LegacyFreezeExceptionStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFreezeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy freeze exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> LegacyFreezeException {
        LegacyFreezeException {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            notes: self.notes.clone(),
            attributes: self.attributes.clone(),
            host: self.host.clone(),
            external_ref: self.external_ref.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy freeze exception in status planned.
pub fn new_legacy_freeze_exception(code: &str, label: &str) -> LegacyFreezeException {
    LegacyFreezeException {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        notes: "".to_string(),
        attributes: BTreeMap::new(),
        host: "".to_string(),
        external_ref: "".to_string(),
        name: "".to_string(),
        pinned: false,
        status: LegacyFreezeExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_freeze_exception("code-869", "label-705");
        assert_eq!(item.status, LegacyFreezeExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_freeze_exception("code-869", "label-705");
        if item.move_to(LegacyFreezeExceptionStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFreezeExceptionStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_freeze_exception("code-869", "label-705");
        item.status = LegacyFreezeExceptionStatus::Cancelled;
        assert!(
            item.move_to(LegacyFreezeExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

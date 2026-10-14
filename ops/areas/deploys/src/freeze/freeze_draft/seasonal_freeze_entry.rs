use super::*;

use acme_kit::text;

/// Where a seasonal freeze entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalFreezeEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalFreezeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal freeze entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeEntry {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub summary: String,
    pub grace_days: i64,
    pub issuer: String,
    pub ratio: f64,
    pub status: SeasonalFreezeEntryStatus,
}

impl SeasonalFreezeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeEntryStatus) -> bool {
        match self.status {
            SeasonalFreezeEntryStatus::Proposed => {
                return next == SeasonalFreezeEntryStatus::Approved;
            }
            SeasonalFreezeEntryStatus::Approved => {
                return next == SeasonalFreezeEntryStatus::Rejected || next == SeasonalFreezeEntryStatus::Withdrawn;
            }
            SeasonalFreezeEntryStatus::Rejected => {
                match next {
                    SeasonalFreezeEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalFreezeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ratio + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFreezeEntry {
        SeasonalFreezeEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            issuer: self.issuer.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal freeze entry in status proposed.
pub fn new_seasonal_freeze_entry(code: &str, name: &str) -> SeasonalFreezeEntry {
    SeasonalFreezeEntry {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        summary: "".to_string(),
        grace_days: 0,
        issuer: "".to_string(),
        ratio: 0.0,
        status: SeasonalFreezeEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_freeze_entry("code-161", "name-876");
        assert_eq!(item.status, SeasonalFreezeEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_freeze_entry("code-161", "name-876");
        if item.move_to(SeasonalFreezeEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_freeze_entry("code-161", "name-876");
        item.status = SeasonalFreezeEntryStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalFreezeEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

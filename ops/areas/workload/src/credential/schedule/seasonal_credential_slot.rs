use super::*;

use acme_kit::text;

/// Where a seasonal credential slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCredentialSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalCredentialSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalCredentialSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal credential slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSlot {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub limit: i64,
    pub minimum: f64,
    pub cost: f64,
    pub status: SeasonalCredentialSlotStatus,
}

impl SeasonalCredentialSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCredentialSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCredentialSlotStatus) -> bool {
        match self.status {
            SeasonalCredentialSlotStatus::Proposed => {
                return next == SeasonalCredentialSlotStatus::Rejected;
            }
            SeasonalCredentialSlotStatus::Approved => {
                match next {
                    SeasonalCredentialSlotStatus::Rejected | SeasonalCredentialSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialSlotStatus::Rejected => {
                match next {
                    SeasonalCredentialSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalCredentialSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal credential slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.window_days as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCredentialSlot {
        SeasonalCredentialSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            limit: self.limit,
            minimum: self.minimum,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal credential slot in status proposed.
pub fn new_seasonal_credential_slot(code: &str, title: &str) -> SeasonalCredentialSlot {
    SeasonalCredentialSlot {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        limit: 0,
        minimum: 0.0,
        cost: 0.0,
        status: SeasonalCredentialSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_credential_slot("code-679", "title-906");
        assert_eq!(item.status, SeasonalCredentialSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_credential_slot("code-679", "title-906");
        if item.move_to(SeasonalCredentialSlotStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCredentialSlotStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_credential_slot("code-679", "title-906");
        item.status = SeasonalCredentialSlotStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalCredentialSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

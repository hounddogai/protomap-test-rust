use super::*;

use acme_kit::text;

/// Where a quote slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QuoteSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QuoteSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSlot {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub pinned: bool,
    pub locale: String,
    pub burst: i64,
    pub amount: f64,
    pub archived: bool,
    pub rotation_days: i64,
    pub ratio: f64,
    pub status: QuoteSlotStatus,
}

impl QuoteSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteSlotStatus) -> bool {
        match self.status {
            QuoteSlotStatus::Proposed => {
                match next {
                    QuoteSlotStatus::Approved | QuoteSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSlotStatus::Approved => {
                match next {
                    QuoteSlotStatus::Rejected | QuoteSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSlotStatus::Rejected => {
                return next == QuoteSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote slot", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.amount * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteSlot {
        QuoteSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            pinned: self.pinned,
            locale: self.locale.clone(),
            burst: self.burst,
            amount: self.amount,
            archived: self.archived,
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a quote slot in status proposed.
pub fn new_quote_slot(code: &str, name: &str) -> QuoteSlot {
    QuoteSlot {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        pinned: false,
        locale: "".to_string(),
        burst: 0,
        amount: 0.0,
        archived: false,
        rotation_days: 0,
        ratio: 0.0,
        status: QuoteSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_quote_slot("code-292", "name-133");
        assert_eq!(item.status, QuoteSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_quote_slot("code-292", "name-133");
        if item.move_to(QuoteSlotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteSlotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_quote_slot("code-292", "name-133");
        item.status = QuoteSlotStatus::Withdrawn;
        assert!(
            item.move_to(QuoteSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

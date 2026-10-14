use super::*;

use acme_kit::text;

/// Where a tiered quote segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredQuoteSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredQuoteSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredQuoteSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered quote segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegment {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub currency: String,
    pub active: bool,
    pub retries: i64,
    pub window_days: i64,
    pub status: TieredQuoteSegmentStatus,
}

impl TieredQuoteSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredQuoteSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredQuoteSegmentStatus) -> bool {
        match self.status {
            TieredQuoteSegmentStatus::Proposed => {
                return next == TieredQuoteSegmentStatus::Approved || next == TieredQuoteSegmentStatus::Withdrawn;
            }
            TieredQuoteSegmentStatus::Approved => {
                match next {
                    TieredQuoteSegmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQuoteSegmentStatus::Rejected => {
                match next {
                    TieredQuoteSegmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredQuoteSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered quote segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredQuoteSegment {
        TieredQuoteSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            active: self.active,
            retries: self.retries,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered quote segment in status proposed.
pub fn new_tiered_quote_segment(code: &str, label: &str) -> TieredQuoteSegment {
    TieredQuoteSegment {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        currency: "".to_string(),
        active: false,
        retries: 0,
        window_days: 0,
        status: TieredQuoteSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_quote_segment("code-665", "label-35");
        assert_eq!(item.status, TieredQuoteSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_quote_segment("code-665", "label-35");
        if item.move_to(TieredQuoteSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredQuoteSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_quote_segment("code-665", "label-35");
        item.status = TieredQuoteSegmentStatus::Withdrawn;
        assert!(
            item.move_to(TieredQuoteSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

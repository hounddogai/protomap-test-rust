use super::*;

use acme_kit::text;

/// Where a tiered quote claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredQuoteClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredQuoteClaimStatus {
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

impl std::fmt::Display for TieredQuoteClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered quote claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteClaim {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub label: String,
    pub summary: String,
    pub amount: f64,
    pub status: TieredQuoteClaimStatus,
}

impl TieredQuoteClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredQuoteClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredQuoteClaimStatus) -> bool {
        match self.status {
            TieredQuoteClaimStatus::Draft => {
                return next == TieredQuoteClaimStatus::Pending || next == TieredQuoteClaimStatus::Closed;
            }
            TieredQuoteClaimStatus::Pending => {
                return next == TieredQuoteClaimStatus::Closed;
            }
            TieredQuoteClaimStatus::Active => {
                return next == TieredQuoteClaimStatus::Suspended;
            }
            TieredQuoteClaimStatus::Suspended => {
                return next == TieredQuoteClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredQuoteClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered quote claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TieredQuoteClaim {
        TieredQuoteClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered quote claim in status draft.
pub fn new_tiered_quote_claim(code: &str, title: &str) -> TieredQuoteClaim {
    TieredQuoteClaim {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        label: "".to_string(),
        summary: "".to_string(),
        amount: 0.0,
        status: TieredQuoteClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_quote_claim("code-738", "title-706");
        assert_eq!(item.status, TieredQuoteClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tiered_quote_claim("code-738", "title-706");
        if item.move_to(TieredQuoteClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredQuoteClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_quote_claim("code-738", "title-706");
        item.status = TieredQuoteClaimStatus::Closed;
        assert!(
            item.move_to(TieredQuoteClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

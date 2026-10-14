use super::*;

use acme_kit::text;

/// Where a quote score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QuoteScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QuoteScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote score as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScore {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub discount: f64,
    pub host: String,
    pub country: String,
    pub audience: String,
    pub status: QuoteScoreStatus,
}

impl QuoteScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteScoreStatus) -> bool {
        match self.status {
            QuoteScoreStatus::Proposed => {
                match next {
                    QuoteScoreStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteScoreStatus::Approved => {
                match next {
                    QuoteScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteScoreStatus::Rejected => {
                return next == QuoteScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> QuoteScore {
        QuoteScore {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            discount: self.discount,
            host: self.host.clone(),
            country: self.country.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a quote score in status proposed.
pub fn new_quote_score(code: &str, label: &str) -> QuoteScore {
    QuoteScore {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        discount: 0.0,
        host: "".to_string(),
        country: "".to_string(),
        audience: "".to_string(),
        status: QuoteScoreStatus::Proposed,
    }
}

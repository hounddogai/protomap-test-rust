use super::*;

use acme_kit::text;

/// Where a weighted page score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPageScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedPageScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedPageScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted page score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageScore {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub currency: String,
    pub status: WeightedPageScoreStatus,
}

impl WeightedPageScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPageScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPageScoreStatus) -> bool {
        match self.status {
            WeightedPageScoreStatus::Proposed => {
                return next == WeightedPageScoreStatus::Approved || next == WeightedPageScoreStatus::Rejected;
            }
            WeightedPageScoreStatus::Approved => {
                match next {
                    WeightedPageScoreStatus::Rejected | WeightedPageScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPageScoreStatus::Rejected => {
                match next {
                    WeightedPageScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedPageScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted page score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> WeightedPageScore {
        WeightedPageScore {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted page score in status proposed.
pub fn new_weighted_page_score(code: &str, name: &str) -> WeightedPageScore {
    WeightedPageScore {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        currency: "".to_string(),
        status: WeightedPageScoreStatus::Proposed,
    }
}

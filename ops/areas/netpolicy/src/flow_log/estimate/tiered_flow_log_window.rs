use super::*;

use acme_kit::text;

/// Where a tiered flow log window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFlowLogWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredFlowLogWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TieredFlowLogWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered flow log window as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindow {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub capacity: i64,
    pub category: String,
    pub weight: f64,
    pub country: String,
    pub floor_value: f64,
    pub ceiling: f64,
    pub unit: String,
    pub issuer: String,
    pub status: TieredFlowLogWindowStatus,
}

impl TieredFlowLogWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFlowLogWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFlowLogWindowStatus) -> bool {
        match self.status {
            TieredFlowLogWindowStatus::Open => {
                match next {
                    TieredFlowLogWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogWindowStatus::InReview => {
                match next {
                    TieredFlowLogWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogWindowStatus::Resolved => {
                return next == TieredFlowLogWindowStatus::Archived;
            }
            TieredFlowLogWindowStatus::Reopened => {
                return next == TieredFlowLogWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredFlowLogWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered flow log window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ceiling + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFlowLogWindow {
        TieredFlowLogWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            capacity: self.capacity,
            category: self.category.clone(),
            weight: self.weight,
            country: self.country.clone(),
            floor_value: self.floor_value,
            ceiling: self.ceiling,
            unit: self.unit.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered flow log window in status open.
pub fn new_tiered_flow_log_window(code: &str, title: &str) -> TieredFlowLogWindow {
    TieredFlowLogWindow {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        capacity: 0,
        category: "".to_string(),
        weight: 0.0,
        country: "".to_string(),
        floor_value: 0.0,
        ceiling: 0.0,
        unit: "".to_string(),
        issuer: "".to_string(),
        status: TieredFlowLogWindowStatus::Open,
    }
}

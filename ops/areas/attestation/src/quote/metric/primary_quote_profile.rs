use super::*;

use acme_kit::text;

/// Where a primary quote profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryQuoteProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryQuoteProfileStatus {
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

impl std::fmt::Display for PrimaryQuoteProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary quote profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfile {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub rate: f64,
    pub weight: f64,
    pub principal: String,
    pub phone: String,
    pub capacity: i64,
    pub description: String,
    pub burst: i64,
    pub source: String,
    pub status: PrimaryQuoteProfileStatus,
}

impl PrimaryQuoteProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryQuoteProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryQuoteProfileStatus) -> bool {
        match self.status {
            PrimaryQuoteProfileStatus::Planned => {
                return next == PrimaryQuoteProfileStatus::Scheduled || next == PrimaryQuoteProfileStatus::Completed;
            }
            PrimaryQuoteProfileStatus::Scheduled => {
                match next {
                    PrimaryQuoteProfileStatus::InProgress | PrimaryQuoteProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryQuoteProfileStatus::InProgress => {
                match next {
                    PrimaryQuoteProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryQuoteProfileStatus::Completed => {
                return next == PrimaryQuoteProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryQuoteProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary quote profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.burst as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryQuoteProfile {
        PrimaryQuoteProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            rate: self.rate,
            weight: self.weight,
            principal: self.principal.clone(),
            phone: self.phone.clone(),
            capacity: self.capacity,
            description: self.description.clone(),
            burst: self.burst,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary quote profile in status planned.
pub fn new_primary_quote_profile(code: &str, label: &str) -> PrimaryQuoteProfile {
    PrimaryQuoteProfile {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        rate: 0.0,
        weight: 0.0,
        principal: "".to_string(),
        phone: "".to_string(),
        capacity: 0,
        description: "".to_string(),
        burst: 0,
        source: "".to_string(),
        status: PrimaryQuoteProfileStatus::Planned,
    }
}

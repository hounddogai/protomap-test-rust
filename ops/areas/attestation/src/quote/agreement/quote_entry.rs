use super::*;

use acme_kit::text;

/// Where a quote entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for QuoteEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntry {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub region: String,
    pub category: String,
    pub quantity: i64,
    pub audience: String,
    pub label: String,
    pub segment: String,
    pub owner_id: String,
    pub margin: f64,
    pub status: QuoteEntryStatus,
}

impl QuoteEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteEntryStatus) -> bool {
        match self.status {
            QuoteEntryStatus::Idle => {
                return next == QuoteEntryStatus::Ready;
            }
            QuoteEntryStatus::Warming => {
                match next {
                    QuoteEntryStatus::Draining | QuoteEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteEntryStatus::Ready => {
                return next == QuoteEntryStatus::Draining || next == QuoteEntryStatus::Stopped;
            }
            QuoteEntryStatus::Draining => {
                match next {
                    QuoteEntryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quantity as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteEntry {
        QuoteEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            region: self.region.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            audience: self.audience.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            owner_id: self.owner_id.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a quote entry in status idle.
pub fn new_quote_entry(code: &str, name: &str) -> QuoteEntry {
    QuoteEntry {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        region: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        audience: "".to_string(),
        label: "".to_string(),
        segment: "".to_string(),
        owner_id: "".to_string(),
        margin: 0.0,
        status: QuoteEntryStatus::Idle,
    }
}

use super::*;

use acme_kit::text;

/// Where a quote checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QuoteCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QuoteCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote checkpoint as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCheckpoint {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub margin: f64,
    pub source: String,
    pub ceiling: f64,
    pub minimum: f64,
    pub pinned: bool,
    pub status: QuoteCheckpointStatus,
}

impl QuoteCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteCheckpointStatus) -> bool {
        match self.status {
            QuoteCheckpointStatus::Created => {
                return next == QuoteCheckpointStatus::Validated || next == QuoteCheckpointStatus::Published;
            }
            QuoteCheckpointStatus::Validated => {
                match next {
                    QuoteCheckpointStatus::Published | QuoteCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteCheckpointStatus::Published => {
                return next == QuoteCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "quote checkpoint", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.minimum * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteCheckpoint {
        QuoteCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            source: self.source.clone(),
            ceiling: self.ceiling,
            minimum: self.minimum,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a quote checkpoint in status created.
pub fn new_quote_checkpoint(code: &str, label: &str) -> QuoteCheckpoint {
    QuoteCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        margin: 0.0,
        source: "".to_string(),
        ceiling: 0.0,
        minimum: 0.0,
        pinned: false,
        status: QuoteCheckpointStatus::Created,
    }
}

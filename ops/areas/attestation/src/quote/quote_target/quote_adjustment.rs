use super::*;

use acme_kit::text;

/// Where a quote adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QuoteAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for QuoteAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustment {
    pub code: String,
    pub label: String,
    pub name: String,
    pub fingerprint: String,
    pub namespace: String,
    pub quantity: i64,
    pub status: QuoteAdjustmentStatus,
}

impl QuoteAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteAdjustmentStatus) -> bool {
        match self.status {
            QuoteAdjustmentStatus::New => {
                return next == QuoteAdjustmentStatus::Queued || next == QuoteAdjustmentStatus::Running;
            }
            QuoteAdjustmentStatus::Queued => {
                match next {
                    QuoteAdjustmentStatus::Running | QuoteAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteAdjustmentStatus::Running => {
                return next == QuoteAdjustmentStatus::Done;
            }
            QuoteAdjustmentStatus::Done => {
                return next == QuoteAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> QuoteAdjustment {
        QuoteAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            namespace: self.namespace.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote adjustment in status new.
pub fn new_quote_adjustment(code: &str, label: &str) -> QuoteAdjustment {
    QuoteAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        fingerprint: "".to_string(),
        namespace: "".to_string(),
        quantity: 0,
        status: QuoteAdjustmentStatus::New,
    }
}

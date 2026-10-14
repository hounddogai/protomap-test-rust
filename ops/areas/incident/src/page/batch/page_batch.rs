use super::*;

use acme_kit::text;

/// Where a page batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageBatchStatus {
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

impl std::fmt::Display for PageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBatch {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub unit: String,
    pub rate: f64,
    pub issuer: String,
    pub status: PageBatchStatus,
}

impl PageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageBatchStatus) -> bool {
        match self.status {
            PageBatchStatus::Idle => {
                return next == PageBatchStatus::Stopped;
            }
            PageBatchStatus::Warming => {
                return next == PageBatchStatus::Stopped;
            }
            PageBatchStatus::Ready => {
                return next == PageBatchStatus::Draining || next == PageBatchStatus::Stopped;
            }
            PageBatchStatus::Draining => {
                match next {
                    PageBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.retries as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageBatch {
        PageBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            unit: self.unit.clone(),
            rate: self.rate,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a page batch in status idle.
pub fn new_page_batch(code: &str, label: &str) -> PageBatch {
    PageBatch {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        unit: "".to_string(),
        rate: 0.0,
        issuer: "".to_string(),
        status: PageBatchStatus::Idle,
    }
}

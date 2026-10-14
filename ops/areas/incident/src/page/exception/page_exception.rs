use super::*;

use acme_kit::text;

/// Where a page exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageExceptionStatus {
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

impl std::fmt::Display for PageExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageException {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub country: String,
    pub minimum: f64,
    pub summary: String,
    pub status: PageExceptionStatus,
}

impl PageException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageExceptionStatus) -> bool {
        match self.status {
            PageExceptionStatus::Idle => {
                match next {
                    PageExceptionStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageExceptionStatus::Warming => {
                match next {
                    PageExceptionStatus::Ready | PageExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageExceptionStatus::Ready => {
                match next {
                    PageExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageExceptionStatus::Draining => {
                match next {
                    PageExceptionStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "page exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> PageException {
        PageException {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            minimum: self.minimum,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a page exception in status idle.
pub fn new_page_exception(code: &str, name: &str) -> PageException {
    PageException {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        country: "".to_string(),
        minimum: 0.0,
        summary: "".to_string(),
        status: PageExceptionStatus::Idle,
    }
}

use super::*;

use acme_kit::text;

/// Where a page group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageGroupStatus {
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

impl std::fmt::Display for PageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page group as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageGroup {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub currency: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub status: PageGroupStatus,
}

impl PageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageGroupStatus) -> bool {
        match self.status {
            PageGroupStatus::Idle => {
                match next {
                    PageGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageGroupStatus::Warming => {
                match next {
                    PageGroupStatus::Ready | PageGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageGroupStatus::Ready => {
                return next == PageGroupStatus::Draining;
            }
            PageGroupStatus::Draining => {
                match next {
                    PageGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.margin - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageGroup {
        PageGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            currency: self.currency.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page group in status idle.
pub fn new_page_group(code: &str, name: &str) -> PageGroup {
    PageGroup {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        currency: "".to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        contact_email: "".to_string(),
        status: PageGroupStatus::Idle,
    }
}

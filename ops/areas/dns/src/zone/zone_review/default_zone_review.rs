use super::*;

use acme_kit::text;

/// Where a default zone review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultZoneReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultZoneReviewStatus {
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

impl std::fmt::Display for DefaultZoneReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default zone review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReview {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub active: bool,
    pub contact_email: String,
    pub status: DefaultZoneReviewStatus,
}

impl DefaultZoneReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultZoneReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultZoneReviewStatus) -> bool {
        match self.status {
            DefaultZoneReviewStatus::Idle => {
                match next {
                    DefaultZoneReviewStatus::Warming | DefaultZoneReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneReviewStatus::Warming => {
                match next {
                    DefaultZoneReviewStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneReviewStatus::Ready => {
                match next {
                    DefaultZoneReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneReviewStatus::Draining => {
                match next {
                    DefaultZoneReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultZoneReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default zone review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> DefaultZoneReview {
        DefaultZoneReview {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            active: self.active,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default zone review in status idle.
pub fn new_default_zone_review(code: &str, name: &str) -> DefaultZoneReview {
    DefaultZoneReview {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        active: false,
        contact_email: "".to_string(),
        status: DefaultZoneReviewStatus::Idle,
    }
}

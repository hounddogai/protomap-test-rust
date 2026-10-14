use super::*;

use acme_kit::text;

/// Where a usage draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl UsageDraftStatus {
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

impl std::fmt::Display for UsageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage draft as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDraft {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub budget: f64,
    pub audience: String,
    pub external_ref: String,
    pub name: String,
    pub country: String,
    pub ratio: f64,
    pub region: String,
    pub batch_size: i64,
    pub status: UsageDraftStatus,
}

impl UsageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageDraftStatus) -> bool {
        match self.status {
            UsageDraftStatus::Idle => {
                match next {
                    UsageDraftStatus::Ready | UsageDraftStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageDraftStatus::Warming => {
                match next {
                    UsageDraftStatus::Draining | UsageDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageDraftStatus::Ready => {
                match next {
                    UsageDraftStatus::Draining | UsageDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageDraftStatus::Draining => {
                match next {
                    UsageDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: UsageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.budget * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageDraft {
        UsageDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            budget: self.budget,
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            ratio: self.ratio,
            region: self.region.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage draft in status idle.
pub fn new_usage_draft(code: &str, label: &str) -> UsageDraft {
    UsageDraft {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        budget: 0.0,
        audience: "".to_string(),
        external_ref: "".to_string(),
        name: "".to_string(),
        country: "".to_string(),
        ratio: 0.0,
        region: "".to_string(),
        batch_size: 0,
        status: UsageDraftStatus::Idle,
    }
}

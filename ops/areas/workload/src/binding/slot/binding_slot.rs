use super::*;

use acme_kit::text;

/// Where a binding slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BindingSlotStatus {
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

impl std::fmt::Display for BindingSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlot {
    pub code: String,
    pub title: String,
    pub country: String,
    pub quantity: i64,
    pub minimum: f64,
    pub segment: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub margin: f64,
    pub status: BindingSlotStatus,
}

impl BindingSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingSlotStatus) -> bool {
        match self.status {
            BindingSlotStatus::New => {
                return next == BindingSlotStatus::Running || next == BindingSlotStatus::Done;
            }
            BindingSlotStatus::Queued => {
                return next == BindingSlotStatus::Running;
            }
            BindingSlotStatus::Running => {
                return next == BindingSlotStatus::Done;
            }
            BindingSlotStatus::Done => {
                match next {
                    BindingSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: BindingSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingSlot {
        BindingSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            minimum: self.minimum,
            segment: self.segment.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a binding slot in status new.
pub fn new_binding_slot(code: &str, title: &str) -> BindingSlot {
    BindingSlot {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        quantity: 0,
        minimum: 0.0,
        segment: "".to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        margin: 0.0,
        status: BindingSlotStatus::New,
    }
}

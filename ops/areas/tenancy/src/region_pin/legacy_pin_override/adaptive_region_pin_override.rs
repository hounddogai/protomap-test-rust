use super::*;

use acme_kit::text;

/// Where a adaptive region pin override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveRegionPinOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveRegionPinOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive region pin override as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinOverride {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub segment: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub status: AdaptiveRegionPinOverrideStatus,
}

impl AdaptiveRegionPinOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinOverrideStatus) -> bool {
        match self.status {
            AdaptiveRegionPinOverrideStatus::Draft => {
                return next == AdaptiveRegionPinOverrideStatus::Suspended;
            }
            AdaptiveRegionPinOverrideStatus::Pending => {
                match next {
                    AdaptiveRegionPinOverrideStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinOverrideStatus::Active => {
                match next {
                    AdaptiveRegionPinOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinOverrideStatus::Suspended => {
                return next == AdaptiveRegionPinOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> AdaptiveRegionPinOverride {
        AdaptiveRegionPinOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            segment: self.segment.clone(),
            tags: self.tags.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive region pin override in status draft.
pub fn new_adaptive_region_pin_override(code: &str, label: &str) -> AdaptiveRegionPinOverride {
    AdaptiveRegionPinOverride {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        segment: "".to_string(),
        tags: Vec::new(),
        score: 0.0,
        status: AdaptiveRegionPinOverrideStatus::Draft,
    }
}

use super::*;

use acme_kit::text;

/// Where a fallback page adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPageAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPageAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPageAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback page adjustment as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageAdjustment {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub amount: f64,
    pub description: String,
    pub locale: String,
    pub retries: i64,
    pub revision: i64,
    pub source: String,
    pub ratio: f64,
    pub status: FallbackPageAdjustmentStatus,
}

impl FallbackPageAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPageAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPageAdjustmentStatus) -> bool {
        match self.status {
            FallbackPageAdjustmentStatus::Created => {
                match next {
                    FallbackPageAdjustmentStatus::Published | FallbackPageAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPageAdjustmentStatus::Validated => {
                match next {
                    FallbackPageAdjustmentStatus::Published | FallbackPageAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPageAdjustmentStatus::Published => {
                return next == FallbackPageAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPageAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback page adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback page adjustment", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.amount * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPageAdjustment {
        FallbackPageAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            amount: self.amount,
            description: self.description.clone(),
            locale: self.locale.clone(),
            retries: self.retries,
            revision: self.revision,
            source: self.source.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback page adjustment in status created.
pub fn new_fallback_page_adjustment(code: &str, name: &str) -> FallbackPageAdjustment {
    FallbackPageAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        amount: 0.0,
        description: "".to_string(),
        locale: "".to_string(),
        retries: 0,
        revision: 0,
        source: "".to_string(),
        ratio: 0.0,
        status: FallbackPageAdjustmentStatus::Created,
    }
}

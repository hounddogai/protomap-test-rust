use super::*;

use acme_kit::text;

/// Where a fallback redaction adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRedactionAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRedactionAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRedactionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback redaction adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustment {
    pub code: String,
    pub title: String,
    pub description: String,
    pub margin: f64,
    pub key_id: String,
    pub locale: String,
    pub revision: i64,
    pub priority: i64,
    pub status: FallbackRedactionAdjustmentStatus,
}

impl FallbackRedactionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRedactionAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRedactionAdjustmentStatus) -> bool {
        match self.status {
            FallbackRedactionAdjustmentStatus::Created => {
                return next == FallbackRedactionAdjustmentStatus::Retired;
            }
            FallbackRedactionAdjustmentStatus::Validated => {
                match next {
                    FallbackRedactionAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRedactionAdjustmentStatus::Published => {
                return next == FallbackRedactionAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRedactionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback redaction adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback redaction adjustment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRedactionAdjustment {
        FallbackRedactionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            margin: self.margin,
            key_id: self.key_id.clone(),
            locale: self.locale.clone(),
            revision: self.revision,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback redaction adjustment in status created.
pub fn new_fallback_redaction_adjustment(code: &str, title: &str) -> FallbackRedactionAdjustment {
    FallbackRedactionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        margin: 0.0,
        key_id: "".to_string(),
        locale: "".to_string(),
        revision: 0,
        priority: 0,
        status: FallbackRedactionAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_redaction_adjustment("code-147", "title-208");
        assert_eq!(item.status, FallbackRedactionAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_fallback_redaction_adjustment("code-147", "title-208");
        if item.move_to(FallbackRedactionAdjustmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRedactionAdjustmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_redaction_adjustment("code-147", "title-208");
        item.status = FallbackRedactionAdjustmentStatus::Retired;
        assert!(
            item.move_to(FallbackRedactionAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

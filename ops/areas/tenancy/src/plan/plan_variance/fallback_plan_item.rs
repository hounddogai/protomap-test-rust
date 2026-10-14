use super::*;

use acme_kit::text;

/// Where a fallback plan item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPlanItemStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPlanItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPlanItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback plan item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItem {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub label: String,
    pub owner_id: String,
    pub created_at: i64,
    pub visible: bool,
    pub notes: String,
    pub quota_units: i64,
    pub status: FallbackPlanItemStatus,
}

impl FallbackPlanItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPlanItemStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPlanItemStatus) -> bool {
        match self.status {
            FallbackPlanItemStatus::Created => {
                return next == FallbackPlanItemStatus::Published || next == FallbackPlanItemStatus::Retired;
            }
            FallbackPlanItemStatus::Validated => {
                return next == FallbackPlanItemStatus::Published || next == FallbackPlanItemStatus::Retired;
            }
            FallbackPlanItemStatus::Published => {
                return next == FallbackPlanItemStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPlanItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback plan item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback plan item", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPlanItem {
        FallbackPlanItem {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            visible: self.visible,
            notes: self.notes.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback plan item in status created.
pub fn new_fallback_plan_item(code: &str, title: &str) -> FallbackPlanItem {
    FallbackPlanItem {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        label: "".to_string(),
        owner_id: "".to_string(),
        created_at: 0,
        visible: false,
        notes: "".to_string(),
        quota_units: 0,
        status: FallbackPlanItemStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_plan_item("code-864", "title-7");
        assert_eq!(item.status, FallbackPlanItemStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_fallback_plan_item("code-864", "title-7");
        if item.move_to(FallbackPlanItemStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPlanItemStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_plan_item("code-864", "title-7");
        item.status = FallbackPlanItemStatus::Retired;
        assert!(
            item.move_to(FallbackPlanItemStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

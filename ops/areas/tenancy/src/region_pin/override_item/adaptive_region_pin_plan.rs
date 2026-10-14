use super::*;

use acme_kit::text;

/// Where a adaptive region pin plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveRegionPinPlanStatus {
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

impl std::fmt::Display for AdaptiveRegionPinPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive region pin plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlan {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub issuer: String,
    pub category: String,
    pub expires_at: i64,
    pub minimum: f64,
    pub capacity: i64,
    pub key_id: String,
    pub starts_at: i64,
    pub status: AdaptiveRegionPinPlanStatus,
}

impl AdaptiveRegionPinPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinPlanStatus) -> bool {
        match self.status {
            AdaptiveRegionPinPlanStatus::New => {
                match next {
                    AdaptiveRegionPinPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinPlanStatus::Queued => {
                match next {
                    AdaptiveRegionPinPlanStatus::Done | AdaptiveRegionPinPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinPlanStatus::Running => {
                return next == AdaptiveRegionPinPlanStatus::Done;
            }
            AdaptiveRegionPinPlanStatus::Done => {
                return next == AdaptiveRegionPinPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive region pin plan", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.capacity as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinPlan {
        AdaptiveRegionPinPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            issuer: self.issuer.clone(),
            category: self.category.clone(),
            expires_at: self.expires_at,
            minimum: self.minimum,
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive region pin plan in status new.
pub fn new_adaptive_region_pin_plan(code: &str, title: &str) -> AdaptiveRegionPinPlan {
    AdaptiveRegionPinPlan {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        issuer: "".to_string(),
        category: "".to_string(),
        expires_at: 0,
        minimum: 0.0,
        capacity: 0,
        key_id: "".to_string(),
        starts_at: 0,
        status: AdaptiveRegionPinPlanStatus::New,
    }
}

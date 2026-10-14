use super::*;

use acme_kit::text;

/// Where a strict forecast milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictForecastMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictForecastMilestoneStatus {
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

impl std::fmt::Display for StrictForecastMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict forecast milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestone {
    pub code: String,
    pub name: String,
    pub region: String,
    pub tenant_id: String,
    pub archived: bool,
    pub label: String,
    pub pinned: bool,
    pub created_at: i64,
    pub principal: String,
    pub namespace: String,
    pub status: StrictForecastMilestoneStatus,
}

impl StrictForecastMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictForecastMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictForecastMilestoneStatus) -> bool {
        match self.status {
            StrictForecastMilestoneStatus::New => {
                return next == StrictForecastMilestoneStatus::Running || next == StrictForecastMilestoneStatus::Failed;
            }
            StrictForecastMilestoneStatus::Queued => {
                match next {
                    StrictForecastMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictForecastMilestoneStatus::Running => {
                return next == StrictForecastMilestoneStatus::Done || next == StrictForecastMilestoneStatus::Failed;
            }
            StrictForecastMilestoneStatus::Done => {
                match next {
                    StrictForecastMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictForecastMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict forecast milestone {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> StrictForecastMilestone {
        StrictForecastMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            label: self.label.clone(),
            pinned: self.pinned,
            created_at: self.created_at,
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict forecast milestone in status new.
pub fn new_strict_forecast_milestone(code: &str, name: &str) -> StrictForecastMilestone {
    StrictForecastMilestone {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        tenant_id: "".to_string(),
        archived: false,
        label: "".to_string(),
        pinned: false,
        created_at: 0,
        principal: "".to_string(),
        namespace: "".to_string(),
        status: StrictForecastMilestoneStatus::New,
    }
}

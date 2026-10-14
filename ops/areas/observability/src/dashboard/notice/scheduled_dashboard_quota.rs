use super::*;

use acme_kit::text;

/// Where a scheduled dashboard quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDashboardQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledDashboardQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledDashboardQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled dashboard quota as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardQuota {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub amount: f64,
    pub audience: String,
    pub key_id: String,
    pub ends_at: i64,
    pub cost: f64,
    pub scope: String,
    pub cluster: String,
    pub status: ScheduledDashboardQuotaStatus,
}

impl ScheduledDashboardQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDashboardQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDashboardQuotaStatus) -> bool {
        match self.status {
            ScheduledDashboardQuotaStatus::Proposed => {
                return next == ScheduledDashboardQuotaStatus::Rejected;
            }
            ScheduledDashboardQuotaStatus::Approved => {
                match next {
                    ScheduledDashboardQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardQuotaStatus::Rejected => {
                match next {
                    ScheduledDashboardQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledDashboardQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dashboard quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled dashboard quota", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ends_at as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDashboardQuota {
        ScheduledDashboardQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            audience: self.audience.clone(),
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            cost: self.cost,
            scope: self.scope.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled dashboard quota in status proposed.
pub fn new_scheduled_dashboard_quota(code: &str, name: &str) -> ScheduledDashboardQuota {
    ScheduledDashboardQuota {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        amount: 0.0,
        audience: "".to_string(),
        key_id: "".to_string(),
        ends_at: 0,
        cost: 0.0,
        scope: "".to_string(),
        cluster: "".to_string(),
        status: ScheduledDashboardQuotaStatus::Proposed,
    }
}

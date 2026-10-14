use super::*;

use acme_kit::text;

/// Where a dashboard quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DashboardQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuota {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub margin: f64,
    pub threshold: f64,
    pub host: String,
    pub starts_at: i64,
    pub updated_at: i64,
    pub discount: f64,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub status: DashboardQuotaStatus,
}

impl DashboardQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardQuotaStatus) -> bool {
        match self.status {
            DashboardQuotaStatus::Planned => {
                match next {
                    DashboardQuotaStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardQuotaStatus::Scheduled => {
                match next {
                    DashboardQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardQuotaStatus::InProgress => {
                return next == DashboardQuotaStatus::Completed || next == DashboardQuotaStatus::Cancelled;
            }
            DashboardQuotaStatus::Completed => {
                match next {
                    DashboardQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DashboardQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardQuota {
        DashboardQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            margin: self.margin,
            threshold: self.threshold,
            host: self.host.clone(),
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            discount: self.discount,
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dashboard quota in status planned.
pub fn new_dashboard_quota(code: &str, name: &str) -> DashboardQuota {
    DashboardQuota {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        margin: 0.0,
        threshold: 0.0,
        host: "".to_string(),
        starts_at: 0,
        updated_at: 0,
        discount: 0.0,
        priority: 0,
        ttl_seconds: 0,
        status: DashboardQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dashboard_quota("code-816", "name-375");
        assert_eq!(item.status, DashboardQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_dashboard_quota("code-816", "name-375");
        if item.move_to(DashboardQuotaStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardQuotaStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dashboard_quota("code-816", "name-375");
        item.status = DashboardQuotaStatus::Cancelled;
        assert!(
            item.move_to(DashboardQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

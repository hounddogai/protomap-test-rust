use super::*;

use acme_kit::text;

/// Where a dashboard metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DashboardMetricStatus {
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

impl std::fmt::Display for DashboardMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard metric as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMetric {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub currency: String,
    pub capacity: i64,
    pub principal: String,
    pub channel: String,
    pub status: DashboardMetricStatus,
}

impl DashboardMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardMetricStatus) -> bool {
        match self.status {
            DashboardMetricStatus::Draft => {
                match next {
                    DashboardMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardMetricStatus::Pending => {
                match next {
                    DashboardMetricStatus::Active | DashboardMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardMetricStatus::Active => {
                return next == DashboardMetricStatus::Closed;
            }
            DashboardMetricStatus::Suspended => {
                match next {
                    DashboardMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: DashboardMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardMetric {
        DashboardMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            currency: self.currency.clone(),
            capacity: self.capacity,
            principal: self.principal.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dashboard metric in status draft.
pub fn new_dashboard_metric(code: &str, title: &str) -> DashboardMetric {
    DashboardMetric {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        fingerprint: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        currency: "".to_string(),
        capacity: 0,
        principal: "".to_string(),
        channel: "".to_string(),
        status: DashboardMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dashboard_metric("code-370", "title-938");
        assert_eq!(item.status, DashboardMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_dashboard_metric("code-370", "title-938");
        if item.move_to(DashboardMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dashboard_metric("code-370", "title-938");
        item.status = DashboardMetricStatus::Closed;
        assert!(
            item.move_to(DashboardMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

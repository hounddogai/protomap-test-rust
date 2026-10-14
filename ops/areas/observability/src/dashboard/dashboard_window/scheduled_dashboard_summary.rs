use super::*;

use acme_kit::text;

/// Where a scheduled dashboard summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDashboardSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledDashboardSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledDashboardSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled dashboard summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardSummary {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub revision: i64,
    pub timezone: String,
    pub namespace: String,
    pub window_days: i64,
    pub margin: f64,
    pub visible: bool,
    pub ratio: f64,
    pub owner_id: String,
    pub status: ScheduledDashboardSummaryStatus,
}

impl ScheduledDashboardSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDashboardSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDashboardSummaryStatus) -> bool {
        match self.status {
            ScheduledDashboardSummaryStatus::Requested => {
                match next {
                    ScheduledDashboardSummaryStatus::Granted | ScheduledDashboardSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardSummaryStatus::Granted => {
                return next == ScheduledDashboardSummaryStatus::Revoked;
            }
            ScheduledDashboardSummaryStatus::Revoked => {
                match next {
                    ScheduledDashboardSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledDashboardSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dashboard summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.revision as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDashboardSummary {
        ScheduledDashboardSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            revision: self.revision,
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            window_days: self.window_days,
            margin: self.margin,
            visible: self.visible,
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled dashboard summary in status requested.
pub fn new_scheduled_dashboard_summary(code: &str, name: &str) -> ScheduledDashboardSummary {
    ScheduledDashboardSummary {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        revision: 0,
        timezone: "".to_string(),
        namespace: "".to_string(),
        window_days: 0,
        margin: 0.0,
        visible: false,
        ratio: 0.0,
        owner_id: "".to_string(),
        status: ScheduledDashboardSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_dashboard_summary("code-11", "name-272");
        assert_eq!(item.status, ScheduledDashboardSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_dashboard_summary("code-11", "name-272");
        if item.move_to(ScheduledDashboardSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDashboardSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_dashboard_summary("code-11", "name-272");
        item.status = ScheduledDashboardSummaryStatus::Expired;
        assert!(
            item.move_to(ScheduledDashboardSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

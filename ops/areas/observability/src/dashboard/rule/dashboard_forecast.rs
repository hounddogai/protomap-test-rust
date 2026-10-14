use super::*;

use acme_kit::text;

/// Where a dashboard forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardForecastStatus {
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

impl std::fmt::Display for DashboardForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard forecast as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecast {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub segment: String,
    pub host: String,
    pub visible: bool,
    pub name: String,
    pub capacity: i64,
    pub status: DashboardForecastStatus,
}

impl DashboardForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardForecastStatus) -> bool {
        match self.status {
            DashboardForecastStatus::New => {
                return next == DashboardForecastStatus::Queued;
            }
            DashboardForecastStatus::Queued => {
                return next == DashboardForecastStatus::Running;
            }
            DashboardForecastStatus::Running => {
                match next {
                    DashboardForecastStatus::Done | DashboardForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardForecastStatus::Done => {
                match next {
                    DashboardForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: DashboardForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> DashboardForecast {
        DashboardForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            segment: self.segment.clone(),
            host: self.host.clone(),
            visible: self.visible,
            name: self.name.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a dashboard forecast in status new.
pub fn new_dashboard_forecast(code: &str, label: &str) -> DashboardForecast {
    DashboardForecast {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        segment: "".to_string(),
        host: "".to_string(),
        visible: false,
        name: "".to_string(),
        capacity: 0,
        status: DashboardForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_forecast("code-918", "label-409");
        assert_eq!(item.status, DashboardForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_dashboard_forecast("code-918", "label-409");
        if item.move_to(DashboardForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_forecast("code-918", "label-409");
        item.status = DashboardForecastStatus::Failed;
        assert!(
            item.move_to(DashboardForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

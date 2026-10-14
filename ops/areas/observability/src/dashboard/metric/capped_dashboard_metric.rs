use super::*;

use acme_kit::text;

/// Where a capped dashboard metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDashboardMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedDashboardMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedDashboardMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped dashboard metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetric {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub burst: i64,
    pub version: i64,
    pub status: CappedDashboardMetricStatus,
}

impl CappedDashboardMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDashboardMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDashboardMetricStatus) -> bool {
        match self.status {
            CappedDashboardMetricStatus::Requested => {
                return next == CappedDashboardMetricStatus::Expired;
            }
            CappedDashboardMetricStatus::Granted => {
                return next == CappedDashboardMetricStatus::Revoked || next == CappedDashboardMetricStatus::Expired;
            }
            CappedDashboardMetricStatus::Revoked => {
                match next {
                    CappedDashboardMetricStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedDashboardMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dashboard metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped dashboard metric", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDashboardMetric {
        CappedDashboardMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped dashboard metric in status requested.
pub fn new_capped_dashboard_metric(code: &str, label: &str) -> CappedDashboardMetric {
    CappedDashboardMetric {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        duration_minutes: 0,
        burst: 0,
        version: 0,
        status: CappedDashboardMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_dashboard_metric("code-622", "label-736");
        assert_eq!(item.status, CappedDashboardMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_capped_dashboard_metric("code-622", "label-736");
        if item.move_to(CappedDashboardMetricStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDashboardMetricStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_dashboard_metric("code-622", "label-736");
        item.status = CappedDashboardMetricStatus::Expired;
        assert!(
            item.move_to(CappedDashboardMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

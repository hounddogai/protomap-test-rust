use super::*;

use acme_kit::text;

/// Where a default dashboard metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDashboardMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultDashboardMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultDashboardMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default dashboard metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardMetric {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub rate: f64,
    pub channel: String,
    pub external_ref: String,
    pub cipher: String,
    pub tags: Vec<String>,
    pub status: DefaultDashboardMetricStatus,
}

impl DefaultDashboardMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDashboardMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDashboardMetricStatus) -> bool {
        match self.status {
            DefaultDashboardMetricStatus::Proposed => {
                return next == DefaultDashboardMetricStatus::Approved;
            }
            DefaultDashboardMetricStatus::Approved => {
                return next == DefaultDashboardMetricStatus::Rejected
                    || next == DefaultDashboardMetricStatus::Withdrawn;
            }
            DefaultDashboardMetricStatus::Rejected => {
                match next {
                    DefaultDashboardMetricStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultDashboardMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dashboard metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDashboardMetric {
        DefaultDashboardMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            rate: self.rate,
            channel: self.channel.clone(),
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default dashboard metric in status proposed.
pub fn new_default_dashboard_metric(code: &str, label: &str) -> DefaultDashboardMetric {
    DefaultDashboardMetric {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        rate: 0.0,
        channel: "".to_string(),
        external_ref: "".to_string(),
        cipher: "".to_string(),
        tags: Vec::new(),
        status: DefaultDashboardMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_dashboard_metric("code-632", "label-233");
        assert_eq!(item.status, DefaultDashboardMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_dashboard_metric("code-632", "label-233");
        if item.move_to(DefaultDashboardMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDashboardMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_dashboard_metric("code-632", "label-233");
        item.status = DefaultDashboardMetricStatus::Withdrawn;
        assert!(
            item.move_to(DefaultDashboardMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

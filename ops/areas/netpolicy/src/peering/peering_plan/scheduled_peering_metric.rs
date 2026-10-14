use super::*;

use acme_kit::text;

/// Where a scheduled peering metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPeeringMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledPeeringMetricStatus {
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

impl std::fmt::Display for ScheduledPeeringMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled peering metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringMetric {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub summary: String,
    pub description: String,
    pub segment: String,
    pub priority: i64,
    pub currency: String,
    pub maximum: f64,
    pub ratio: f64,
    pub status: ScheduledPeeringMetricStatus,
}

impl ScheduledPeeringMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPeeringMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPeeringMetricStatus) -> bool {
        match self.status {
            ScheduledPeeringMetricStatus::Draft => {
                return next == ScheduledPeeringMetricStatus::Closed;
            }
            ScheduledPeeringMetricStatus::Pending => {
                return next == ScheduledPeeringMetricStatus::Suspended || next == ScheduledPeeringMetricStatus::Closed;
            }
            ScheduledPeeringMetricStatus::Active => {
                return next == ScheduledPeeringMetricStatus::Suspended || next == ScheduledPeeringMetricStatus::Closed;
            }
            ScheduledPeeringMetricStatus::Suspended => {
                return next == ScheduledPeeringMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledPeeringMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled peering metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.maximum + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPeeringMetric {
        ScheduledPeeringMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            summary: self.summary.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            priority: self.priority,
            currency: self.currency.clone(),
            maximum: self.maximum,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled peering metric in status draft.
pub fn new_scheduled_peering_metric(code: &str, name: &str) -> ScheduledPeeringMetric {
    ScheduledPeeringMetric {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        summary: "".to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        priority: 0,
        currency: "".to_string(),
        maximum: 0.0,
        ratio: 0.0,
        status: ScheduledPeeringMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_peering_metric("code-832", "name-917");
        assert_eq!(item.status, ScheduledPeeringMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_scheduled_peering_metric("code-832", "name-917");
        if item.move_to(ScheduledPeeringMetricStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPeeringMetricStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_peering_metric("code-832", "name-917");
        item.status = ScheduledPeeringMetricStatus::Closed;
        assert!(
            item.move_to(ScheduledPeeringMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

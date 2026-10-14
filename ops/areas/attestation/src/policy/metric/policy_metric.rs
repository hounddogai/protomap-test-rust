use super::*;

use acme_kit::text;

/// Where a policy metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyMetricStatus {
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

impl std::fmt::Display for PolicyMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetric {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub created_at: i64,
    pub notes: String,
    pub title: String,
    pub priority: i64,
    pub weight: f64,
    pub category: String,
    pub channel: String,
    pub rate: f64,
    pub status: PolicyMetricStatus,
}

impl PolicyMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyMetricStatus) -> bool {
        match self.status {
            PolicyMetricStatus::Planned => {
                match next {
                    PolicyMetricStatus::InProgress | PolicyMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyMetricStatus::Scheduled => {
                return next == PolicyMetricStatus::InProgress;
            }
            PolicyMetricStatus::InProgress => {
                match next {
                    PolicyMetricStatus::Completed | PolicyMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyMetricStatus::Completed => {
                return next == PolicyMetricStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy metric {} cannot move from {} to {}",
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
        format!("{} {}", "policy metric", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.weight - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyMetric {
        PolicyMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            created_at: self.created_at,
            notes: self.notes.clone(),
            title: self.title.clone(),
            priority: self.priority,
            weight: self.weight,
            category: self.category.clone(),
            channel: self.channel.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a policy metric in status planned.
pub fn new_policy_metric(code: &str, label: &str) -> PolicyMetric {
    PolicyMetric {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        created_at: 0,
        notes: "".to_string(),
        title: "".to_string(),
        priority: 0,
        weight: 0.0,
        category: "".to_string(),
        channel: "".to_string(),
        rate: 0.0,
        status: PolicyMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_metric("code-600", "label-155");
        assert_eq!(item.status, PolicyMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_policy_metric("code-600", "label-155");
        if item.move_to(PolicyMetricStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyMetricStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_metric("code-600", "label-155");
        item.status = PolicyMetricStatus::Cancelled;
        assert!(
            item.move_to(PolicyMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

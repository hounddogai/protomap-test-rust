use super::*;

use acme_kit::text;

/// Where a default retention metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRetentionMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRetentionMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultRetentionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default retention metric as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionMetric {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub approved: bool,
    pub ceiling: f64,
    pub limit: i64,
    pub issuer: String,
    pub reviewer: String,
    pub weight: f64,
    pub cost: f64,
    pub floor_value: f64,
    pub status: DefaultRetentionMetricStatus,
}

impl DefaultRetentionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRetentionMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRetentionMetricStatus) -> bool {
        match self.status {
            DefaultRetentionMetricStatus::Open => {
                match next {
                    DefaultRetentionMetricStatus::Resolved | DefaultRetentionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRetentionMetricStatus::InReview => {
                match next {
                    DefaultRetentionMetricStatus::Reopened | DefaultRetentionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRetentionMetricStatus::Resolved => {
                return next == DefaultRetentionMetricStatus::Archived;
            }
            DefaultRetentionMetricStatus::Reopened => {
                match next {
                    DefaultRetentionMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRetentionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default retention metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.limit as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRetentionMetric {
        DefaultRetentionMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            approved: self.approved,
            ceiling: self.ceiling,
            limit: self.limit,
            issuer: self.issuer.clone(),
            reviewer: self.reviewer.clone(),
            weight: self.weight,
            cost: self.cost,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default retention metric in status open.
pub fn new_default_retention_metric(code: &str, name: &str) -> DefaultRetentionMetric {
    DefaultRetentionMetric {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        approved: false,
        ceiling: 0.0,
        limit: 0,
        issuer: "".to_string(),
        reviewer: "".to_string(),
        weight: 0.0,
        cost: 0.0,
        floor_value: 0.0,
        status: DefaultRetentionMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_retention_metric("code-94", "name-356");
        assert_eq!(item.status, DefaultRetentionMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_retention_metric("code-94", "name-356");
        if item.move_to(DefaultRetentionMetricStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRetentionMetricStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_retention_metric("code-94", "name-356");
        item.status = DefaultRetentionMetricStatus::Archived;
        assert!(
            item.move_to(DefaultRetentionMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

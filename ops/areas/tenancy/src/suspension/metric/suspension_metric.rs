use super::*;

use acme_kit::text;

/// Where a suspension metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionMetricStatus {
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

impl std::fmt::Display for SuspensionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMetric {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub source: String,
    pub budget: f64,
    pub cluster: String,
    pub description: String,
    pub retries: i64,
    pub status: SuspensionMetricStatus,
}

impl SuspensionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionMetricStatus) -> bool {
        match self.status {
            SuspensionMetricStatus::Open => {
                return next == SuspensionMetricStatus::InReview || next == SuspensionMetricStatus::Reopened;
            }
            SuspensionMetricStatus::InReview => {
                match next {
                    SuspensionMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionMetricStatus::Resolved => {
                match next {
                    SuspensionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionMetricStatus::Reopened => {
                match next {
                    SuspensionMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: SuspensionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.budget * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionMetric {
        SuspensionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            source: self.source.clone(),
            budget: self.budget,
            cluster: self.cluster.clone(),
            description: self.description.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension metric in status open.
pub fn new_suspension_metric(code: &str, title: &str) -> SuspensionMetric {
    SuspensionMetric {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        source: "".to_string(),
        budget: 0.0,
        cluster: "".to_string(),
        description: "".to_string(),
        retries: 0,
        status: SuspensionMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_metric("code-231", "title-408");
        assert_eq!(item.status, SuspensionMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_suspension_metric("code-231", "title-408");
        if item.move_to(SuspensionMetricStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionMetricStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_metric("code-231", "title-408");
        item.status = SuspensionMetricStatus::Archived;
        assert!(
            item.move_to(SuspensionMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

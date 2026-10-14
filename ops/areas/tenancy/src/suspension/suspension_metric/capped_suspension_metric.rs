use super::*;

use acme_kit::text;

/// Where a capped suspension metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSuspensionMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedSuspensionMetricStatus {
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

impl std::fmt::Display for CappedSuspensionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped suspension metric as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionMetric {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub audience: String,
    pub revision: i64,
    pub source: String,
    pub ratio: f64,
    pub status: CappedSuspensionMetricStatus,
}

impl CappedSuspensionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSuspensionMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSuspensionMetricStatus) -> bool {
        match self.status {
            CappedSuspensionMetricStatus::Open => {
                return next == CappedSuspensionMetricStatus::Resolved
                    || next == CappedSuspensionMetricStatus::Archived;
            }
            CappedSuspensionMetricStatus::InReview => {
                match next {
                    CappedSuspensionMetricStatus::Reopened | CappedSuspensionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSuspensionMetricStatus::Resolved => {
                match next {
                    CappedSuspensionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSuspensionMetricStatus::Reopened => {
                return next == CappedSuspensionMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSuspensionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped suspension metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSuspensionMetric {
        CappedSuspensionMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            audience: self.audience.clone(),
            revision: self.revision,
            source: self.source.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped suspension metric in status open.
pub fn new_capped_suspension_metric(code: &str, label: &str) -> CappedSuspensionMetric {
    CappedSuspensionMetric {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        audience: "".to_string(),
        revision: 0,
        source: "".to_string(),
        ratio: 0.0,
        status: CappedSuspensionMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_suspension_metric("code-473", "label-772");
        assert_eq!(item.status, CappedSuspensionMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_capped_suspension_metric("code-473", "label-772");
        if item.move_to(CappedSuspensionMetricStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSuspensionMetricStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_suspension_metric("code-473", "label-772");
        item.status = CappedSuspensionMetricStatus::Archived;
        assert!(
            item.move_to(CappedSuspensionMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a adaptive region pin metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveRegionPinMetricStatus {
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

impl std::fmt::Display for AdaptiveRegionPinMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive region pin metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinMetric {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub source: String,
    pub scope: String,
    pub maximum: f64,
    pub status: AdaptiveRegionPinMetricStatus,
}

impl AdaptiveRegionPinMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinMetricStatus) -> bool {
        match self.status {
            AdaptiveRegionPinMetricStatus::Open => {
                match next {
                    AdaptiveRegionPinMetricStatus::Reopened | AdaptiveRegionPinMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinMetricStatus::InReview => {
                match next {
                    AdaptiveRegionPinMetricStatus::Resolved | AdaptiveRegionPinMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinMetricStatus::Resolved => {
                return next == AdaptiveRegionPinMetricStatus::Reopened
                    || next == AdaptiveRegionPinMetricStatus::Archived;
            }
            AdaptiveRegionPinMetricStatus::Reopened => {
                match next {
                    AdaptiveRegionPinMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveRegionPinMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.score * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinMetric {
        AdaptiveRegionPinMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            source: self.source.clone(),
            scope: self.scope.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive region pin metric in status open.
pub fn new_adaptive_region_pin_metric(code: &str, title: &str) -> AdaptiveRegionPinMetric {
    AdaptiveRegionPinMetric {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        source: "".to_string(),
        scope: "".to_string(),
        maximum: 0.0,
        status: AdaptiveRegionPinMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_region_pin_metric("code-981", "title-743");
        assert_eq!(item.status, AdaptiveRegionPinMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_adaptive_region_pin_metric("code-981", "title-743");
        if item.move_to(AdaptiveRegionPinMetricStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRegionPinMetricStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_region_pin_metric("code-981", "title-743");
        item.status = AdaptiveRegionPinMetricStatus::Archived;
        assert!(
            item.move_to(AdaptiveRegionPinMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

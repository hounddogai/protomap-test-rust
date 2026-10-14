use super::*;

use acme_kit::text;

/// Where a capped segment metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSegmentMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedSegmentMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedSegmentMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped segment metric as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetric {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub locale: String,
    pub summary: String,
    pub cost: f64,
    pub segment: String,
    pub weight: f64,
    pub rotation_days: i64,
    pub principal: String,
    pub status: CappedSegmentMetricStatus,
}

impl CappedSegmentMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSegmentMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSegmentMetricStatus) -> bool {
        match self.status {
            CappedSegmentMetricStatus::Proposed => {
                return next == CappedSegmentMetricStatus::Approved || next == CappedSegmentMetricStatus::Withdrawn;
            }
            CappedSegmentMetricStatus::Approved => {
                match next {
                    CappedSegmentMetricStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSegmentMetricStatus::Rejected => {
                return next == CappedSegmentMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSegmentMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped segment metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped segment metric", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSegmentMetric {
        CappedSegmentMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            summary: self.summary.clone(),
            cost: self.cost,
            segment: self.segment.clone(),
            weight: self.weight,
            rotation_days: self.rotation_days,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped segment metric in status proposed.
pub fn new_capped_segment_metric(code: &str, label: &str) -> CappedSegmentMetric {
    CappedSegmentMetric {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        locale: "".to_string(),
        summary: "".to_string(),
        cost: 0.0,
        segment: "".to_string(),
        weight: 0.0,
        rotation_days: 0,
        principal: "".to_string(),
        status: CappedSegmentMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_segment_metric("code-479", "label-53");
        assert_eq!(item.status, CappedSegmentMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_segment_metric("code-479", "label-53");
        if item.move_to(CappedSegmentMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSegmentMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_segment_metric("code-479", "label-53");
        item.status = CappedSegmentMetricStatus::Withdrawn;
        assert!(
            item.move_to(CappedSegmentMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

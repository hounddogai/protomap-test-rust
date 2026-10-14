use super::*;

use acme_kit::text;

/// Where a weighted segment variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedSegmentVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedSegmentVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted segment variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentVariance {
    pub code: String,
    pub name: String,
    pub source: String,
    pub weight: f64,
    pub sequence: i64,
    pub category: String,
    pub status: WeightedSegmentVarianceStatus,
}

impl WeightedSegmentVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentVarianceStatus) -> bool {
        match self.status {
            WeightedSegmentVarianceStatus::Proposed => {
                match next {
                    WeightedSegmentVarianceStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentVarianceStatus::Approved => {
                match next {
                    WeightedSegmentVarianceStatus::Rejected | WeightedSegmentVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentVarianceStatus::Rejected => {
                return next == WeightedSegmentVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedSegmentVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSegmentVariance {
        WeightedSegmentVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            weight: self.weight,
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted segment variance in status proposed.
pub fn new_weighted_segment_variance(code: &str, name: &str) -> WeightedSegmentVariance {
    WeightedSegmentVariance {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        weight: 0.0,
        sequence: 0,
        category: "".to_string(),
        status: WeightedSegmentVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_segment_variance("code-36", "name-90");
        assert_eq!(item.status, WeightedSegmentVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_segment_variance("code-36", "name-90");
        if item.move_to(WeightedSegmentVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSegmentVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_segment_variance("code-36", "name-90");
        item.status = WeightedSegmentVarianceStatus::Withdrawn;
        assert!(
            item.move_to(WeightedSegmentVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

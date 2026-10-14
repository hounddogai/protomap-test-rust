use super::*;

use acme_kit::text;

/// Where a weighted data export segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedDataExportSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedDataExportSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedDataExportSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted data export segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportSegment {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub ends_at: i64,
    pub cluster: String,
    pub margin: f64,
    pub cost: f64,
    pub status: WeightedDataExportSegmentStatus,
}

impl WeightedDataExportSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedDataExportSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedDataExportSegmentStatus) -> bool {
        match self.status {
            WeightedDataExportSegmentStatus::Idle => {
                match next {
                    WeightedDataExportSegmentStatus::Warming | WeightedDataExportSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDataExportSegmentStatus::Warming => {
                match next {
                    WeightedDataExportSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDataExportSegmentStatus::Ready => {
                match next {
                    WeightedDataExportSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDataExportSegmentStatus::Draining => {
                return next == WeightedDataExportSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedDataExportSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted data export segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.score * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedDataExportSegment {
        WeightedDataExportSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            ends_at: self.ends_at,
            cluster: self.cluster.clone(),
            margin: self.margin,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a weighted data export segment in status idle.
pub fn new_weighted_data_export_segment(code: &str, label: &str) -> WeightedDataExportSegment {
    WeightedDataExportSegment {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        ends_at: 0,
        cluster: "".to_string(),
        margin: 0.0,
        cost: 0.0,
        status: WeightedDataExportSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_data_export_segment("code-597", "label-891");
        assert_eq!(item.status, WeightedDataExportSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_data_export_segment("code-597", "label-891");
        if item.move_to(WeightedDataExportSegmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedDataExportSegmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_data_export_segment("code-597", "label-891");
        item.status = WeightedDataExportSegmentStatus::Stopped;
        assert!(
            item.move_to(WeightedDataExportSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a strict query threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQueryThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictQueryThresholdStatus {
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

impl std::fmt::Display for StrictQueryThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict query threshold as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThreshold {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub locale: String,
    pub quota_units: i64,
    pub starts_at: i64,
    pub floor_value: f64,
    pub weight: f64,
    pub batch_size: i64,
    pub amount: f64,
    pub status: StrictQueryThresholdStatus,
}

impl StrictQueryThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQueryThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQueryThresholdStatus) -> bool {
        match self.status {
            StrictQueryThresholdStatus::Idle => {
                match next {
                    StrictQueryThresholdStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryThresholdStatus::Warming => {
                match next {
                    StrictQueryThresholdStatus::Draining | StrictQueryThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryThresholdStatus::Ready => {
                match next {
                    StrictQueryThresholdStatus::Draining | StrictQueryThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryThresholdStatus::Draining => {
                return next == StrictQueryThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictQueryThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict query threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.weight * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQueryThreshold {
        StrictQueryThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            starts_at: self.starts_at,
            floor_value: self.floor_value,
            weight: self.weight,
            batch_size: self.batch_size,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict query threshold in status idle.
pub fn new_strict_query_threshold(code: &str, label: &str) -> StrictQueryThreshold {
    StrictQueryThreshold {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        locale: "".to_string(),
        quota_units: 0,
        starts_at: 0,
        floor_value: 0.0,
        weight: 0.0,
        batch_size: 0,
        amount: 0.0,
        status: StrictQueryThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_query_threshold("code-972", "label-656");
        assert_eq!(item.status, StrictQueryThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_query_threshold("code-972", "label-656");
        if item.move_to(StrictQueryThresholdStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictQueryThresholdStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_query_threshold("code-972", "label-656");
        item.status = StrictQueryThresholdStatus::Stopped;
        assert!(
            item.move_to(StrictQueryThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

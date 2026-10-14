use super::*;

use acme_kit::text;

/// Where a weighted failover group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFailoverGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedFailoverGroupStatus {
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

impl std::fmt::Display for WeightedFailoverGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted failover group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverGroup {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub title: String,
    pub priority: i64,
    pub source: String,
    pub minimum: f64,
    pub amount: f64,
    pub duration_minutes: i64,
    pub rotation_days: i64,
    pub threshold: f64,
    pub status: WeightedFailoverGroupStatus,
}

impl WeightedFailoverGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFailoverGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFailoverGroupStatus) -> bool {
        match self.status {
            WeightedFailoverGroupStatus::Planned => {
                match next {
                    WeightedFailoverGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverGroupStatus::Scheduled => {
                match next {
                    WeightedFailoverGroupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverGroupStatus::InProgress => {
                match next {
                    WeightedFailoverGroupStatus::Completed | WeightedFailoverGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverGroupStatus::Completed => {
                match next {
                    WeightedFailoverGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedFailoverGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted failover group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.amount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFailoverGroup {
        WeightedFailoverGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            title: self.title.clone(),
            priority: self.priority,
            source: self.source.clone(),
            minimum: self.minimum,
            amount: self.amount,
            duration_minutes: self.duration_minutes,
            rotation_days: self.rotation_days,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted failover group in status planned.
pub fn new_weighted_failover_group(code: &str, label: &str) -> WeightedFailoverGroup {
    WeightedFailoverGroup {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        title: "".to_string(),
        priority: 0,
        source: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        duration_minutes: 0,
        rotation_days: 0,
        threshold: 0.0,
        status: WeightedFailoverGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_failover_group("code-1", "label-629");
        assert_eq!(item.status, WeightedFailoverGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_weighted_failover_group("code-1", "label-629");
        if item.move_to(WeightedFailoverGroupStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFailoverGroupStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_failover_group("code-1", "label-629");
        item.status = WeightedFailoverGroupStatus::Cancelled;
        assert!(
            item.move_to(WeightedFailoverGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

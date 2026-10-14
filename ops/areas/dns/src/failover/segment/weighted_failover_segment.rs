use super::*;

use acme_kit::text;

/// Where a weighted failover segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFailoverSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedFailoverSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedFailoverSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted failover segment as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegment {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub checksum: String,
    pub name: String,
    pub limit: i64,
    pub window_days: i64,
    pub status: WeightedFailoverSegmentStatus,
}

impl WeightedFailoverSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFailoverSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFailoverSegmentStatus) -> bool {
        match self.status {
            WeightedFailoverSegmentStatus::New => {
                match next {
                    WeightedFailoverSegmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverSegmentStatus::Queued => {
                return next == WeightedFailoverSegmentStatus::Failed;
            }
            WeightedFailoverSegmentStatus::Running => {
                return next == WeightedFailoverSegmentStatus::Failed;
            }
            WeightedFailoverSegmentStatus::Done => {
                match next {
                    WeightedFailoverSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedFailoverSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted failover segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.limit as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFailoverSegment {
        WeightedFailoverSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            limit: self.limit,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted failover segment in status new.
pub fn new_weighted_failover_segment(code: &str, title: &str) -> WeightedFailoverSegment {
    WeightedFailoverSegment {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        limit: 0,
        window_days: 0,
        status: WeightedFailoverSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_failover_segment("code-85", "title-5");
        assert_eq!(item.status, WeightedFailoverSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_weighted_failover_segment("code-85", "title-5");
        if item.move_to(WeightedFailoverSegmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFailoverSegmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_failover_segment("code-85", "title-5");
        item.status = WeightedFailoverSegmentStatus::Failed;
        assert!(
            item.move_to(WeightedFailoverSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

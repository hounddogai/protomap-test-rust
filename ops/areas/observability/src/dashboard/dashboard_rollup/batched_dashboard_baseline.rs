use super::*;

use acme_kit::text;

/// Where a batched dashboard baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDashboardBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedDashboardBaselineStatus {
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

impl std::fmt::Display for BatchedDashboardBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched dashboard baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDashboardBaseline {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub key_id: String,
    pub visible: bool,
    pub region: String,
    pub cluster: String,
    pub ends_at: i64,
    pub margin: f64,
    pub maximum: f64,
    pub status: BatchedDashboardBaselineStatus,
}

impl BatchedDashboardBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDashboardBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDashboardBaselineStatus) -> bool {
        match self.status {
            BatchedDashboardBaselineStatus::New => {
                match next {
                    BatchedDashboardBaselineStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDashboardBaselineStatus::Queued => {
                match next {
                    BatchedDashboardBaselineStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDashboardBaselineStatus::Running => {
                match next {
                    BatchedDashboardBaselineStatus::Done | BatchedDashboardBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDashboardBaselineStatus::Done => {
                match next {
                    BatchedDashboardBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedDashboardBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched dashboard baseline {} cannot move from {} to {}",
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

    pub fn effective_value(&self) -> f64 {
        ((self.margin + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDashboardBaseline {
        BatchedDashboardBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            key_id: self.key_id.clone(),
            visible: self.visible,
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            ends_at: self.ends_at,
            margin: self.margin,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched dashboard baseline in status new.
pub fn new_batched_dashboard_baseline(code: &str, label: &str) -> BatchedDashboardBaseline {
    BatchedDashboardBaseline {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        key_id: "".to_string(),
        visible: false,
        region: "".to_string(),
        cluster: "".to_string(),
        ends_at: 0,
        margin: 0.0,
        maximum: 0.0,
        status: BatchedDashboardBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_dashboard_baseline("code-183", "label-245");
        assert_eq!(item.status, BatchedDashboardBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_dashboard_baseline("code-183", "label-245");
        if item.move_to(BatchedDashboardBaselineStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDashboardBaselineStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_dashboard_baseline("code-183", "label-245");
        item.status = BatchedDashboardBaselineStatus::Failed;
        assert!(
            item.move_to(BatchedDashboardBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

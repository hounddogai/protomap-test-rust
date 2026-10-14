use super::*;

use acme_kit::text;

/// Where a seasonal dashboard snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDashboardSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalDashboardSnapshotStatus {
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

impl std::fmt::Display for SeasonalDashboardSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal dashboard snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardSnapshot {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub ends_at: i64,
    pub sequence: i64,
    pub maximum: f64,
    pub pinned: bool,
    pub cluster: String,
    pub cost: f64,
    pub currency: String,
    pub status: SeasonalDashboardSnapshotStatus,
}

impl SeasonalDashboardSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDashboardSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDashboardSnapshotStatus) -> bool {
        match self.status {
            SeasonalDashboardSnapshotStatus::Idle => {
                return next == SeasonalDashboardSnapshotStatus::Draining;
            }
            SeasonalDashboardSnapshotStatus::Warming => {
                match next {
                    SeasonalDashboardSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDashboardSnapshotStatus::Ready => {
                return next == SeasonalDashboardSnapshotStatus::Draining
                    || next == SeasonalDashboardSnapshotStatus::Stopped;
            }
            SeasonalDashboardSnapshotStatus::Draining => {
                return next == SeasonalDashboardSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDashboardSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dashboard snapshot {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.maximum - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDashboardSnapshot {
        SeasonalDashboardSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            ends_at: self.ends_at,
            sequence: self.sequence,
            maximum: self.maximum,
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            cost: self.cost,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal dashboard snapshot in status idle.
pub fn new_seasonal_dashboard_snapshot(code: &str, label: &str) -> SeasonalDashboardSnapshot {
    SeasonalDashboardSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        ends_at: 0,
        sequence: 0,
        maximum: 0.0,
        pinned: false,
        cluster: "".to_string(),
        cost: 0.0,
        currency: "".to_string(),
        status: SeasonalDashboardSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_dashboard_snapshot("code-231", "label-564");
        assert_eq!(item.status, SeasonalDashboardSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_dashboard_snapshot("code-231", "label-564");
        if item.move_to(SeasonalDashboardSnapshotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDashboardSnapshotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_dashboard_snapshot("code-231", "label-564");
        item.status = SeasonalDashboardSnapshotStatus::Stopped;
        assert!(
            item.move_to(SeasonalDashboardSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

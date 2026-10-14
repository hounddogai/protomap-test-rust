use super::*;

use acme_kit::text;

/// Where a rolling slo batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSloBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingSloBatchStatus {
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

impl std::fmt::Display for RollingSloBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling slo batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloBatch {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub summary: String,
    pub tags: Vec<String>,
    pub title: String,
    pub status: RollingSloBatchStatus,
}

impl RollingSloBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSloBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSloBatchStatus) -> bool {
        match self.status {
            RollingSloBatchStatus::Idle => {
                match next {
                    RollingSloBatchStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSloBatchStatus::Warming => {
                return next == RollingSloBatchStatus::Stopped;
            }
            RollingSloBatchStatus::Ready => {
                match next {
                    RollingSloBatchStatus::Draining | RollingSloBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSloBatchStatus::Draining => {
                match next {
                    RollingSloBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingSloBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling slo batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> RollingSloBatch {
        RollingSloBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling slo batch in status idle.
pub fn new_rolling_slo_batch(code: &str, name: &str) -> RollingSloBatch {
    RollingSloBatch {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        summary: "".to_string(),
        tags: Vec::new(),
        title: "".to_string(),
        status: RollingSloBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_slo_batch("code-319", "name-591");
        assert_eq!(item.status, RollingSloBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rolling_slo_batch("code-319", "name-591");
        if item.move_to(RollingSloBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSloBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_slo_batch("code-319", "name-591");
        item.status = RollingSloBatchStatus::Stopped;
        assert!(
            item.move_to(RollingSloBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

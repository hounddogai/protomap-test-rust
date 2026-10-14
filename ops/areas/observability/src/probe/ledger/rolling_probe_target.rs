use super::*;

use acme_kit::text;

/// Where a rolling probe target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingProbeTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingProbeTargetStatus {
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

impl std::fmt::Display for RollingProbeTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling probe target as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeTarget {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub active: bool,
    pub port: i64,
    pub cost: f64,
    pub description: String,
    pub maximum: f64,
    pub status: RollingProbeTargetStatus,
}

impl RollingProbeTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingProbeTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingProbeTargetStatus) -> bool {
        match self.status {
            RollingProbeTargetStatus::New => {
                return next == RollingProbeTargetStatus::Running;
            }
            RollingProbeTargetStatus::Queued => {
                match next {
                    RollingProbeTargetStatus::Running | RollingProbeTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeTargetStatus::Running => {
                return next == RollingProbeTargetStatus::Failed;
            }
            RollingProbeTargetStatus::Done => {
                match next {
                    RollingProbeTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingProbeTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling probe target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling probe target", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.maximum * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingProbeTarget {
        RollingProbeTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            active: self.active,
            port: self.port,
            cost: self.cost,
            description: self.description.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling probe target in status new.
pub fn new_rolling_probe_target(code: &str, title: &str) -> RollingProbeTarget {
    RollingProbeTarget {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        active: false,
        port: 0,
        cost: 0.0,
        description: "".to_string(),
        maximum: 0.0,
        status: RollingProbeTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_probe_target("code-516", "title-486");
        assert_eq!(item.status, RollingProbeTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_probe_target("code-516", "title-486");
        if item.move_to(RollingProbeTargetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingProbeTargetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_probe_target("code-516", "title-486");
        item.status = RollingProbeTargetStatus::Failed;
        assert!(
            item.move_to(RollingProbeTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

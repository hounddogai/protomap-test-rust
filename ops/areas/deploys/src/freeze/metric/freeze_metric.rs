use super::*;

use acme_kit::text;

/// Where a freeze metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeMetricStatus {
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

impl std::fmt::Display for FreezeMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze metric as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMetric {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub fingerprint: String,
    pub approved: bool,
    pub source: String,
    pub visible: bool,
    pub owner_id: String,
    pub status: FreezeMetricStatus,
}

impl FreezeMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeMetricStatus) -> bool {
        match self.status {
            FreezeMetricStatus::New => {
                return next == FreezeMetricStatus::Queued || next == FreezeMetricStatus::Done;
            }
            FreezeMetricStatus::Queued => {
                match next {
                    FreezeMetricStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeMetricStatus::Running => {
                return next == FreezeMetricStatus::Done || next == FreezeMetricStatus::Failed;
            }
            FreezeMetricStatus::Done => {
                match next {
                    FreezeMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: FreezeMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> FreezeMetric {
        FreezeMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            source: self.source.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a freeze metric in status new.
pub fn new_freeze_metric(code: &str, name: &str) -> FreezeMetric {
    FreezeMetric {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        fingerprint: "".to_string(),
        approved: false,
        source: "".to_string(),
        visible: false,
        owner_id: "".to_string(),
        status: FreezeMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_metric("code-701", "name-841");
        assert_eq!(item.status, FreezeMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_freeze_metric("code-701", "name-841");
        if item.move_to(FreezeMetricStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeMetricStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_metric("code-701", "name-841");
        item.status = FreezeMetricStatus::Failed;
        assert!(item.move_to(FreezeMetricStatus::New).is_err(), "expected move to fail");
    }
}

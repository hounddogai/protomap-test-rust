use super::*;

use acme_kit::text;

/// Where a global alert rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAlertRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalAlertRollupStatus {
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

impl std::fmt::Display for GlobalAlertRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global alert rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertRollup {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub priority: i64,
    pub version: i64,
    pub quantity: i64,
    pub limit: i64,
    pub expires_at: i64,
    pub threshold: f64,
    pub ttl_seconds: i64,
    pub currency: String,
    pub status: GlobalAlertRollupStatus,
}

impl GlobalAlertRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAlertRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAlertRollupStatus) -> bool {
        match self.status {
            GlobalAlertRollupStatus::New => {
                return next == GlobalAlertRollupStatus::Running || next == GlobalAlertRollupStatus::Done;
            }
            GlobalAlertRollupStatus::Queued => {
                return next == GlobalAlertRollupStatus::Running || next == GlobalAlertRollupStatus::Failed;
            }
            GlobalAlertRollupStatus::Running => {
                match next {
                    GlobalAlertRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAlertRollupStatus::Done => {
                return next == GlobalAlertRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalAlertRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global alert rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.limit as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAlertRollup {
        GlobalAlertRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            priority: self.priority,
            version: self.version,
            quantity: self.quantity,
            limit: self.limit,
            expires_at: self.expires_at,
            threshold: self.threshold,
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global alert rollup in status new.
pub fn new_global_alert_rollup(code: &str, title: &str) -> GlobalAlertRollup {
    GlobalAlertRollup {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        priority: 0,
        version: 0,
        quantity: 0,
        limit: 0,
        expires_at: 0,
        threshold: 0.0,
        ttl_seconds: 0,
        currency: "".to_string(),
        status: GlobalAlertRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_alert_rollup("code-985", "title-473");
        assert_eq!(item.status, GlobalAlertRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_global_alert_rollup("code-985", "title-473");
        if item.move_to(GlobalAlertRollupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAlertRollupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_alert_rollup("code-985", "title-473");
        item.status = GlobalAlertRollupStatus::Failed;
        assert!(
            item.move_to(GlobalAlertRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

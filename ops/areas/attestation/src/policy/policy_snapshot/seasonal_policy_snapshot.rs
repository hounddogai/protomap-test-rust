use super::*;

use acme_kit::text;

/// Where a seasonal policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPolicySnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalPolicySnapshotStatus {
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

impl std::fmt::Display for SeasonalPolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal policy snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshot {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub label: String,
    pub created_at: i64,
    pub retries: i64,
    pub description: String,
    pub revision: i64,
    pub status: SeasonalPolicySnapshotStatus,
}

impl SeasonalPolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPolicySnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPolicySnapshotStatus) -> bool {
        match self.status {
            SeasonalPolicySnapshotStatus::Idle => {
                match next {
                    SeasonalPolicySnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPolicySnapshotStatus::Warming => {
                match next {
                    SeasonalPolicySnapshotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPolicySnapshotStatus::Ready => {
                return next == SeasonalPolicySnapshotStatus::Draining || next == SeasonalPolicySnapshotStatus::Stopped;
            }
            SeasonalPolicySnapshotStatus::Draining => {
                return next == SeasonalPolicySnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalPolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPolicySnapshot {
        SeasonalPolicySnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            label: self.label.clone(),
            created_at: self.created_at,
            retries: self.retries,
            description: self.description.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal policy snapshot in status idle.
pub fn new_seasonal_policy_snapshot(code: &str, name: &str) -> SeasonalPolicySnapshot {
    SeasonalPolicySnapshot {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        label: "".to_string(),
        created_at: 0,
        retries: 0,
        description: "".to_string(),
        revision: 0,
        status: SeasonalPolicySnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_policy_snapshot("code-974", "name-960");
        assert_eq!(item.status, SeasonalPolicySnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_seasonal_policy_snapshot("code-974", "name-960");
        if item.move_to(SeasonalPolicySnapshotStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPolicySnapshotStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_policy_snapshot("code-974", "name-960");
        item.status = SeasonalPolicySnapshotStatus::Stopped;
        assert!(
            item.move_to(SeasonalPolicySnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

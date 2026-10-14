use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional usage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalUsageSnapshotStatus {
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

impl std::fmt::Display for RegionalUsageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional usage snapshot as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshot {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub fingerprint: String,
    pub audience: String,
    pub cipher: String,
    pub threshold: f64,
    pub scope: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub window_days: i64,
    pub status: RegionalUsageSnapshotStatus,
}

impl RegionalUsageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageSnapshotStatus) -> bool {
        match self.status {
            RegionalUsageSnapshotStatus::New => {
                match next {
                    RegionalUsageSnapshotStatus::Running | RegionalUsageSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageSnapshotStatus::Queued => {
                return next == RegionalUsageSnapshotStatus::Done || next == RegionalUsageSnapshotStatus::Failed;
            }
            RegionalUsageSnapshotStatus::Running => {
                return next == RegionalUsageSnapshotStatus::Done;
            }
            RegionalUsageSnapshotStatus::Done => {
                return next == RegionalUsageSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalUsageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional usage snapshot", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_value(&self) -> f64 {
        (self.amount * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageSnapshot {
        RegionalUsageSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            cipher: self.cipher.clone(),
            threshold: self.threshold,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional usage snapshot in status new.
pub fn new_regional_usage_snapshot(code: &str, name: &str) -> RegionalUsageSnapshot {
    RegionalUsageSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        cipher: "".to_string(),
        threshold: 0.0,
        scope: "".to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        window_days: 0,
        status: RegionalUsageSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_usage_snapshot("code-678", "name-181");
        assert_eq!(item.status, RegionalUsageSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_usage_snapshot("code-678", "name-181");
        if item.move_to(RegionalUsageSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_usage_snapshot("code-678", "name-181");
        item.status = RegionalUsageSnapshotStatus::Failed;
        assert!(
            item.move_to(RegionalUsageSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

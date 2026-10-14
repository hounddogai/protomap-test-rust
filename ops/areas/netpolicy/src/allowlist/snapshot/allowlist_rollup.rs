use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a allowlist rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistRollupStatus {
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

impl std::fmt::Display for AllowlistRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollup {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub floor_value: f64,
    pub version: i64,
    pub window_days: i64,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: AllowlistRollupStatus,
}

impl AllowlistRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistRollupStatus) -> bool {
        match self.status {
            AllowlistRollupStatus::New => {
                return next == AllowlistRollupStatus::Running || next == AllowlistRollupStatus::Failed;
            }
            AllowlistRollupStatus::Queued => {
                match next {
                    AllowlistRollupStatus::Running | AllowlistRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistRollupStatus::Running => {
                match next {
                    AllowlistRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistRollupStatus::Done => {
                return next == AllowlistRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn net_load(&self) -> f64 {
        ((self.version as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistRollup {
        AllowlistRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            floor_value: self.floor_value,
            version: self.version,
            window_days: self.window_days,
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist rollup in status new.
pub fn new_allowlist_rollup(code: &str, name: &str) -> AllowlistRollup {
    AllowlistRollup {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        floor_value: 0.0,
        version: 0,
        window_days: 0,
        capacity: 0,
        attributes: BTreeMap::new(),
        status: AllowlistRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_allowlist_rollup("code-164", "name-864");
        assert_eq!(item.status, AllowlistRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_allowlist_rollup("code-164", "name-864");
        if item.move_to(AllowlistRollupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistRollupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_allowlist_rollup("code-164", "name-864");
        item.status = AllowlistRollupStatus::Failed;
        assert!(
            item.move_to(AllowlistRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

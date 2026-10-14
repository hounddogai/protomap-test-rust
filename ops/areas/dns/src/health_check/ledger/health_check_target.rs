use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a health check target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckTargetStatus {
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

impl std::fmt::Display for HealthCheckTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTarget {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub status: HealthCheckTargetStatus,
}

impl HealthCheckTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckTargetStatus) -> bool {
        match self.status {
            HealthCheckTargetStatus::New => {
                return next == HealthCheckTargetStatus::Queued || next == HealthCheckTargetStatus::Running;
            }
            HealthCheckTargetStatus::Queued => {
                match next {
                    HealthCheckTargetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckTargetStatus::Running => {
                return next == HealthCheckTargetStatus::Done;
            }
            HealthCheckTargetStatus::Done => {
                match next {
                    HealthCheckTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: HealthCheckTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> HealthCheckTarget {
        HealthCheckTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a health check target in status new.
pub fn new_health_check_target(code: &str, title: &str) -> HealthCheckTarget {
    HealthCheckTarget {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        status: HealthCheckTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_health_check_target("code-123", "title-775");
        assert_eq!(item.status, HealthCheckTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_health_check_target("code-123", "title-775");
        if item.move_to(HealthCheckTargetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckTargetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_health_check_target("code-123", "title-775");
        item.status = HealthCheckTargetStatus::Failed;
        assert!(
            item.move_to(HealthCheckTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

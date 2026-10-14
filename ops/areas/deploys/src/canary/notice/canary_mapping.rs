use super::*;

use acme_kit::text;

/// Where a canary mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryMappingStatus {
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

impl std::fmt::Display for CanaryMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary mapping as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMapping {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub description: String,
    pub grace_days: i64,
    pub duration_minutes: i64,
    pub principal: String,
    pub status: CanaryMappingStatus,
}

impl CanaryMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryMappingStatus) -> bool {
        match self.status {
            CanaryMappingStatus::New => {
                return next == CanaryMappingStatus::Running;
            }
            CanaryMappingStatus::Queued => {
                return next == CanaryMappingStatus::Running || next == CanaryMappingStatus::Failed;
            }
            CanaryMappingStatus::Running => {
                return next == CanaryMappingStatus::Done || next == CanaryMappingStatus::Failed;
            }
            CanaryMappingStatus::Done => {
                return next == CanaryMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryMapping {
        CanaryMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            grace_days: self.grace_days,
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary mapping in status new.
pub fn new_canary_mapping(code: &str, name: &str) -> CanaryMapping {
    CanaryMapping {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        grace_days: 0,
        duration_minutes: 0,
        principal: "".to_string(),
        status: CanaryMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_canary_mapping("code-401", "name-492");
        assert_eq!(item.status, CanaryMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_canary_mapping("code-401", "name-492");
        if item.move_to(CanaryMappingStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryMappingStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_canary_mapping("code-401", "name-492");
        item.status = CanaryMappingStatus::Failed;
        assert!(item.move_to(CanaryMappingStatus::New).is_err(), "expected move to fail");
    }
}

use super::*;

use acme_kit::text;

/// Where a global binding target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalBindingTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalBindingTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalBindingTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global binding target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTarget {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub country: String,
    pub fingerprint: String,
    pub host: String,
    pub limit: i64,
    pub quota_units: i64,
    pub status: GlobalBindingTargetStatus,
}

impl GlobalBindingTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalBindingTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalBindingTargetStatus) -> bool {
        match self.status {
            GlobalBindingTargetStatus::Planned => {
                return next == GlobalBindingTargetStatus::InProgress;
            }
            GlobalBindingTargetStatus::Scheduled => {
                match next {
                    GlobalBindingTargetStatus::InProgress | GlobalBindingTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBindingTargetStatus::InProgress => {
                return next == GlobalBindingTargetStatus::Completed;
            }
            GlobalBindingTargetStatus::Completed => {
                match next {
                    GlobalBindingTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalBindingTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global binding target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.quota_units as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalBindingTarget {
        GlobalBindingTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global binding target in status planned.
pub fn new_global_binding_target(code: &str, title: &str) -> GlobalBindingTarget {
    GlobalBindingTarget {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        country: "".to_string(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        limit: 0,
        quota_units: 0,
        status: GlobalBindingTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_binding_target("code-47", "title-222");
        assert_eq!(item.status, GlobalBindingTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_binding_target("code-47", "title-222");
        if item.move_to(GlobalBindingTargetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalBindingTargetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_binding_target("code-47", "title-222");
        item.status = GlobalBindingTargetStatus::Cancelled;
        assert!(
            item.move_to(GlobalBindingTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

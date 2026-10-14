use super::*;

use acme_kit::text;

/// Where a usage override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl UsageOverrideStatus {
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

impl std::fmt::Display for UsageOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage override as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverride {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub duration_minutes: i64,
    pub ends_at: i64,
    pub category: String,
    pub locale: String,
    pub starts_at: i64,
    pub approved: bool,
    pub namespace: String,
    pub status: UsageOverrideStatus,
}

impl UsageOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageOverrideStatus) -> bool {
        match self.status {
            UsageOverrideStatus::New => {
                match next {
                    UsageOverrideStatus::Running | UsageOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageOverrideStatus::Queued => {
                return next == UsageOverrideStatus::Done;
            }
            UsageOverrideStatus::Running => {
                match next {
                    UsageOverrideStatus::Done | UsageOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageOverrideStatus::Done => {
                return next == UsageOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageOverride {
        UsageOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            duration_minutes: self.duration_minutes,
            ends_at: self.ends_at,
            category: self.category.clone(),
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a usage override in status new.
pub fn new_usage_override(code: &str, label: &str) -> UsageOverride {
    UsageOverride {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        duration_minutes: 0,
        ends_at: 0,
        category: "".to_string(),
        locale: "".to_string(),
        starts_at: 0,
        approved: false,
        namespace: "".to_string(),
        status: UsageOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_usage_override("code-358", "label-727");
        assert_eq!(item.status, UsageOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_usage_override("code-358", "label-727");
        if item.move_to(UsageOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_usage_override("code-358", "label-727");
        item.status = UsageOverrideStatus::Failed;
        assert!(item.move_to(UsageOverrideStatus::New).is_err(), "expected move to fail");
    }
}

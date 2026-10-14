use super::*;

use acme_kit::text;

/// Where a alert profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AlertProfileStatus {
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

impl std::fmt::Display for AlertProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfile {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub budget: f64,
    pub quantity: i64,
    pub principal: String,
    pub status: AlertProfileStatus,
}

impl AlertProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertProfileStatus) -> bool {
        match self.status {
            AlertProfileStatus::New => {
                return next == AlertProfileStatus::Running || next == AlertProfileStatus::Failed;
            }
            AlertProfileStatus::Queued => {
                return next == AlertProfileStatus::Done || next == AlertProfileStatus::Failed;
            }
            AlertProfileStatus::Running => {
                match next {
                    AlertProfileStatus::Done | AlertProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertProfileStatus::Done => {
                return next == AlertProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertProfile {
        AlertProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            budget: self.budget,
            quantity: self.quantity,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a alert profile in status new.
pub fn new_alert_profile(code: &str, name: &str) -> AlertProfile {
    AlertProfile {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        budget: 0.0,
        quantity: 0,
        principal: "".to_string(),
        status: AlertProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_alert_profile("code-17", "name-71");
        assert_eq!(item.status, AlertProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_alert_profile("code-17", "name-71");
        if item.move_to(AlertProfileStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertProfileStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_alert_profile("code-17", "name-71");
        item.status = AlertProfileStatus::Failed;
        assert!(item.move_to(AlertProfileStatus::New).is_err(), "expected move to fail");
    }
}

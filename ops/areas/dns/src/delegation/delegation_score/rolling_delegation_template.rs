use super::*;

use acme_kit::text;

/// Where a rolling delegation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDelegationTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingDelegationTemplateStatus {
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

impl std::fmt::Display for RollingDelegationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling delegation template as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationTemplate {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub timezone: String,
    pub audience: String,
    pub ends_at: i64,
    pub capacity: i64,
    pub status: RollingDelegationTemplateStatus,
}

impl RollingDelegationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDelegationTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDelegationTemplateStatus) -> bool {
        match self.status {
            RollingDelegationTemplateStatus::Planned => {
                return next == RollingDelegationTemplateStatus::Scheduled;
            }
            RollingDelegationTemplateStatus::Scheduled => {
                match next {
                    RollingDelegationTemplateStatus::InProgress | RollingDelegationTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDelegationTemplateStatus::InProgress => {
                return next == RollingDelegationTemplateStatus::Cancelled;
            }
            RollingDelegationTemplateStatus::Completed => {
                return next == RollingDelegationTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDelegationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling delegation template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDelegationTemplate {
        RollingDelegationTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling delegation template in status planned.
pub fn new_rolling_delegation_template(code: &str, name: &str) -> RollingDelegationTemplate {
    RollingDelegationTemplate {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        timezone: "".to_string(),
        audience: "".to_string(),
        ends_at: 0,
        capacity: 0,
        status: RollingDelegationTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_delegation_template("code-676", "name-64");
        assert_eq!(item.status, RollingDelegationTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_delegation_template("code-676", "name-64");
        if item.move_to(RollingDelegationTemplateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDelegationTemplateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_delegation_template("code-676", "name-64");
        item.status = RollingDelegationTemplateStatus::Cancelled;
        assert!(
            item.move_to(RollingDelegationTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

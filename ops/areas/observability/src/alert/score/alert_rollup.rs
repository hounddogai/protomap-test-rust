use super::*;

use acme_kit::text;

/// Where a alert rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertRollupStatus {
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

impl std::fmt::Display for AlertRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert rollup as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollup {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub summary: String,
    pub contact_email: String,
    pub principal: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub ends_at: i64,
    pub status: AlertRollupStatus,
}

impl AlertRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertRollupStatus) -> bool {
        match self.status {
            AlertRollupStatus::Planned => {
                match next {
                    AlertRollupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertRollupStatus::Scheduled => {
                match next {
                    AlertRollupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertRollupStatus::InProgress => {
                return next == AlertRollupStatus::Cancelled;
            }
            AlertRollupStatus::Completed => {
                match next {
                    AlertRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AlertRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertRollup {
        AlertRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a alert rollup in status planned.
pub fn new_alert_rollup(code: &str, title: &str) -> AlertRollup {
    AlertRollup {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        summary: "".to_string(),
        contact_email: "".to_string(),
        principal: "".to_string(),
        sequence: 0,
        expires_at: 0,
        ends_at: 0,
        status: AlertRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_alert_rollup("code-930", "title-328");
        assert_eq!(item.status, AlertRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_alert_rollup("code-930", "title-328");
        if item.move_to(AlertRollupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertRollupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_alert_rollup("code-930", "title-328");
        item.status = AlertRollupStatus::Cancelled;
        assert!(
            item.move_to(AlertRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a alert baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AlertBaselineStatus {
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

impl std::fmt::Display for AlertBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert baseline as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBaseline {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub rotation_days: i64,
    pub approved: bool,
    pub expires_at: i64,
    pub version: i64,
    pub status: AlertBaselineStatus,
}

impl AlertBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertBaselineStatus) -> bool {
        match self.status {
            AlertBaselineStatus::New => {
                return next == AlertBaselineStatus::Queued || next == AlertBaselineStatus::Running;
            }
            AlertBaselineStatus::Queued => {
                return next == AlertBaselineStatus::Done;
            }
            AlertBaselineStatus::Running => {
                return next == AlertBaselineStatus::Done || next == AlertBaselineStatus::Failed;
            }
            AlertBaselineStatus::Done => {
                match next {
                    AlertBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: AlertBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertBaseline {
        AlertBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            approved: self.approved,
            expires_at: self.expires_at,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a alert baseline in status new.
pub fn new_alert_baseline(code: &str, title: &str) -> AlertBaseline {
    AlertBaseline {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        rotation_days: 0,
        approved: false,
        expires_at: 0,
        version: 0,
        status: AlertBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_alert_baseline("code-975", "title-632");
        assert_eq!(item.status, AlertBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_alert_baseline("code-975", "title-632");
        if item.move_to(AlertBaselineStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertBaselineStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_alert_baseline("code-975", "title-632");
        item.status = AlertBaselineStatus::Failed;
        assert!(item.move_to(AlertBaselineStatus::New).is_err(), "expected move to fail");
    }
}

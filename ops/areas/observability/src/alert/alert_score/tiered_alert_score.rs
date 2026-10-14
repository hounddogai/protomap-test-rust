use super::*;

use acme_kit::text;

/// Where a tiered alert score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredAlertScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredAlertScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered alert score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertScore {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub weight: f64,
    pub quota_units: i64,
    pub description: String,
    pub margin: f64,
    pub tenant_id: String,
    pub summary: String,
    pub locale: String,
    pub status: TieredAlertScoreStatus,
}

impl TieredAlertScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertScoreStatus) -> bool {
        match self.status {
            TieredAlertScoreStatus::Idle => {
                return next == TieredAlertScoreStatus::Ready;
            }
            TieredAlertScoreStatus::Warming => {
                return next == TieredAlertScoreStatus::Stopped;
            }
            TieredAlertScoreStatus::Ready => {
                match next {
                    TieredAlertScoreStatus::Draining | TieredAlertScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertScoreStatus::Draining => {
                return next == TieredAlertScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredAlertScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered alert score", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quota_units as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAlertScore {
        TieredAlertScore {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            weight: self.weight,
            quota_units: self.quota_units,
            description: self.description.clone(),
            margin: self.margin,
            tenant_id: self.tenant_id.clone(),
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered alert score in status idle.
pub fn new_tiered_alert_score(code: &str, title: &str) -> TieredAlertScore {
    TieredAlertScore {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        weight: 0.0,
        quota_units: 0,
        description: "".to_string(),
        margin: 0.0,
        tenant_id: "".to_string(),
        summary: "".to_string(),
        locale: "".to_string(),
        status: TieredAlertScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_alert_score("code-364", "title-638");
        assert_eq!(item.status, TieredAlertScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tiered_alert_score("code-364", "title-638");
        if item.move_to(TieredAlertScoreStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAlertScoreStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_alert_score("code-364", "title-638");
        item.status = TieredAlertScoreStatus::Stopped;
        assert!(
            item.move_to(TieredAlertScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

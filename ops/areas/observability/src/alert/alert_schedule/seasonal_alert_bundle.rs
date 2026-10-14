use super::*;

use acme_kit::text;

/// Where a seasonal alert bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAlertBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalAlertBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalAlertBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal alert bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertBundle {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub expires_at: i64,
    pub score: f64,
    pub status: SeasonalAlertBundleStatus,
}

impl SeasonalAlertBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAlertBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAlertBundleStatus) -> bool {
        match self.status {
            SeasonalAlertBundleStatus::Requested => {
                return next == SeasonalAlertBundleStatus::Granted || next == SeasonalAlertBundleStatus::Revoked;
            }
            SeasonalAlertBundleStatus::Granted => {
                return next == SeasonalAlertBundleStatus::Revoked;
            }
            SeasonalAlertBundleStatus::Revoked => {
                match next {
                    SeasonalAlertBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalAlertBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal alert bundle {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.expires_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAlertBundle {
        SeasonalAlertBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            expires_at: self.expires_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal alert bundle in status requested.
pub fn new_seasonal_alert_bundle(code: &str, name: &str) -> SeasonalAlertBundle {
    SeasonalAlertBundle {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        expires_at: 0,
        score: 0.0,
        status: SeasonalAlertBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_alert_bundle("code-775", "name-931");
        assert_eq!(item.status, SeasonalAlertBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_alert_bundle("code-775", "name-931");
        if item.move_to(SeasonalAlertBundleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAlertBundleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_alert_bundle("code-775", "name-931");
        item.status = SeasonalAlertBundleStatus::Expired;
        assert!(
            item.move_to(SeasonalAlertBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

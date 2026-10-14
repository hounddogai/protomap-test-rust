use super::*;

use acme_kit::text;

/// Where a pending allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAllowlistForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingAllowlistForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingAllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending allowlist forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistForecast {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub archived: bool,
    pub label: String,
    pub source: String,
    pub tenant_id: String,
    pub category: String,
    pub status: PendingAllowlistForecastStatus,
}

impl PendingAllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAllowlistForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAllowlistForecastStatus) -> bool {
        match self.status {
            PendingAllowlistForecastStatus::Requested => {
                match next {
                    PendingAllowlistForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAllowlistForecastStatus::Granted => {
                match next {
                    PendingAllowlistForecastStatus::Revoked | PendingAllowlistForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAllowlistForecastStatus::Revoked => {
                return next == PendingAllowlistForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingAllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending allowlist forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PendingAllowlistForecast {
        PendingAllowlistForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            archived: self.archived,
            label: self.label.clone(),
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending allowlist forecast in status requested.
pub fn new_pending_allowlist_forecast(code: &str, name: &str) -> PendingAllowlistForecast {
    PendingAllowlistForecast {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        archived: false,
        label: "".to_string(),
        source: "".to_string(),
        tenant_id: "".to_string(),
        category: "".to_string(),
        status: PendingAllowlistForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_allowlist_forecast("code-478", "name-205");
        assert_eq!(item.status, PendingAllowlistForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_allowlist_forecast("code-478", "name-205");
        if item.move_to(PendingAllowlistForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingAllowlistForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_allowlist_forecast("code-478", "name-205");
        item.status = PendingAllowlistForecastStatus::Expired;
        assert!(
            item.move_to(PendingAllowlistForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

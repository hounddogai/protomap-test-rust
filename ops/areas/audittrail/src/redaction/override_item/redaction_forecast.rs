use super::*;

use acme_kit::text;

/// Where a redaction forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RedactionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecast {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub category: String,
    pub updated_at: i64,
    pub status: RedactionForecastStatus,
}

impl RedactionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionForecastStatus) -> bool {
        match self.status {
            RedactionForecastStatus::Draft => {
                return next == RedactionForecastStatus::Pending;
            }
            RedactionForecastStatus::Pending => {
                match next {
                    RedactionForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionForecastStatus::Active => {
                return next == RedactionForecastStatus::Suspended || next == RedactionForecastStatus::Closed;
            }
            RedactionForecastStatus::Suspended => {
                match next {
                    RedactionForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: RedactionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction forecast", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionForecast {
        RedactionForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            category: self.category.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction forecast in status draft.
pub fn new_redaction_forecast(code: &str, name: &str) -> RedactionForecast {
    RedactionForecast {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        category: "".to_string(),
        updated_at: 0,
        status: RedactionForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_redaction_forecast("code-671", "name-645");
        assert_eq!(item.status, RedactionForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_redaction_forecast("code-671", "name-645");
        if item.move_to(RedactionForecastStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionForecastStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_redaction_forecast("code-671", "name-645");
        item.status = RedactionForecastStatus::Closed;
        assert!(
            item.move_to(RedactionForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

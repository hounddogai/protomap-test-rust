use super::*;

use acme_kit::text;

/// Where a binding forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingForecastStatus {
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

impl std::fmt::Display for BindingForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingForecast {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub created_at: i64,
    pub host: String,
    pub duration_minutes: i64,
    pub limit: i64,
    pub sequence: i64,
    pub status: BindingForecastStatus,
}

impl BindingForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingForecastStatus) -> bool {
        match self.status {
            BindingForecastStatus::Draft => {
                match next {
                    BindingForecastStatus::Active | BindingForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingForecastStatus::Pending => {
                match next {
                    BindingForecastStatus::Suspended | BindingForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingForecastStatus::Active => {
                match next {
                    BindingForecastStatus::Suspended | BindingForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingForecastStatus::Suspended => {
                match next {
                    BindingForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: BindingForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingForecast {
        BindingForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding forecast in status draft.
pub fn new_binding_forecast(code: &str, name: &str) -> BindingForecast {
    BindingForecast {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        created_at: 0,
        host: "".to_string(),
        duration_minutes: 0,
        limit: 0,
        sequence: 0,
        status: BindingForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_forecast("code-805", "name-798");
        assert_eq!(item.status, BindingForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_binding_forecast("code-805", "name-798");
        if item.move_to(BindingForecastStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingForecastStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_forecast("code-805", "name-798");
        item.status = BindingForecastStatus::Closed;
        assert!(
            item.move_to(BindingForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

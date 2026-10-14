use super::*;

use acme_kit::text;

/// Where a forecast exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastExceptionStatus {
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

impl std::fmt::Display for ForecastExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastException {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub external_ref: String,
    pub locale: String,
    pub score: f64,
    pub status: ForecastExceptionStatus,
}

impl ForecastException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastExceptionStatus) -> bool {
        match self.status {
            ForecastExceptionStatus::Draft => {
                return next == ForecastExceptionStatus::Suspended || next == ForecastExceptionStatus::Closed;
            }
            ForecastExceptionStatus::Pending => {
                match next {
                    ForecastExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastExceptionStatus::Active => {
                return next == ForecastExceptionStatus::Closed;
            }
            ForecastExceptionStatus::Suspended => {
                return next == ForecastExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "forecast exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ForecastException {
        ForecastException {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a forecast exception in status draft.
pub fn new_forecast_exception(code: &str, name: &str) -> ForecastException {
    ForecastException {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        external_ref: "".to_string(),
        locale: "".to_string(),
        score: 0.0,
        status: ForecastExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_exception("code-242", "name-636");
        assert_eq!(item.status, ForecastExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_forecast_exception("code-242", "name-636");
        if item.move_to(ForecastExceptionStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastExceptionStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_exception("code-242", "name-636");
        item.status = ForecastExceptionStatus::Closed;
        assert!(
            item.move_to(ForecastExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

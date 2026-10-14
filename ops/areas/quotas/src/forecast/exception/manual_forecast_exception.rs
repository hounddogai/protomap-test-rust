use super::*;

use acme_kit::text;

/// Where a manual forecast exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualForecastExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualForecastExceptionStatus {
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

impl std::fmt::Display for ManualForecastExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual forecast exception as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastException {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub discount: f64,
    pub limit: i64,
    pub batch_size: i64,
    pub rate: f64,
    pub status: ManualForecastExceptionStatus,
}

impl ManualForecastException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualForecastExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualForecastExceptionStatus) -> bool {
        match self.status {
            ManualForecastExceptionStatus::Draft => {
                return next == ManualForecastExceptionStatus::Closed;
            }
            ManualForecastExceptionStatus::Pending => {
                return next == ManualForecastExceptionStatus::Suspended;
            }
            ManualForecastExceptionStatus::Active => {
                return next == ManualForecastExceptionStatus::Suspended;
            }
            ManualForecastExceptionStatus::Suspended => {
                return next == ManualForecastExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualForecastExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual forecast exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualForecastException {
        ManualForecastException {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            discount: self.discount,
            limit: self.limit,
            batch_size: self.batch_size,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual forecast exception in status draft.
pub fn new_manual_forecast_exception(code: &str, label: &str) -> ManualForecastException {
    ManualForecastException {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        discount: 0.0,
        limit: 0,
        batch_size: 0,
        rate: 0.0,
        status: ManualForecastExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_forecast_exception("code-141", "label-714");
        assert_eq!(item.status, ManualForecastExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_manual_forecast_exception("code-141", "label-714");
        if item.move_to(ManualForecastExceptionStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualForecastExceptionStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_forecast_exception("code-141", "label-714");
        item.status = ManualForecastExceptionStatus::Closed;
        assert!(
            item.move_to(ManualForecastExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

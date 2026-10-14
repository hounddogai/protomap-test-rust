use super::*;

use acme_kit::text;

/// Where a forecast template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastTemplateStatus {
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

impl std::fmt::Display for ForecastTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplate {
    pub code: String,
    pub title: String,
    pub country: String,
    pub category: String,
    pub archived: bool,
    pub limit: i64,
    pub approved: bool,
    pub contact_email: String,
    pub status: ForecastTemplateStatus,
}

impl ForecastTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastTemplateStatus) -> bool {
        match self.status {
            ForecastTemplateStatus::Idle => {
                return next == ForecastTemplateStatus::Stopped;
            }
            ForecastTemplateStatus::Warming => {
                match next {
                    ForecastTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastTemplateStatus::Ready => {
                return next == ForecastTemplateStatus::Draining || next == ForecastTemplateStatus::Stopped;
            }
            ForecastTemplateStatus::Draining => {
                return next == ForecastTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> ForecastTemplate {
        ForecastTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            category: self.category.clone(),
            archived: self.archived,
            limit: self.limit,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a forecast template in status idle.
pub fn new_forecast_template(code: &str, title: &str) -> ForecastTemplate {
    ForecastTemplate {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        category: "".to_string(),
        archived: false,
        limit: 0,
        approved: false,
        contact_email: "".to_string(),
        status: ForecastTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_template("code-407", "title-495");
        assert_eq!(item.status, ForecastTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_forecast_template("code-407", "title-495");
        if item.move_to(ForecastTemplateStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastTemplateStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_template("code-407", "title-495");
        item.status = ForecastTemplateStatus::Stopped;
        assert!(
            item.move_to(ForecastTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

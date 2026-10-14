use super::*;

use acme_kit::text;

/// Where a suspension forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecast {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub revision: i64,
    pub title: String,
    pub reviewer: String,
    pub window_days: i64,
    pub namespace: String,
    pub cost: f64,
    pub limit: i64,
    pub version: i64,
    pub status: SuspensionForecastStatus,
}

impl SuspensionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionForecastStatus) -> bool {
        match self.status {
            SuspensionForecastStatus::Created => {
                return next == SuspensionForecastStatus::Validated || next == SuspensionForecastStatus::Published;
            }
            SuspensionForecastStatus::Validated => {
                match next {
                    SuspensionForecastStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionForecastStatus::Published => {
                return next == SuspensionForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionForecast {
        SuspensionForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            revision: self.revision,
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            cost: self.cost,
            limit: self.limit,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a suspension forecast in status created.
pub fn new_suspension_forecast(code: &str, name: &str) -> SuspensionForecast {
    SuspensionForecast {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        revision: 0,
        title: "".to_string(),
        reviewer: "".to_string(),
        window_days: 0,
        namespace: "".to_string(),
        cost: 0.0,
        limit: 0,
        version: 0,
        status: SuspensionForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_suspension_forecast("code-434", "name-560");
        assert_eq!(item.status, SuspensionForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_suspension_forecast("code-434", "name-560");
        if item.move_to(SuspensionForecastStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionForecastStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_suspension_forecast("code-434", "name-560");
        item.status = SuspensionForecastStatus::Retired;
        assert!(
            item.move_to(SuspensionForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

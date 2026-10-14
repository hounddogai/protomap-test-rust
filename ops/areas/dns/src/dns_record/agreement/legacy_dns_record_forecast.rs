use super::*;

use acme_kit::text;

/// Where a legacy dns record forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDnsRecordForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyDnsRecordForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyDnsRecordForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy dns record forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecast {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub category: String,
    pub archived: bool,
    pub limit: i64,
    pub status: LegacyDnsRecordForecastStatus,
}

impl LegacyDnsRecordForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDnsRecordForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDnsRecordForecastStatus) -> bool {
        match self.status {
            LegacyDnsRecordForecastStatus::Created => {
                match next {
                    LegacyDnsRecordForecastStatus::Validated | LegacyDnsRecordForecastStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDnsRecordForecastStatus::Validated => {
                return next == LegacyDnsRecordForecastStatus::Retired;
            }
            LegacyDnsRecordForecastStatus::Published => {
                return next == LegacyDnsRecordForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDnsRecordForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dns record forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy dns record forecast", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDnsRecordForecast {
        LegacyDnsRecordForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            category: self.category.clone(),
            archived: self.archived,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy dns record forecast in status created.
pub fn new_legacy_dns_record_forecast(code: &str, name: &str) -> LegacyDnsRecordForecast {
    LegacyDnsRecordForecast {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        category: "".to_string(),
        archived: false,
        limit: 0,
        status: LegacyDnsRecordForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_dns_record_forecast("code-747", "name-533");
        assert_eq!(item.status, LegacyDnsRecordForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_dns_record_forecast("code-747", "name-533");
        if item.move_to(LegacyDnsRecordForecastStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDnsRecordForecastStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_dns_record_forecast("code-747", "name-533");
        item.status = LegacyDnsRecordForecastStatus::Retired;
        assert!(
            item.move_to(LegacyDnsRecordForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

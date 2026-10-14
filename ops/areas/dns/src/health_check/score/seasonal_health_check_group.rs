use super::*;

use acme_kit::text;

/// Where a seasonal health check group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalHealthCheckGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalHealthCheckGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalHealthCheckGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal health check group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroup {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub score: f64,
    pub rate: f64,
    pub contact_email: String,
    pub visible: bool,
    pub source: String,
    pub scope: String,
    pub capacity: i64,
    pub status: SeasonalHealthCheckGroupStatus,
}

impl SeasonalHealthCheckGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalHealthCheckGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalHealthCheckGroupStatus) -> bool {
        match self.status {
            SeasonalHealthCheckGroupStatus::Created => {
                match next {
                    SeasonalHealthCheckGroupStatus::Validated | SeasonalHealthCheckGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckGroupStatus::Validated => {
                return next == SeasonalHealthCheckGroupStatus::Published;
            }
            SeasonalHealthCheckGroupStatus::Published => {
                match next {
                    SeasonalHealthCheckGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalHealthCheckGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal health check group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.rate * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalHealthCheckGroup {
        SeasonalHealthCheckGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            score: self.score,
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            source: self.source.clone(),
            scope: self.scope.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal health check group in status created.
pub fn new_seasonal_health_check_group(code: &str, name: &str) -> SeasonalHealthCheckGroup {
    SeasonalHealthCheckGroup {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        score: 0.0,
        rate: 0.0,
        contact_email: "".to_string(),
        visible: false,
        source: "".to_string(),
        scope: "".to_string(),
        capacity: 0,
        status: SeasonalHealthCheckGroupStatus::Created,
    }
}

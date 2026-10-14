use super::*;

use acme_kit::text;

/// Where a tiered forecast mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredForecastMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredForecastMappingStatus {
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

impl std::fmt::Display for TieredForecastMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered forecast mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMapping {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub minimum: f64,
    pub pinned: bool,
    pub region: String,
    pub rotation_days: i64,
    pub scope: String,
    pub version: i64,
    pub phone: String,
    pub status: TieredForecastMappingStatus,
}

impl TieredForecastMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredForecastMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredForecastMappingStatus) -> bool {
        match self.status {
            TieredForecastMappingStatus::Draft => {
                return next == TieredForecastMappingStatus::Closed;
            }
            TieredForecastMappingStatus::Pending => {
                return next == TieredForecastMappingStatus::Active || next == TieredForecastMappingStatus::Suspended;
            }
            TieredForecastMappingStatus::Active => {
                return next == TieredForecastMappingStatus::Suspended || next == TieredForecastMappingStatus::Closed;
            }
            TieredForecastMappingStatus::Suspended => {
                return next == TieredForecastMappingStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredForecastMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered forecast mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered forecast mapping", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredForecastMapping {
        TieredForecastMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            minimum: self.minimum,
            pinned: self.pinned,
            region: self.region.clone(),
            rotation_days: self.rotation_days,
            scope: self.scope.clone(),
            version: self.version,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered forecast mapping in status draft.
pub fn new_tiered_forecast_mapping(code: &str, label: &str) -> TieredForecastMapping {
    TieredForecastMapping {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        minimum: 0.0,
        pinned: false,
        region: "".to_string(),
        rotation_days: 0,
        scope: "".to_string(),
        version: 0,
        phone: "".to_string(),
        status: TieredForecastMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_forecast_mapping("code-793", "label-57");
        assert_eq!(item.status, TieredForecastMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_tiered_forecast_mapping("code-793", "label-57");
        if item.move_to(TieredForecastMappingStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredForecastMappingStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_forecast_mapping("code-793", "label-57");
        item.status = TieredForecastMappingStatus::Closed;
        assert!(
            item.move_to(TieredForecastMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

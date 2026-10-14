use super::*;

use acme_kit::text;

/// Where a weighted forecast mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedForecastMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedForecastMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedForecastMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted forecast mapping as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMapping {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub scope: String,
    pub fingerprint: String,
    pub label: String,
    pub revision: i64,
    pub starts_at: i64,
    pub timezone: String,
    pub status: WeightedForecastMappingStatus,
}

impl WeightedForecastMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedForecastMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedForecastMappingStatus) -> bool {
        match self.status {
            WeightedForecastMappingStatus::Proposed => {
                match next {
                    WeightedForecastMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastMappingStatus::Approved => {
                match next {
                    WeightedForecastMappingStatus::Rejected | WeightedForecastMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastMappingStatus::Rejected => {
                match next {
                    WeightedForecastMappingStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedForecastMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted forecast mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.revision as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedForecastMapping {
        WeightedForecastMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            scope: self.scope.clone(),
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            revision: self.revision,
            starts_at: self.starts_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted forecast mapping in status proposed.
pub fn new_weighted_forecast_mapping(code: &str, title: &str) -> WeightedForecastMapping {
    WeightedForecastMapping {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        scope: "".to_string(),
        fingerprint: "".to_string(),
        label: "".to_string(),
        revision: 0,
        starts_at: 0,
        timezone: "".to_string(),
        status: WeightedForecastMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_forecast_mapping("code-782", "title-380");
        assert_eq!(item.status, WeightedForecastMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_weighted_forecast_mapping("code-782", "title-380");
        if item.move_to(WeightedForecastMappingStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedForecastMappingStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_forecast_mapping("code-782", "title-380");
        item.status = WeightedForecastMappingStatus::Withdrawn;
        assert!(
            item.move_to(WeightedForecastMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a primary forecast mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryForecastMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryForecastMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryForecastMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary forecast mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastMapping {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub weight: f64,
    pub summary: String,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub cost: f64,
    pub status: PrimaryForecastMappingStatus,
}

impl PrimaryForecastMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryForecastMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryForecastMappingStatus) -> bool {
        match self.status {
            PrimaryForecastMappingStatus::Proposed => {
                return next == PrimaryForecastMappingStatus::Approved;
            }
            PrimaryForecastMappingStatus::Approved => {
                return next == PrimaryForecastMappingStatus::Withdrawn;
            }
            PrimaryForecastMappingStatus::Rejected => {
                return next == PrimaryForecastMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryForecastMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary forecast mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.score - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryForecastMapping {
        PrimaryForecastMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            weight: self.weight,
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary forecast mapping in status proposed.
pub fn new_primary_forecast_mapping(code: &str, label: &str) -> PrimaryForecastMapping {
    PrimaryForecastMapping {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        weight: 0.0,
        summary: "".to_string(),
        issuer: "".to_string(),
        ttl_seconds: 0,
        cost: 0.0,
        status: PrimaryForecastMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_forecast_mapping("code-335", "label-888");
        assert_eq!(item.status, PrimaryForecastMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_forecast_mapping("code-335", "label-888");
        if item.move_to(PrimaryForecastMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryForecastMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_forecast_mapping("code-335", "label-888");
        item.status = PrimaryForecastMappingStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryForecastMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

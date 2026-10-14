use super::*;

use acme_kit::text;

/// Where a weighted federation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedFederationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedFederationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted federation estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationEstimate {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub name: String,
    pub country: String,
    pub created_at: i64,
    pub minimum: f64,
    pub currency: String,
    pub status: WeightedFederationEstimateStatus,
}

impl WeightedFederationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationEstimateStatus) -> bool {
        match self.status {
            WeightedFederationEstimateStatus::Created => {
                return next == WeightedFederationEstimateStatus::Validated
                    || next == WeightedFederationEstimateStatus::Published;
            }
            WeightedFederationEstimateStatus::Validated => {
                return next == WeightedFederationEstimateStatus::Published
                    || next == WeightedFederationEstimateStatus::Retired;
            }
            WeightedFederationEstimateStatus::Published => {
                match next {
                    WeightedFederationEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: WeightedFederationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation estimate {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationEstimate {
        WeightedFederationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            name: self.name.clone(),
            country: self.country.clone(),
            created_at: self.created_at,
            minimum: self.minimum,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted federation estimate in status created.
pub fn new_weighted_federation_estimate(code: &str, label: &str) -> WeightedFederationEstimate {
    WeightedFederationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        name: "".to_string(),
        country: "".to_string(),
        created_at: 0,
        minimum: 0.0,
        currency: "".to_string(),
        status: WeightedFederationEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_federation_estimate("code-596", "label-296");
        assert_eq!(item.status, WeightedFederationEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_federation_estimate("code-596", "label-296");
        if item.move_to(WeightedFederationEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFederationEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_federation_estimate("code-596", "label-296");
        item.status = WeightedFederationEstimateStatus::Retired;
        assert!(
            item.move_to(WeightedFederationEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

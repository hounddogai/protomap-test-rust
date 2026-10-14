use super::*;

use acme_kit::text;

/// Where a forecast manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ForecastManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ForecastManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastManifest {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub locale: String,
    pub maximum: f64,
    pub reviewer: String,
    pub status: ForecastManifestStatus,
}

impl ForecastManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastManifestStatus) -> bool {
        match self.status {
            ForecastManifestStatus::Requested => {
                match next {
                    ForecastManifestStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastManifestStatus::Granted => {
                return next == ForecastManifestStatus::Revoked || next == ForecastManifestStatus::Expired;
            }
            ForecastManifestStatus::Revoked => {
                return next == ForecastManifestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "forecast manifest", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.capacity as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastManifest {
        ForecastManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            locale: self.locale.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a forecast manifest in status requested.
pub fn new_forecast_manifest(code: &str, name: &str) -> ForecastManifest {
    ForecastManifest {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        locale: "".to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        status: ForecastManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_forecast_manifest("code-638", "name-302");
        assert_eq!(item.status, ForecastManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_forecast_manifest("code-638", "name-302");
        if item.move_to(ForecastManifestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastManifestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_forecast_manifest("code-638", "name-302");
        item.status = ForecastManifestStatus::Expired;
        assert!(
            item.move_to(ForecastManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a capped measurement quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedMeasurementQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedMeasurementQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedMeasurementQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped measurement quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuota {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub country: String,
    pub owner_id: String,
    pub title: String,
    pub notes: String,
    pub priority: i64,
    pub port: i64,
    pub ratio: f64,
    pub audience: String,
    pub status: CappedMeasurementQuotaStatus,
}

impl CappedMeasurementQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedMeasurementQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedMeasurementQuotaStatus) -> bool {
        match self.status {
            CappedMeasurementQuotaStatus::Created => {
                match next {
                    CappedMeasurementQuotaStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedMeasurementQuotaStatus::Validated => {
                return next == CappedMeasurementQuotaStatus::Published
                    || next == CappedMeasurementQuotaStatus::Retired;
            }
            CappedMeasurementQuotaStatus::Published => {
                return next == CappedMeasurementQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedMeasurementQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped measurement quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped measurement quota", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.priority as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedMeasurementQuota {
        CappedMeasurementQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            priority: self.priority,
            port: self.port,
            ratio: self.ratio,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped measurement quota in status created.
pub fn new_capped_measurement_quota(code: &str, name: &str) -> CappedMeasurementQuota {
    CappedMeasurementQuota {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        owner_id: "".to_string(),
        title: "".to_string(),
        notes: "".to_string(),
        priority: 0,
        port: 0,
        ratio: 0.0,
        audience: "".to_string(),
        status: CappedMeasurementQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_measurement_quota("code-128", "name-285");
        assert_eq!(item.status, CappedMeasurementQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_measurement_quota("code-128", "name-285");
        if item.move_to(CappedMeasurementQuotaStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedMeasurementQuotaStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_measurement_quota("code-128", "name-285");
        item.status = CappedMeasurementQuotaStatus::Retired;
        assert!(
            item.move_to(CappedMeasurementQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

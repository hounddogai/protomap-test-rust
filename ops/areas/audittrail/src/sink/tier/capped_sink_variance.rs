use super::*;

use acme_kit::text;

/// Where a capped sink variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSinkVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedSinkVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedSinkVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped sink variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkVariance {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub rate: f64,
    pub phone: String,
    pub duration_minutes: i64,
    pub unit: String,
    pub attempts: i64,
    pub status: CappedSinkVarianceStatus,
}

impl CappedSinkVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSinkVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSinkVarianceStatus) -> bool {
        match self.status {
            CappedSinkVarianceStatus::Created => {
                match next {
                    CappedSinkVarianceStatus::Published | CappedSinkVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSinkVarianceStatus::Validated => {
                match next {
                    CappedSinkVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSinkVarianceStatus::Published => {
                return next == CappedSinkVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSinkVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped sink variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped sink variance", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSinkVariance {
        CappedSinkVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            rate: self.rate,
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            unit: self.unit.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped sink variance in status created.
pub fn new_capped_sink_variance(code: &str, title: &str) -> CappedSinkVariance {
    CappedSinkVariance {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        rate: 0.0,
        phone: "".to_string(),
        duration_minutes: 0,
        unit: "".to_string(),
        attempts: 0,
        status: CappedSinkVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_sink_variance("code-955", "title-48");
        assert_eq!(item.status, CappedSinkVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_sink_variance("code-955", "title-48");
        if item.move_to(CappedSinkVarianceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSinkVarianceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_sink_variance("code-955", "title-48");
        item.status = CappedSinkVarianceStatus::Retired;
        assert!(
            item.move_to(CappedSinkVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

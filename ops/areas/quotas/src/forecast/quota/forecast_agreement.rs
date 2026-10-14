use super::*;

use acme_kit::text;

/// Where a forecast agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastAgreementStatus {
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

impl std::fmt::Display for ForecastAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAgreement {
    pub code: String,
    pub title: String,
    pub region: String,
    pub source: String,
    pub ends_at: i64,
    pub owner_id: String,
    pub issuer: String,
    pub status: ForecastAgreementStatus,
}

impl ForecastAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastAgreementStatus) -> bool {
        match self.status {
            ForecastAgreementStatus::Draft => {
                return next == ForecastAgreementStatus::Active || next == ForecastAgreementStatus::Suspended;
            }
            ForecastAgreementStatus::Pending => {
                return next == ForecastAgreementStatus::Closed;
            }
            ForecastAgreementStatus::Active => {
                return next == ForecastAgreementStatus::Suspended || next == ForecastAgreementStatus::Closed;
            }
            ForecastAgreementStatus::Suspended => {
                match next {
                    ForecastAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: ForecastAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> ForecastAgreement {
        ForecastAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            source: self.source.clone(),
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a forecast agreement in status draft.
pub fn new_forecast_agreement(code: &str, title: &str) -> ForecastAgreement {
    ForecastAgreement {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        source: "".to_string(),
        ends_at: 0,
        owner_id: "".to_string(),
        issuer: "".to_string(),
        status: ForecastAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_agreement("code-914", "title-106");
        assert_eq!(item.status, ForecastAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_forecast_agreement("code-914", "title-106");
        if item.move_to(ForecastAgreementStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastAgreementStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_agreement("code-914", "title-106");
        item.status = ForecastAgreementStatus::Closed;
        assert!(
            item.move_to(ForecastAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

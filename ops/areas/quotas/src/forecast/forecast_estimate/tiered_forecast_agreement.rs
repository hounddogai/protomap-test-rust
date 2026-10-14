use super::*;

use acme_kit::text;

/// Where a tiered forecast agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredForecastAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredForecastAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredForecastAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered forecast agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAgreement {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub ends_at: i64,
    pub archived: bool,
    pub minimum: f64,
    pub source: String,
    pub country: String,
    pub status: TieredForecastAgreementStatus,
}

impl TieredForecastAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredForecastAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredForecastAgreementStatus) -> bool {
        match self.status {
            TieredForecastAgreementStatus::Idle => {
                match next {
                    TieredForecastAgreementStatus::Ready | TieredForecastAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredForecastAgreementStatus::Warming => {
                match next {
                    TieredForecastAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredForecastAgreementStatus::Ready => {
                match next {
                    TieredForecastAgreementStatus::Draining | TieredForecastAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredForecastAgreementStatus::Draining => {
                match next {
                    TieredForecastAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredForecastAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered forecast agreement {} cannot move from {} to {}",
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
        format!("{} {}", "tiered forecast agreement", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.minimum - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredForecastAgreement {
        TieredForecastAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            ends_at: self.ends_at,
            archived: self.archived,
            minimum: self.minimum,
            source: self.source.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered forecast agreement in status idle.
pub fn new_tiered_forecast_agreement(code: &str, title: &str) -> TieredForecastAgreement {
    TieredForecastAgreement {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        ends_at: 0,
        archived: false,
        minimum: 0.0,
        source: "".to_string(),
        country: "".to_string(),
        status: TieredForecastAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_forecast_agreement("code-168", "title-255");
        assert_eq!(item.status, TieredForecastAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tiered_forecast_agreement("code-168", "title-255");
        if item.move_to(TieredForecastAgreementStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredForecastAgreementStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_forecast_agreement("code-168", "title-255");
        item.status = TieredForecastAgreementStatus::Stopped;
        assert!(
            item.move_to(TieredForecastAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

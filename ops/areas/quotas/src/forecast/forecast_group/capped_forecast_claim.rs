use super::*;

use acme_kit::text;

/// Where a capped forecast claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedForecastClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedForecastClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CappedForecastClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped forecast claim as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastClaim {
    pub code: String,
    pub label: String,
    pub source: String,
    pub updated_at: i64,
    pub audience: String,
    pub notes: String,
    pub quantity: i64,
    pub tenant_id: String,
    pub score: f64,
    pub scope: String,
    pub visible: bool,
    pub status: CappedForecastClaimStatus,
}

impl CappedForecastClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedForecastClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedForecastClaimStatus) -> bool {
        match self.status {
            CappedForecastClaimStatus::Open => {
                return next == CappedForecastClaimStatus::InReview || next == CappedForecastClaimStatus::Archived;
            }
            CappedForecastClaimStatus::InReview => {
                match next {
                    CappedForecastClaimStatus::Reopened | CappedForecastClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedForecastClaimStatus::Resolved => {
                match next {
                    CappedForecastClaimStatus::Reopened | CappedForecastClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedForecastClaimStatus::Reopened => {
                match next {
                    CappedForecastClaimStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedForecastClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped forecast claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped forecast claim", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.score + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedForecastClaim {
        CappedForecastClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            scope: self.scope.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped forecast claim in status open.
pub fn new_capped_forecast_claim(code: &str, label: &str) -> CappedForecastClaim {
    CappedForecastClaim {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        updated_at: 0,
        audience: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        tenant_id: "".to_string(),
        score: 0.0,
        scope: "".to_string(),
        visible: false,
        status: CappedForecastClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_forecast_claim("code-994", "label-725");
        assert_eq!(item.status, CappedForecastClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_capped_forecast_claim("code-994", "label-725");
        if item.move_to(CappedForecastClaimStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedForecastClaimStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_forecast_claim("code-994", "label-725");
        item.status = CappedForecastClaimStatus::Archived;
        assert!(
            item.move_to(CappedForecastClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a seasonal plan variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPlanVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalPlanVarianceStatus {
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

impl std::fmt::Display for SeasonalPlanVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal plan variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanVariance {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub host: String,
    pub threshold: f64,
    pub status: SeasonalPlanVarianceStatus,
}

impl SeasonalPlanVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPlanVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPlanVarianceStatus) -> bool {
        match self.status {
            SeasonalPlanVarianceStatus::Draft => {
                match next {
                    SeasonalPlanVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPlanVarianceStatus::Pending => {
                return next == SeasonalPlanVarianceStatus::Closed;
            }
            SeasonalPlanVarianceStatus::Active => {
                return next == SeasonalPlanVarianceStatus::Suspended || next == SeasonalPlanVarianceStatus::Closed;
            }
            SeasonalPlanVarianceStatus::Suspended => {
                match next {
                    SeasonalPlanVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalPlanVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal plan variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rate - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPlanVariance {
        SeasonalPlanVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            host: self.host.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal plan variance in status draft.
pub fn new_seasonal_plan_variance(code: &str, label: &str) -> SeasonalPlanVariance {
    SeasonalPlanVariance {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        host: "".to_string(),
        threshold: 0.0,
        status: SeasonalPlanVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_plan_variance("code-611", "label-832");
        assert_eq!(item.status, SeasonalPlanVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_seasonal_plan_variance("code-611", "label-832");
        if item.move_to(SeasonalPlanVarianceStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPlanVarianceStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_plan_variance("code-611", "label-832");
        item.status = SeasonalPlanVarianceStatus::Closed;
        assert!(
            item.move_to(SeasonalPlanVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

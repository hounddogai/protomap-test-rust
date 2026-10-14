use super::*;

use acme_kit::text;

/// Where a fallback failover metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFailoverMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackFailoverMetricStatus {
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

impl std::fmt::Display for FallbackFailoverMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback failover metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverMetric {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub segment: String,
    pub rate: f64,
    pub currency: String,
    pub visible: bool,
    pub status: FallbackFailoverMetricStatus,
}

impl FallbackFailoverMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFailoverMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFailoverMetricStatus) -> bool {
        match self.status {
            FallbackFailoverMetricStatus::Draft => {
                match next {
                    FallbackFailoverMetricStatus::Pending | FallbackFailoverMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverMetricStatus::Pending => {
                match next {
                    FallbackFailoverMetricStatus::Active | FallbackFailoverMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverMetricStatus::Active => {
                match next {
                    FallbackFailoverMetricStatus::Suspended | FallbackFailoverMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverMetricStatus::Suspended => {
                match next {
                    FallbackFailoverMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackFailoverMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback failover metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback failover metric", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ratio + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFailoverMetric {
        FallbackFailoverMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            segment: self.segment.clone(),
            rate: self.rate,
            currency: self.currency.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback failover metric in status draft.
pub fn new_fallback_failover_metric(code: &str, name: &str) -> FallbackFailoverMetric {
    FallbackFailoverMetric {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        segment: "".to_string(),
        rate: 0.0,
        currency: "".to_string(),
        visible: false,
        status: FallbackFailoverMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_failover_metric("code-708", "name-738");
        assert_eq!(item.status, FallbackFailoverMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_failover_metric("code-708", "name-738");
        if item.move_to(FallbackFailoverMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackFailoverMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_failover_metric("code-708", "name-738");
        item.status = FallbackFailoverMetricStatus::Closed;
        assert!(
            item.move_to(FallbackFailoverMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

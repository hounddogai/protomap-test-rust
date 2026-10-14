use super::*;

use acme_kit::text;

/// Where a failover metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverMetricStatus {
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

impl std::fmt::Display for FailoverMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover metric as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetric {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub retries: i64,
    pub weight: f64,
    pub owner_id: String,
    pub source: String,
    pub status: FailoverMetricStatus,
}

impl FailoverMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverMetricStatus) -> bool {
        match self.status {
            FailoverMetricStatus::Draft => {
                return next == FailoverMetricStatus::Suspended || next == FailoverMetricStatus::Closed;
            }
            FailoverMetricStatus::Pending => {
                match next {
                    FailoverMetricStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverMetricStatus::Active => {
                match next {
                    FailoverMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverMetricStatus::Suspended => {
                match next {
                    FailoverMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: FailoverMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverMetric {
        FailoverMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            retries: self.retries,
            weight: self.weight,
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover metric in status draft.
pub fn new_failover_metric(code: &str, label: &str) -> FailoverMetric {
    FailoverMetric {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        retries: 0,
        weight: 0.0,
        owner_id: "".to_string(),
        source: "".to_string(),
        status: FailoverMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_failover_metric("code-223", "label-726");
        assert_eq!(item.status, FailoverMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_failover_metric("code-223", "label-726");
        if item.move_to(FailoverMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_failover_metric("code-223", "label-726");
        item.status = FailoverMetricStatus::Closed;
        assert!(
            item.move_to(FailoverMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

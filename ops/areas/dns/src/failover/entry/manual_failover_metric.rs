use super::*;

use acme_kit::text;

/// Where a manual failover metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFailoverMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualFailoverMetricStatus {
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

impl std::fmt::Display for ManualFailoverMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual failover metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverMetric {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub timezone: String,
    pub batch_size: i64,
    pub principal: String,
    pub description: String,
    pub active: bool,
    pub priority: i64,
    pub checksum: String,
    pub source: String,
    pub status: ManualFailoverMetricStatus,
}

impl ManualFailoverMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFailoverMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFailoverMetricStatus) -> bool {
        match self.status {
            ManualFailoverMetricStatus::Draft => {
                match next {
                    ManualFailoverMetricStatus::Pending | ManualFailoverMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFailoverMetricStatus::Pending => {
                return next == ManualFailoverMetricStatus::Active || next == ManualFailoverMetricStatus::Suspended;
            }
            ManualFailoverMetricStatus::Active => {
                return next == ManualFailoverMetricStatus::Suspended || next == ManualFailoverMetricStatus::Closed;
            }
            ManualFailoverMetricStatus::Suspended => {
                return next == ManualFailoverMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualFailoverMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual failover metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFailoverMetric {
        ManualFailoverMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            description: self.description.clone(),
            active: self.active,
            priority: self.priority,
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual failover metric in status draft.
pub fn new_manual_failover_metric(code: &str, title: &str) -> ManualFailoverMetric {
    ManualFailoverMetric {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        timezone: "".to_string(),
        batch_size: 0,
        principal: "".to_string(),
        description: "".to_string(),
        active: false,
        priority: 0,
        checksum: "".to_string(),
        source: "".to_string(),
        status: ManualFailoverMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_failover_metric("code-42", "title-57");
        assert_eq!(item.status, ManualFailoverMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_manual_failover_metric("code-42", "title-57");
        if item.move_to(ManualFailoverMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFailoverMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_failover_metric("code-42", "title-57");
        item.status = ManualFailoverMetricStatus::Closed;
        assert!(
            item.move_to(ManualFailoverMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

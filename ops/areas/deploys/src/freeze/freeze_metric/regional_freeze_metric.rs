use super::*;

use acme_kit::text;

/// Where a regional freeze metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFreezeMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalFreezeMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalFreezeMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional freeze metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetric {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub priority: i64,
    pub margin: f64,
    pub archived: bool,
    pub fingerprint: String,
    pub cost: f64,
    pub source: String,
    pub amount: f64,
    pub status: RegionalFreezeMetricStatus,
}

impl RegionalFreezeMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFreezeMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFreezeMetricStatus) -> bool {
        match self.status {
            RegionalFreezeMetricStatus::Requested => {
                match next {
                    RegionalFreezeMetricStatus::Granted | RegionalFreezeMetricStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFreezeMetricStatus::Granted => {
                match next {
                    RegionalFreezeMetricStatus::Revoked | RegionalFreezeMetricStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFreezeMetricStatus::Revoked => {
                return next == RegionalFreezeMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFreezeMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional freeze metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional freeze metric", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.cost * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFreezeMetric {
        RegionalFreezeMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            priority: self.priority,
            margin: self.margin,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            cost: self.cost,
            source: self.source.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional freeze metric in status requested.
pub fn new_regional_freeze_metric(code: &str, title: &str) -> RegionalFreezeMetric {
    RegionalFreezeMetric {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        priority: 0,
        margin: 0.0,
        archived: false,
        fingerprint: "".to_string(),
        cost: 0.0,
        source: "".to_string(),
        amount: 0.0,
        status: RegionalFreezeMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_freeze_metric("code-866", "title-203");
        assert_eq!(item.status, RegionalFreezeMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_freeze_metric("code-866", "title-203");
        if item.move_to(RegionalFreezeMetricStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFreezeMetricStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_freeze_metric("code-866", "title-203");
        item.status = RegionalFreezeMetricStatus::Expired;
        assert!(
            item.move_to(RegionalFreezeMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

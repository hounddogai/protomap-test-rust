use super::*;

use acme_kit::text;

/// Where a regional failover variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFailoverVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalFailoverVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalFailoverVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional failover variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVariance {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub created_at: i64,
    pub window_days: i64,
    pub margin: f64,
    pub scope: String,
    pub ceiling: f64,
    pub title: String,
    pub cluster: String,
    pub status: RegionalFailoverVarianceStatus,
}

impl RegionalFailoverVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFailoverVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFailoverVarianceStatus) -> bool {
        match self.status {
            RegionalFailoverVarianceStatus::Requested => {
                return next == RegionalFailoverVarianceStatus::Granted
                    || next == RegionalFailoverVarianceStatus::Expired;
            }
            RegionalFailoverVarianceStatus::Granted => {
                return next == RegionalFailoverVarianceStatus::Expired;
            }
            RegionalFailoverVarianceStatus::Revoked => {
                return next == RegionalFailoverVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFailoverVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional failover variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional failover variance", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quantity as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFailoverVariance {
        RegionalFailoverVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            created_at: self.created_at,
            window_days: self.window_days,
            margin: self.margin,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional failover variance in status requested.
pub fn new_regional_failover_variance(code: &str, name: &str) -> RegionalFailoverVariance {
    RegionalFailoverVariance {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        created_at: 0,
        window_days: 0,
        margin: 0.0,
        scope: "".to_string(),
        ceiling: 0.0,
        title: "".to_string(),
        cluster: "".to_string(),
        status: RegionalFailoverVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_failover_variance("code-144", "name-653");
        assert_eq!(item.status, RegionalFailoverVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_failover_variance("code-144", "name-653");
        if item.move_to(RegionalFailoverVarianceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFailoverVarianceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_failover_variance("code-144", "name-653");
        item.status = RegionalFailoverVarianceStatus::Expired;
        assert!(
            item.move_to(RegionalFailoverVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

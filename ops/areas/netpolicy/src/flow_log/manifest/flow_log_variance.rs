use super::*;

use acme_kit::text;

/// Where a flow log variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FlowLogVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FlowLogVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log variance as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVariance {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub audience: String,
    pub issuer: String,
    pub rate: f64,
    pub cost: f64,
    pub burst: i64,
    pub status: FlowLogVarianceStatus,
}

impl FlowLogVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogVarianceStatus) -> bool {
        match self.status {
            FlowLogVarianceStatus::Requested => {
                return next == FlowLogVarianceStatus::Granted || next == FlowLogVarianceStatus::Revoked;
            }
            FlowLogVarianceStatus::Granted => {
                match next {
                    FlowLogVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogVarianceStatus::Revoked => {
                match next {
                    FlowLogVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: FlowLogVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log variance", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogVariance {
        FlowLogVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            audience: self.audience.clone(),
            issuer: self.issuer.clone(),
            rate: self.rate,
            cost: self.cost,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a flow log variance in status requested.
pub fn new_flow_log_variance(code: &str, name: &str) -> FlowLogVariance {
    FlowLogVariance {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        audience: "".to_string(),
        issuer: "".to_string(),
        rate: 0.0,
        cost: 0.0,
        burst: 0,
        status: FlowLogVarianceStatus::Requested,
    }
}

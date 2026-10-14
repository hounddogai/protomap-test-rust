use super::*;

use acme_kit::text;

/// Where a regional quote budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQuoteBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalQuoteBudgetStatus {
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

impl std::fmt::Display for RegionalQuoteBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional quote budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudget {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub fingerprint: String,
    pub host: String,
    pub external_ref: String,
    pub ratio: f64,
    pub key_id: String,
    pub visible: bool,
    pub rate: f64,
    pub reviewer: String,
    pub status: RegionalQuoteBudgetStatus,
}

impl RegionalQuoteBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQuoteBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQuoteBudgetStatus) -> bool {
        match self.status {
            RegionalQuoteBudgetStatus::Idle => {
                match next {
                    RegionalQuoteBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQuoteBudgetStatus::Warming => {
                match next {
                    RegionalQuoteBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQuoteBudgetStatus::Ready => {
                match next {
                    RegionalQuoteBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQuoteBudgetStatus::Draining => {
                return next == RegionalQuoteBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalQuoteBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional quote budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        (self.rate * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalQuoteBudget {
        RegionalQuoteBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            key_id: self.key_id.clone(),
            visible: self.visible,
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional quote budget in status idle.
pub fn new_regional_quote_budget(code: &str, name: &str) -> RegionalQuoteBudget {
    RegionalQuoteBudget {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        external_ref: "".to_string(),
        ratio: 0.0,
        key_id: "".to_string(),
        visible: false,
        rate: 0.0,
        reviewer: "".to_string(),
        status: RegionalQuoteBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_quote_budget("code-248", "name-928");
        assert_eq!(item.status, RegionalQuoteBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_quote_budget("code-248", "name-928");
        if item.move_to(RegionalQuoteBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalQuoteBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_quote_budget("code-248", "name-928");
        item.status = RegionalQuoteBudgetStatus::Stopped;
        assert!(
            item.move_to(RegionalQuoteBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

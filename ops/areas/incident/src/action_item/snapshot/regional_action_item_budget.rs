use super::*;

use acme_kit::text;

/// Where a regional action item budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalActionItemBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalActionItemBudgetStatus {
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

impl std::fmt::Display for RegionalActionItemBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional action item budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudget {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub margin: f64,
    pub updated_at: i64,
    pub floor_value: f64,
    pub reviewer: String,
    pub minimum: f64,
    pub tenant_id: String,
    pub region: String,
    pub rate: f64,
    pub status: RegionalActionItemBudgetStatus,
}

impl RegionalActionItemBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalActionItemBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalActionItemBudgetStatus) -> bool {
        match self.status {
            RegionalActionItemBudgetStatus::Idle => {
                match next {
                    RegionalActionItemBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemBudgetStatus::Warming => {
                return next == RegionalActionItemBudgetStatus::Ready
                    || next == RegionalActionItemBudgetStatus::Stopped;
            }
            RegionalActionItemBudgetStatus::Ready => {
                match next {
                    RegionalActionItemBudgetStatus::Draining | RegionalActionItemBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemBudgetStatus::Draining => {
                return next == RegionalActionItemBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalActionItemBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional action item budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.port as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalActionItemBudget {
        RegionalActionItemBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            margin: self.margin,
            updated_at: self.updated_at,
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional action item budget in status idle.
pub fn new_regional_action_item_budget(code: &str, name: &str) -> RegionalActionItemBudget {
    RegionalActionItemBudget {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        margin: 0.0,
        updated_at: 0,
        floor_value: 0.0,
        reviewer: "".to_string(),
        minimum: 0.0,
        tenant_id: "".to_string(),
        region: "".to_string(),
        rate: 0.0,
        status: RegionalActionItemBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_action_item_budget("code-388", "name-910");
        assert_eq!(item.status, RegionalActionItemBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_action_item_budget("code-388", "name-910");
        if item.move_to(RegionalActionItemBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalActionItemBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_action_item_budget("code-388", "name-910");
        item.status = RegionalActionItemBudgetStatus::Stopped;
        assert!(
            item.move_to(RegionalActionItemBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

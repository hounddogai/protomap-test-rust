use super::*;

use acme_kit::text;

/// Where a flow log tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogTierStatus {
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

impl std::fmt::Display for FlowLogTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log tier as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTier {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub created_at: i64,
    pub approved: bool,
    pub weight: f64,
    pub status: FlowLogTierStatus,
}

impl FlowLogTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogTierStatus) -> bool {
        match self.status {
            FlowLogTierStatus::Draft => {
                return next == FlowLogTierStatus::Pending || next == FlowLogTierStatus::Suspended;
            }
            FlowLogTierStatus::Pending => {
                match next {
                    FlowLogTierStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogTierStatus::Active => {
                match next {
                    FlowLogTierStatus::Suspended | FlowLogTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogTierStatus::Suspended => {
                match next {
                    FlowLogTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log tier {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.created_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogTier {
        FlowLogTier {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            created_at: self.created_at,
            approved: self.approved,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log tier in status draft.
pub fn new_flow_log_tier(code: &str, title: &str) -> FlowLogTier {
    FlowLogTier {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        created_at: 0,
        approved: false,
        weight: 0.0,
        status: FlowLogTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_tier("code-58", "title-197");
        assert_eq!(item.status, FlowLogTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_flow_log_tier("code-58", "title-197");
        if item.move_to(FlowLogTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_tier("code-58", "title-197");
        item.status = FlowLogTierStatus::Closed;
        assert!(item.move_to(FlowLogTierStatus::Draft).is_err(), "expected move to fail");
    }
}

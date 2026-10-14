use super::*;

use acme_kit::text;

/// Where a weighted action item cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedActionItemCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedActionItemCycleStatus {
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

impl std::fmt::Display for WeightedActionItemCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted action item cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycle {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub limit: i64,
    pub owner_id: String,
    pub notes: String,
    pub capacity: i64,
    pub starts_at: i64,
    pub status: WeightedActionItemCycleStatus,
}

impl WeightedActionItemCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedActionItemCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedActionItemCycleStatus) -> bool {
        match self.status {
            WeightedActionItemCycleStatus::Idle => {
                return next == WeightedActionItemCycleStatus::Warming;
            }
            WeightedActionItemCycleStatus::Warming => {
                match next {
                    WeightedActionItemCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedActionItemCycleStatus::Ready => {
                match next {
                    WeightedActionItemCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedActionItemCycleStatus::Draining => {
                match next {
                    WeightedActionItemCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedActionItemCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted action item cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.port as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedActionItemCycle {
        WeightedActionItemCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            notes: self.notes.clone(),
            capacity: self.capacity,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted action item cycle in status idle.
pub fn new_weighted_action_item_cycle(code: &str, name: &str) -> WeightedActionItemCycle {
    WeightedActionItemCycle {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        limit: 0,
        owner_id: "".to_string(),
        notes: "".to_string(),
        capacity: 0,
        starts_at: 0,
        status: WeightedActionItemCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_action_item_cycle("code-93", "name-732");
        assert_eq!(item.status, WeightedActionItemCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_action_item_cycle("code-93", "name-732");
        if item.move_to(WeightedActionItemCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedActionItemCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_action_item_cycle("code-93", "name-732");
        item.status = WeightedActionItemCycleStatus::Stopped;
        assert!(
            item.move_to(WeightedActionItemCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

use super::*;

use acme_kit::text;

/// Where a global quote cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalQuoteCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalQuoteCycleStatus {
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

impl std::fmt::Display for GlobalQuoteCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global quote cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCycle {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub unit: String,
    pub revision: i64,
    pub quantity: i64,
    pub discount: f64,
    pub capacity: i64,
    pub currency: String,
    pub ratio: f64,
    pub title: String,
    pub status: GlobalQuoteCycleStatus,
}

impl GlobalQuoteCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalQuoteCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalQuoteCycleStatus) -> bool {
        match self.status {
            GlobalQuoteCycleStatus::Idle => {
                match next {
                    GlobalQuoteCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteCycleStatus::Warming => {
                match next {
                    GlobalQuoteCycleStatus::Draining | GlobalQuoteCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteCycleStatus::Ready => {
                match next {
                    GlobalQuoteCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteCycleStatus::Draining => {
                match next {
                    GlobalQuoteCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalQuoteCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global quote cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quantity as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalQuoteCycle {
        GlobalQuoteCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            revision: self.revision,
            quantity: self.quantity,
            discount: self.discount,
            capacity: self.capacity,
            currency: self.currency.clone(),
            ratio: self.ratio,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global quote cycle in status idle.
pub fn new_global_quote_cycle(code: &str, name: &str) -> GlobalQuoteCycle {
    GlobalQuoteCycle {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        unit: "".to_string(),
        revision: 0,
        quantity: 0,
        discount: 0.0,
        capacity: 0,
        currency: "".to_string(),
        ratio: 0.0,
        title: "".to_string(),
        status: GlobalQuoteCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_quote_cycle("code-447", "name-595");
        assert_eq!(item.status, GlobalQuoteCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_global_quote_cycle("code-447", "name-595");
        if item.move_to(GlobalQuoteCycleStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalQuoteCycleStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_quote_cycle("code-447", "name-595");
        item.status = GlobalQuoteCycleStatus::Stopped;
        assert!(
            item.move_to(GlobalQuoteCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

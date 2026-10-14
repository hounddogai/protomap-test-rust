use super::*;

use acme_kit::text;

/// Where a global peering window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPeeringWindowStatus {
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

impl std::fmt::Display for GlobalPeeringWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global peering window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindow {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub key_id: String,
    pub discount: f64,
    pub threshold: f64,
    pub reviewer: String,
    pub batch_size: i64,
    pub timezone: String,
    pub status: GlobalPeeringWindowStatus,
}

impl GlobalPeeringWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringWindowStatus) -> bool {
        match self.status {
            GlobalPeeringWindowStatus::Draft => {
                return next == GlobalPeeringWindowStatus::Closed;
            }
            GlobalPeeringWindowStatus::Pending => {
                return next == GlobalPeeringWindowStatus::Suspended;
            }
            GlobalPeeringWindowStatus::Active => {
                return next == GlobalPeeringWindowStatus::Closed;
            }
            GlobalPeeringWindowStatus::Suspended => {
                match next {
                    GlobalPeeringWindowStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalPeeringWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.threshold + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringWindow {
        GlobalPeeringWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            key_id: self.key_id.clone(),
            discount: self.discount,
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            batch_size: self.batch_size,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global peering window in status draft.
pub fn new_global_peering_window(code: &str, name: &str) -> GlobalPeeringWindow {
    GlobalPeeringWindow {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        key_id: "".to_string(),
        discount: 0.0,
        threshold: 0.0,
        reviewer: "".to_string(),
        batch_size: 0,
        timezone: "".to_string(),
        status: GlobalPeeringWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_peering_window("code-534", "name-976");
        assert_eq!(item.status, GlobalPeeringWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_global_peering_window("code-534", "name-976");
        if item.move_to(GlobalPeeringWindowStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringWindowStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_peering_window("code-534", "name-976");
        item.status = GlobalPeeringWindowStatus::Closed;
        assert!(
            item.move_to(GlobalPeeringWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

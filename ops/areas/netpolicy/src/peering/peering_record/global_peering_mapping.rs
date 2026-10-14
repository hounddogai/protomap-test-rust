use super::*;

use acme_kit::text;

/// Where a global peering mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalPeeringMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for GlobalPeeringMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global peering mapping as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMapping {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub namespace: String,
    pub quantity: i64,
    pub approved: bool,
    pub burst: i64,
    pub status: GlobalPeeringMappingStatus,
}

impl GlobalPeeringMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringMappingStatus) -> bool {
        match self.status {
            GlobalPeeringMappingStatus::New => {
                match next {
                    GlobalPeeringMappingStatus::Queued | GlobalPeeringMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringMappingStatus::Queued => {
                return next == GlobalPeeringMappingStatus::Done;
            }
            GlobalPeeringMappingStatus::Running => {
                return next == GlobalPeeringMappingStatus::Done || next == GlobalPeeringMappingStatus::Failed;
            }
            GlobalPeeringMappingStatus::Done => {
                return next == GlobalPeeringMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPeeringMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global peering mapping", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.burst as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringMapping {
        GlobalPeeringMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            namespace: self.namespace.clone(),
            quantity: self.quantity,
            approved: self.approved,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global peering mapping in status new.
pub fn new_global_peering_mapping(code: &str, name: &str) -> GlobalPeeringMapping {
    GlobalPeeringMapping {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        namespace: "".to_string(),
        quantity: 0,
        approved: false,
        burst: 0,
        status: GlobalPeeringMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_peering_mapping("code-257", "name-823");
        assert_eq!(item.status, GlobalPeeringMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_global_peering_mapping("code-257", "name-823");
        if item.move_to(GlobalPeeringMappingStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringMappingStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_peering_mapping("code-257", "name-823");
        item.status = GlobalPeeringMappingStatus::Failed;
        assert!(
            item.move_to(GlobalPeeringMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

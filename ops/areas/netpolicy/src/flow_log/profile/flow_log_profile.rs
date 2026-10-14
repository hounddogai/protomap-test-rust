use super::*;

use acme_kit::text;

/// Where a flow log profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogProfileStatus {
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

impl std::fmt::Display for FlowLogProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfile {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub rotation_days: i64,
    pub sequence: i64,
    pub region: String,
    pub status: FlowLogProfileStatus,
}

impl FlowLogProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogProfileStatus) -> bool {
        match self.status {
            FlowLogProfileStatus::Draft => {
                match next {
                    FlowLogProfileStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogProfileStatus::Pending => {
                match next {
                    FlowLogProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogProfileStatus::Active => {
                match next {
                    FlowLogProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogProfileStatus::Suspended => {
                match next {
                    FlowLogProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.sequence as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogProfile {
        FlowLogProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a flow log profile in status draft.
pub fn new_flow_log_profile(code: &str, name: &str) -> FlowLogProfile {
    FlowLogProfile {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        rotation_days: 0,
        sequence: 0,
        region: "".to_string(),
        status: FlowLogProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_profile("code-947", "name-178");
        assert_eq!(item.status, FlowLogProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_flow_log_profile("code-947", "name-178");
        if item.move_to(FlowLogProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_profile("code-947", "name-178");
        item.status = FlowLogProfileStatus::Closed;
        assert!(
            item.move_to(FlowLogProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

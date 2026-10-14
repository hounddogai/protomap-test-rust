use super::*;

use acme_kit::text;

/// Where a escalation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EscalationThresholdStatus {
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

impl std::fmt::Display for EscalationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationThreshold {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub port: i64,
    pub label: String,
    pub floor_value: f64,
    pub version: i64,
    pub principal: String,
    pub rotation_days: i64,
    pub status: EscalationThresholdStatus,
}

impl EscalationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationThresholdStatus) -> bool {
        match self.status {
            EscalationThresholdStatus::Draft => {
                return next == EscalationThresholdStatus::Pending || next == EscalationThresholdStatus::Active;
            }
            EscalationThresholdStatus::Pending => {
                return next == EscalationThresholdStatus::Suspended || next == EscalationThresholdStatus::Closed;
            }
            EscalationThresholdStatus::Active => {
                match next {
                    EscalationThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationThresholdStatus::Suspended => {
                return next == EscalationThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.cost + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationThreshold {
        EscalationThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            port: self.port,
            label: self.label.clone(),
            floor_value: self.floor_value,
            version: self.version,
            principal: self.principal.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation threshold in status draft.
pub fn new_escalation_threshold(code: &str, name: &str) -> EscalationThreshold {
    EscalationThreshold {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        port: 0,
        label: "".to_string(),
        floor_value: 0.0,
        version: 0,
        principal: "".to_string(),
        rotation_days: 0,
        status: EscalationThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_escalation_threshold("code-830", "name-993");
        assert_eq!(item.status, EscalationThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_escalation_threshold("code-830", "name-993");
        if item.move_to(EscalationThresholdStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationThresholdStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_escalation_threshold("code-830", "name-993");
        item.status = EscalationThresholdStatus::Closed;
        assert!(
            item.move_to(EscalationThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

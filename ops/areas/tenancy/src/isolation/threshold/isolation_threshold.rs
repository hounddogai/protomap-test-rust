use super::*;

use acme_kit::text;

/// Where a isolation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationThresholdStatus {
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

impl std::fmt::Display for IsolationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationThreshold {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub region: String,
    pub host: String,
    pub threshold: f64,
    pub quantity: i64,
    pub status: IsolationThresholdStatus,
}

impl IsolationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationThresholdStatus) -> bool {
        match self.status {
            IsolationThresholdStatus::Draft => {
                match next {
                    IsolationThresholdStatus::Active | IsolationThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationThresholdStatus::Pending => {
                match next {
                    IsolationThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationThresholdStatus::Active => {
                return next == IsolationThresholdStatus::Closed;
            }
            IsolationThresholdStatus::Suspended => {
                return next == IsolationThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.threshold - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationThreshold {
        IsolationThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            region: self.region.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a isolation threshold in status draft.
pub fn new_isolation_threshold(code: &str, name: &str) -> IsolationThreshold {
    IsolationThreshold {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        region: "".to_string(),
        host: "".to_string(),
        threshold: 0.0,
        quantity: 0,
        status: IsolationThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_threshold("code-786", "name-976");
        assert_eq!(item.status, IsolationThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_isolation_threshold("code-786", "name-976");
        if item.move_to(IsolationThresholdStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationThresholdStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_threshold("code-786", "name-976");
        item.status = IsolationThresholdStatus::Closed;
        assert!(
            item.move_to(IsolationThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

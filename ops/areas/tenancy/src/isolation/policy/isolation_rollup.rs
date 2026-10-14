use super::*;

use acme_kit::text;

/// Where a isolation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationRollupStatus {
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

impl std::fmt::Display for IsolationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRollup {
    pub code: String,
    pub title: String,
    pub country: String,
    pub expires_at: i64,
    pub quantity: i64,
    pub name: String,
    pub issuer: String,
    pub minimum: f64,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub status: IsolationRollupStatus,
}

impl IsolationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationRollupStatus) -> bool {
        match self.status {
            IsolationRollupStatus::Idle => {
                return next == IsolationRollupStatus::Draining;
            }
            IsolationRollupStatus::Warming => {
                return next == IsolationRollupStatus::Ready || next == IsolationRollupStatus::Draining;
            }
            IsolationRollupStatus::Ready => {
                return next == IsolationRollupStatus::Draining || next == IsolationRollupStatus::Stopped;
            }
            IsolationRollupStatus::Draining => {
                return next == IsolationRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation rollup", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationRollup {
        IsolationRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            quantity: self.quantity,
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a isolation rollup in status idle.
pub fn new_isolation_rollup(code: &str, title: &str) -> IsolationRollup {
    IsolationRollup {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        expires_at: 0,
        quantity: 0,
        name: "".to_string(),
        issuer: "".to_string(),
        minimum: 0.0,
        starts_at: 0,
        duration_minutes: 0,
        status: IsolationRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_rollup("code-723", "title-360");
        assert_eq!(item.status, IsolationRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_isolation_rollup("code-723", "title-360");
        if item.move_to(IsolationRollupStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationRollupStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_rollup("code-723", "title-360");
        item.status = IsolationRollupStatus::Stopped;
        assert!(
            item.move_to(IsolationRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}

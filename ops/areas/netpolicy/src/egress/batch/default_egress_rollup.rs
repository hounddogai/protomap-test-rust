use super::*;

use acme_kit::text;

/// Where a default egress rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEgressRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultEgressRollupStatus {
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

impl std::fmt::Display for DefaultEgressRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default egress rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressRollup {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub minimum: f64,
    pub amount: f64,
    pub maximum: f64,
    pub expires_at: i64,
    pub checksum: String,
    pub status: DefaultEgressRollupStatus,
}

impl DefaultEgressRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEgressRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEgressRollupStatus) -> bool {
        match self.status {
            DefaultEgressRollupStatus::Draft => {
                return next == DefaultEgressRollupStatus::Active;
            }
            DefaultEgressRollupStatus::Pending => {
                match next {
                    DefaultEgressRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressRollupStatus::Active => {
                return next == DefaultEgressRollupStatus::Suspended;
            }
            DefaultEgressRollupStatus::Suspended => {
                return next == DefaultEgressRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEgressRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default egress rollup {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.maximum - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEgressRollup {
        DefaultEgressRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            minimum: self.minimum,
            amount: self.amount,
            maximum: self.maximum,
            expires_at: self.expires_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default egress rollup in status draft.
pub fn new_default_egress_rollup(code: &str, label: &str) -> DefaultEgressRollup {
    DefaultEgressRollup {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        maximum: 0.0,
        expires_at: 0,
        checksum: "".to_string(),
        status: DefaultEgressRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_egress_rollup("code-623", "label-676");
        assert_eq!(item.status, DefaultEgressRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_default_egress_rollup("code-623", "label-676");
        if item.move_to(DefaultEgressRollupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEgressRollupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_egress_rollup("code-623", "label-676");
        item.status = DefaultEgressRollupStatus::Closed;
        assert!(
            item.move_to(DefaultEgressRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

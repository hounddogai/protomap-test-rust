use super::*;

use acme_kit::text;

/// Where a manual silence cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualSilenceCycleStatus {
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

impl std::fmt::Display for ManualSilenceCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual silence cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceCycle {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub version: i64,
    pub minimum: f64,
    pub starts_at: i64,
    pub status: ManualSilenceCycleStatus,
}

impl ManualSilenceCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceCycleStatus) -> bool {
        match self.status {
            ManualSilenceCycleStatus::Draft => {
                match next {
                    ManualSilenceCycleStatus::Active | ManualSilenceCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceCycleStatus::Pending => {
                return next == ManualSilenceCycleStatus::Active || next == ManualSilenceCycleStatus::Suspended;
            }
            ManualSilenceCycleStatus::Active => {
                return next == ManualSilenceCycleStatus::Suspended;
            }
            ManualSilenceCycleStatus::Suspended => {
                match next {
                    ManualSilenceCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualSilenceCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        (self.amount * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSilenceCycle {
        ManualSilenceCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            version: self.version,
            minimum: self.minimum,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual silence cycle in status draft.
pub fn new_manual_silence_cycle(code: &str, name: &str) -> ManualSilenceCycle {
    ManualSilenceCycle {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        version: 0,
        minimum: 0.0,
        starts_at: 0,
        status: ManualSilenceCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_silence_cycle("code-233", "name-706");
        assert_eq!(item.status, ManualSilenceCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_silence_cycle("code-233", "name-706");
        if item.move_to(ManualSilenceCycleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSilenceCycleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_silence_cycle("code-233", "name-706");
        item.status = ManualSilenceCycleStatus::Closed;
        assert!(
            item.move_to(ManualSilenceCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

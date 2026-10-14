use super::*;

use acme_kit::text;

/// Where a lenient isolation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientIsolationRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientIsolationRollupStatus {
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

impl std::fmt::Display for LenientIsolationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient isolation rollup as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollup {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub active: bool,
    pub region: String,
    pub cipher: String,
    pub status: LenientIsolationRollupStatus,
}

impl LenientIsolationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientIsolationRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientIsolationRollupStatus) -> bool {
        match self.status {
            LenientIsolationRollupStatus::Draft => {
                return next == LenientIsolationRollupStatus::Pending;
            }
            LenientIsolationRollupStatus::Pending => {
                return next == LenientIsolationRollupStatus::Active || next == LenientIsolationRollupStatus::Suspended;
            }
            LenientIsolationRollupStatus::Active => {
                match next {
                    LenientIsolationRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientIsolationRollupStatus::Suspended => {
                match next {
                    LenientIsolationRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientIsolationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient isolation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> LenientIsolationRollup {
        LenientIsolationRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            active: self.active,
            region: self.region.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient isolation rollup in status draft.
pub fn new_lenient_isolation_rollup(code: &str, name: &str) -> LenientIsolationRollup {
    LenientIsolationRollup {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        active: false,
        region: "".to_string(),
        cipher: "".to_string(),
        status: LenientIsolationRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_isolation_rollup("code-58", "name-955");
        assert_eq!(item.status, LenientIsolationRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_isolation_rollup("code-58", "name-955");
        if item.move_to(LenientIsolationRollupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientIsolationRollupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_isolation_rollup("code-58", "name-955");
        item.status = LenientIsolationRollupStatus::Closed;
        assert!(
            item.move_to(LenientIsolationRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

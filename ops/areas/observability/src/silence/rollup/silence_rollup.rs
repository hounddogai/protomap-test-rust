use super::*;

use acme_kit::text;

/// Where a silence rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollup {
    pub code: String,
    pub name: String,
    pub description: String,
    pub limit: i64,
    pub segment: String,
    pub status: SilenceRollupStatus,
}

impl SilenceRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceRollupStatus) -> bool {
        match self.status {
            SilenceRollupStatus::Requested => {
                match next {
                    SilenceRollupStatus::Granted | SilenceRollupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceRollupStatus::Granted => {
                match next {
                    SilenceRollupStatus::Revoked | SilenceRollupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceRollupStatus::Revoked => {
                return next == SilenceRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> SilenceRollup {
        SilenceRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            limit: self.limit,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a silence rollup in status requested.
pub fn new_silence_rollup(code: &str, name: &str) -> SilenceRollup {
    SilenceRollup {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        limit: 0,
        segment: "".to_string(),
        status: SilenceRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_silence_rollup("code-642", "name-711");
        assert_eq!(item.status, SilenceRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_silence_rollup("code-642", "name-711");
        if item.move_to(SilenceRollupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceRollupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_silence_rollup("code-642", "name-711");
        item.status = SilenceRollupStatus::Expired;
        assert!(
            item.move_to(SilenceRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}

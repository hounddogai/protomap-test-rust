use super::*;

use acme_kit::text;

/// Where a regional verifier rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalVerifierRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalVerifierRollupStatus {
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

impl std::fmt::Display for RegionalVerifierRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional verifier rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollup {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub channel: String,
    pub version: i64,
    pub audience: String,
    pub budget: f64,
    pub status: RegionalVerifierRollupStatus,
}

impl RegionalVerifierRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalVerifierRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalVerifierRollupStatus) -> bool {
        match self.status {
            RegionalVerifierRollupStatus::New => {
                match next {
                    RegionalVerifierRollupStatus::Queued | RegionalVerifierRollupStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalVerifierRollupStatus::Queued => {
                return next == RegionalVerifierRollupStatus::Done || next == RegionalVerifierRollupStatus::Failed;
            }
            RegionalVerifierRollupStatus::Running => {
                match next {
                    RegionalVerifierRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalVerifierRollupStatus::Done => {
                return next == RegionalVerifierRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalVerifierRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional verifier rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.budget * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalVerifierRollup {
        RegionalVerifierRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            channel: self.channel.clone(),
            version: self.version,
            audience: self.audience.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional verifier rollup in status new.
pub fn new_regional_verifier_rollup(code: &str, title: &str) -> RegionalVerifierRollup {
    RegionalVerifierRollup {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        channel: "".to_string(),
        version: 0,
        audience: "".to_string(),
        budget: 0.0,
        status: RegionalVerifierRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_verifier_rollup("code-835", "title-812");
        assert_eq!(item.status, RegionalVerifierRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_regional_verifier_rollup("code-835", "title-812");
        if item.move_to(RegionalVerifierRollupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalVerifierRollupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_verifier_rollup("code-835", "title-812");
        item.status = RegionalVerifierRollupStatus::Failed;
        assert!(
            item.move_to(RegionalVerifierRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

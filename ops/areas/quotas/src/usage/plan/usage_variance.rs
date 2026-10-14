use super::*;

use acme_kit::text;

/// Where a usage variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl UsageVarianceStatus {
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

impl std::fmt::Display for UsageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageVariance {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub region: String,
    pub notes: String,
    pub amount: f64,
    pub status: UsageVarianceStatus,
}

impl UsageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageVarianceStatus) -> bool {
        match self.status {
            UsageVarianceStatus::New => {
                match next {
                    UsageVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageVarianceStatus::Queued => {
                return next == UsageVarianceStatus::Running || next == UsageVarianceStatus::Done;
            }
            UsageVarianceStatus::Running => {
                return next == UsageVarianceStatus::Done || next == UsageVarianceStatus::Failed;
            }
            UsageVarianceStatus::Done => {
                match next {
                    UsageVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: UsageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage variance", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> UsageVariance {
        UsageVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a usage variance in status new.
pub fn new_usage_variance(code: &str, name: &str) -> UsageVariance {
    UsageVariance {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        region: "".to_string(),
        notes: "".to_string(),
        amount: 0.0,
        status: UsageVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_usage_variance("code-690", "name-751");
        assert_eq!(item.status, UsageVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_usage_variance("code-690", "name-751");
        if item.move_to(UsageVarianceStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageVarianceStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_usage_variance("code-690", "name-751");
        item.status = UsageVarianceStatus::Failed;
        assert!(item.move_to(UsageVarianceStatus::New).is_err(), "expected move to fail");
    }
}

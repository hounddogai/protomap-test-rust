use super::*;

use acme_kit::text;

/// Where a rolling redaction policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRedactionPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingRedactionPolicyStatus {
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

impl std::fmt::Display for RollingRedactionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling redaction policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionPolicy {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub audience: String,
    pub discount: f64,
    pub owner_id: String,
    pub namespace: String,
    pub version: i64,
    pub cipher: String,
    pub label: String,
    pub status: RollingRedactionPolicyStatus,
}

impl RollingRedactionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRedactionPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRedactionPolicyStatus) -> bool {
        match self.status {
            RollingRedactionPolicyStatus::New => {
                match next {
                    RollingRedactionPolicyStatus::Queued | RollingRedactionPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionPolicyStatus::Queued => {
                match next {
                    RollingRedactionPolicyStatus::Running | RollingRedactionPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionPolicyStatus::Running => {
                return next == RollingRedactionPolicyStatus::Done || next == RollingRedactionPolicyStatus::Failed;
            }
            RollingRedactionPolicyStatus::Done => {
                return next == RollingRedactionPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRedactionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling redaction policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling redaction policy", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRedactionPolicy {
        RollingRedactionPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            audience: self.audience.clone(),
            discount: self.discount,
            owner_id: self.owner_id.clone(),
            namespace: self.namespace.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling redaction policy in status new.
pub fn new_rolling_redaction_policy(code: &str, title: &str) -> RollingRedactionPolicy {
    RollingRedactionPolicy {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        audience: "".to_string(),
        discount: 0.0,
        owner_id: "".to_string(),
        namespace: "".to_string(),
        version: 0,
        cipher: "".to_string(),
        label: "".to_string(),
        status: RollingRedactionPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_redaction_policy("code-842", "title-787");
        assert_eq!(item.status, RollingRedactionPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rolling_redaction_policy("code-842", "title-787");
        if item.move_to(RollingRedactionPolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRedactionPolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_redaction_policy("code-842", "title-787");
        item.status = RollingRedactionPolicyStatus::Failed;
        assert!(
            item.move_to(RollingRedactionPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

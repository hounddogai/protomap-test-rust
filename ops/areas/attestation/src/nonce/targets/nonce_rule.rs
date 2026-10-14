use super::*;

use acme_kit::text;

/// Where a nonce rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NonceRuleStatus {
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

impl std::fmt::Display for NonceRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce rule as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRule {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub issuer: String,
    pub contact_email: String,
    pub rate: f64,
    pub margin: f64,
    pub external_ref: String,
    pub rotation_days: i64,
    pub channel: String,
    pub status: NonceRuleStatus,
}

impl NonceRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceRuleStatus) -> bool {
        match self.status {
            NonceRuleStatus::New => {
                return next == NonceRuleStatus::Done;
            }
            NonceRuleStatus::Queued => {
                match next {
                    NonceRuleStatus::Running | NonceRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceRuleStatus::Running => {
                return next == NonceRuleStatus::Failed;
            }
            NonceRuleStatus::Done => {
                match next {
                    NonceRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: NonceRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceRule {
        NonceRule {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            margin: self.margin,
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a nonce rule in status new.
pub fn new_nonce_rule(code: &str, title: &str) -> NonceRule {
    NonceRule {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        issuer: "".to_string(),
        contact_email: "".to_string(),
        rate: 0.0,
        margin: 0.0,
        external_ref: "".to_string(),
        rotation_days: 0,
        channel: "".to_string(),
        status: NonceRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_rule("code-295", "title-107");
        assert_eq!(item.status, NonceRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_nonce_rule("code-295", "title-107");
        if item.move_to(NonceRuleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceRuleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_rule("code-295", "title-107");
        item.status = NonceRuleStatus::Failed;
        assert!(item.move_to(NonceRuleStatus::New).is_err(), "expected move to fail");
    }
}

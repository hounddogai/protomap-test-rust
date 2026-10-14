use super::*;

use acme_kit::text;

/// Where a default escalation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEscalationProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultEscalationProfileStatus {
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

impl std::fmt::Display for DefaultEscalationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default escalation profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationProfile {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub port: i64,
    pub issuer: String,
    pub rate: f64,
    pub channel: String,
    pub status: DefaultEscalationProfileStatus,
}

impl DefaultEscalationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEscalationProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEscalationProfileStatus) -> bool {
        match self.status {
            DefaultEscalationProfileStatus::Draft => {
                return next == DefaultEscalationProfileStatus::Suspended;
            }
            DefaultEscalationProfileStatus::Pending => {
                return next == DefaultEscalationProfileStatus::Active
                    || next == DefaultEscalationProfileStatus::Closed;
            }
            DefaultEscalationProfileStatus::Active => {
                match next {
                    DefaultEscalationProfileStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEscalationProfileStatus::Suspended => {
                return next == DefaultEscalationProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEscalationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default escalation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEscalationProfile {
        DefaultEscalationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            port: self.port,
            issuer: self.issuer.clone(),
            rate: self.rate,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default escalation profile in status draft.
pub fn new_default_escalation_profile(code: &str, title: &str) -> DefaultEscalationProfile {
    DefaultEscalationProfile {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        port: 0,
        issuer: "".to_string(),
        rate: 0.0,
        channel: "".to_string(),
        status: DefaultEscalationProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_escalation_profile("code-288", "title-401");
        assert_eq!(item.status, DefaultEscalationProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_default_escalation_profile("code-288", "title-401");
        if item.move_to(DefaultEscalationProfileStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEscalationProfileStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_escalation_profile("code-288", "title-401");
        item.status = DefaultEscalationProfileStatus::Closed;
        assert!(
            item.move_to(DefaultEscalationProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

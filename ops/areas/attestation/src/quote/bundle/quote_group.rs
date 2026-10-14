use super::*;

use acme_kit::text;

/// Where a quote group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteGroupStatus {
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

impl std::fmt::Display for QuoteGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroup {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub contact_email: String,
    pub grace_days: i64,
    pub weight: f64,
    pub tenant_id: String,
    pub active: bool,
    pub namespace: String,
    pub pinned: bool,
    pub status: QuoteGroupStatus,
}

impl QuoteGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteGroupStatus) -> bool {
        match self.status {
            QuoteGroupStatus::Idle => {
                return next == QuoteGroupStatus::Ready || next == QuoteGroupStatus::Stopped;
            }
            QuoteGroupStatus::Warming => {
                match next {
                    QuoteGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteGroupStatus::Ready => {
                match next {
                    QuoteGroupStatus::Draining | QuoteGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteGroupStatus::Draining => {
                match next {
                    QuoteGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.grace_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteGroup {
        QuoteGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            weight: self.weight,
            tenant_id: self.tenant_id.clone(),
            active: self.active,
            namespace: self.namespace.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote group in status idle.
pub fn new_quote_group(code: &str, name: &str) -> QuoteGroup {
    QuoteGroup {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        contact_email: "".to_string(),
        grace_days: 0,
        weight: 0.0,
        tenant_id: "".to_string(),
        active: false,
        namespace: "".to_string(),
        pinned: false,
        status: QuoteGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_group("code-952", "name-227");
        assert_eq!(item.status, QuoteGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_quote_group("code-952", "name-227");
        if item.move_to(QuoteGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_group("code-952", "name-227");
        item.status = QuoteGroupStatus::Stopped;
        assert!(item.move_to(QuoteGroupStatus::Idle).is_err(), "expected move to fail");
    }
}

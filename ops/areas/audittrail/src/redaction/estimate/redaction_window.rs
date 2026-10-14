use super::*;

use acme_kit::text;

/// Where a redaction window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RedactionWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RedactionWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction window as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindow {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub principal: String,
    pub starts_at: i64,
    pub status: RedactionWindowStatus,
}

impl RedactionWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionWindowStatus) -> bool {
        match self.status {
            RedactionWindowStatus::Proposed => {
                match next {
                    RedactionWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionWindowStatus::Approved => {
                return next == RedactionWindowStatus::Rejected || next == RedactionWindowStatus::Withdrawn;
            }
            RedactionWindowStatus::Rejected => {
                match next {
                    RedactionWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RedactionWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.score * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionWindow {
        RedactionWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            principal: self.principal.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction window in status proposed.
pub fn new_redaction_window(code: &str, title: &str) -> RedactionWindow {
    RedactionWindow {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        principal: "".to_string(),
        starts_at: 0,
        status: RedactionWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_redaction_window("code-628", "title-890");
        assert_eq!(item.status, RedactionWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_redaction_window("code-628", "title-890");
        if item.move_to(RedactionWindowStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionWindowStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_redaction_window("code-628", "title-890");
        item.status = RedactionWindowStatus::Withdrawn;
        assert!(
            item.move_to(RedactionWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}

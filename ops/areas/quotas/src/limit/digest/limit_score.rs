use super::*;

use acme_kit::text;

/// Where a limit score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitScoreStatus {
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

impl std::fmt::Display for LimitScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit score as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitScore {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub attempts: i64,
    pub batch_size: i64,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub ratio: f64,
    pub window_days: i64,
    pub margin: f64,
    pub visible: bool,
    pub status: LimitScoreStatus,
}

impl LimitScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitScoreStatus) -> bool {
        match self.status {
            LimitScoreStatus::Draft => {
                return next == LimitScoreStatus::Active;
            }
            LimitScoreStatus::Pending => {
                return next == LimitScoreStatus::Closed;
            }
            LimitScoreStatus::Active => {
                match next {
                    LimitScoreStatus::Suspended | LimitScoreStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitScoreStatus::Suspended => {
                return next == LimitScoreStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ratio - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitScore {
        LimitScore {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            attempts: self.attempts,
            batch_size: self.batch_size,
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            ratio: self.ratio,
            window_days: self.window_days,
            margin: self.margin,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a limit score in status draft.
pub fn new_limit_score(code: &str, label: &str) -> LimitScore {
    LimitScore {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        attempts: 0,
        batch_size: 0,
        ttl_seconds: 0,
        maximum: 0.0,
        ratio: 0.0,
        window_days: 0,
        margin: 0.0,
        visible: false,
        status: LimitScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_limit_score("code-998", "label-419");
        assert_eq!(item.status, LimitScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_limit_score("code-998", "label-419");
        if item.move_to(LimitScoreStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitScoreStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_limit_score("code-998", "label-419");
        item.status = LimitScoreStatus::Closed;
        assert!(item.move_to(LimitScoreStatus::Draft).is_err(), "expected move to fail");
    }
}

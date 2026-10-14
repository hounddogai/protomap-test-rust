use super::*;

use acme_kit::text;

/// Where a redaction target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionTargetStatus {
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

impl std::fmt::Display for RedactionTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTarget {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub source: String,
    pub capacity: i64,
    pub port: i64,
    pub expires_at: i64,
    pub revision: i64,
    pub threshold: f64,
    pub created_at: i64,
    pub amount: f64,
    pub status: RedactionTargetStatus,
}

impl RedactionTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionTargetStatus) -> bool {
        match self.status {
            RedactionTargetStatus::Draft => {
                match next {
                    RedactionTargetStatus::Active | RedactionTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionTargetStatus::Pending => {
                return next == RedactionTargetStatus::Suspended;
            }
            RedactionTargetStatus::Active => {
                match next {
                    RedactionTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionTargetStatus::Suspended => {
                match next {
                    RedactionTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: RedactionTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.batch_size as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionTarget {
        RedactionTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            source: self.source.clone(),
            capacity: self.capacity,
            port: self.port,
            expires_at: self.expires_at,
            revision: self.revision,
            threshold: self.threshold,
            created_at: self.created_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a redaction target in status draft.
pub fn new_redaction_target(code: &str, title: &str) -> RedactionTarget {
    RedactionTarget {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        source: "".to_string(),
        capacity: 0,
        port: 0,
        expires_at: 0,
        revision: 0,
        threshold: 0.0,
        created_at: 0,
        amount: 0.0,
        status: RedactionTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_redaction_target("code-526", "title-244");
        assert_eq!(item.status, RedactionTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_redaction_target("code-526", "title-244");
        if item.move_to(RedactionTargetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionTargetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_redaction_target("code-526", "title-244");
        item.status = RedactionTargetStatus::Closed;
        assert!(
            item.move_to(RedactionTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

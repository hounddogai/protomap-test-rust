use super::*;

use acme_kit::text;

/// Where a quote target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QuoteTargetStatus {
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

impl std::fmt::Display for QuoteTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTarget {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub checksum: String,
    pub cipher: String,
    pub burst: i64,
    pub contact_email: String,
    pub maximum: f64,
    pub audience: String,
    pub category: String,
    pub status: QuoteTargetStatus,
}

impl QuoteTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteTargetStatus) -> bool {
        match self.status {
            QuoteTargetStatus::Draft => {
                match next {
                    QuoteTargetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteTargetStatus::Pending => {
                match next {
                    QuoteTargetStatus::Active | QuoteTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteTargetStatus::Active => {
                match next {
                    QuoteTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteTargetStatus::Suspended => {
                match next {
                    QuoteTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: QuoteTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteTarget {
        QuoteTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            burst: self.burst,
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            audience: self.audience.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a quote target in status draft.
pub fn new_quote_target(code: &str, name: &str) -> QuoteTarget {
    QuoteTarget {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        cipher: "".to_string(),
        burst: 0,
        contact_email: "".to_string(),
        maximum: 0.0,
        audience: "".to_string(),
        category: "".to_string(),
        status: QuoteTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_quote_target("code-797", "name-925");
        assert_eq!(item.status, QuoteTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_quote_target("code-797", "name-925");
        if item.move_to(QuoteTargetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteTargetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_quote_target("code-797", "name-925");
        item.status = QuoteTargetStatus::Closed;
        assert!(item.move_to(QuoteTargetStatus::Draft).is_err(), "expected move to fail");
    }
}

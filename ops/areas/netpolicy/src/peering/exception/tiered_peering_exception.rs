use super::*;

use acme_kit::text;

/// Where a tiered peering exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPeeringExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredPeeringExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredPeeringExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered peering exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringException {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub approved: bool,
    pub port: i64,
    pub contact_email: String,
    pub score: f64,
    pub country: String,
    pub issuer: String,
    pub tags: Vec<String>,
    pub status: TieredPeeringExceptionStatus,
}

impl TieredPeeringException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPeeringExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPeeringExceptionStatus) -> bool {
        match self.status {
            TieredPeeringExceptionStatus::Created => {
                return next == TieredPeeringExceptionStatus::Retired;
            }
            TieredPeeringExceptionStatus::Validated => {
                match next {
                    TieredPeeringExceptionStatus::Published | TieredPeeringExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringExceptionStatus::Published => {
                match next {
                    TieredPeeringExceptionStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredPeeringExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered peering exception {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered peering exception", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.port as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPeeringException {
        TieredPeeringException {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            port: self.port,
            contact_email: self.contact_email.clone(),
            score: self.score,
            country: self.country.clone(),
            issuer: self.issuer.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered peering exception in status created.
pub fn new_tiered_peering_exception(code: &str, label: &str) -> TieredPeeringException {
    TieredPeeringException {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        approved: false,
        port: 0,
        contact_email: "".to_string(),
        score: 0.0,
        country: "".to_string(),
        issuer: "".to_string(),
        tags: Vec::new(),
        status: TieredPeeringExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_peering_exception("code-510", "label-214");
        assert_eq!(item.status, TieredPeeringExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_tiered_peering_exception("code-510", "label-214");
        if item.move_to(TieredPeeringExceptionStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPeeringExceptionStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_peering_exception("code-510", "label-214");
        item.status = TieredPeeringExceptionStatus::Retired;
        assert!(
            item.move_to(TieredPeeringExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

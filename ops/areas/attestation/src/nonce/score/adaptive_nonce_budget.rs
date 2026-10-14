use super::*;

use acme_kit::text;

/// Where a adaptive nonce budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveNonceBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveNonceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveNonceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive nonce budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceBudget {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub region: String,
    pub archived: bool,
    pub starts_at: i64,
    pub approved: bool,
    pub discount: f64,
    pub unit: String,
    pub phone: String,
    pub status: AdaptiveNonceBudgetStatus,
}

impl AdaptiveNonceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveNonceBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveNonceBudgetStatus) -> bool {
        match self.status {
            AdaptiveNonceBudgetStatus::Created => {
                match next {
                    AdaptiveNonceBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceBudgetStatus::Validated => {
                match next {
                    AdaptiveNonceBudgetStatus::Published | AdaptiveNonceBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceBudgetStatus::Published => {
                match next {
                    AdaptiveNonceBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: AdaptiveNonceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive nonce budget {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.discount + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveNonceBudget {
        AdaptiveNonceBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            region: self.region.clone(),
            archived: self.archived,
            starts_at: self.starts_at,
            approved: self.approved,
            discount: self.discount,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive nonce budget in status created.
pub fn new_adaptive_nonce_budget(code: &str, title: &str) -> AdaptiveNonceBudget {
    AdaptiveNonceBudget {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        region: "".to_string(),
        archived: false,
        starts_at: 0,
        approved: false,
        discount: 0.0,
        unit: "".to_string(),
        phone: "".to_string(),
        status: AdaptiveNonceBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_nonce_budget("code-955", "title-982");
        assert_eq!(item.status, AdaptiveNonceBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_nonce_budget("code-955", "title-982");
        if item.move_to(AdaptiveNonceBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveNonceBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_nonce_budget("code-955", "title-982");
        item.status = AdaptiveNonceBudgetStatus::Retired;
        assert!(
            item.move_to(AdaptiveNonceBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}

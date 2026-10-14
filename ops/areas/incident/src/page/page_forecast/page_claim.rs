use super::*;

use acme_kit::text;

/// Where a page claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PageClaimStatus {
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

impl std::fmt::Display for PageClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageClaim {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub source: String,
    pub currency: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub weight: f64,
    pub cluster: String,
    pub status: PageClaimStatus,
}

impl PageClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageClaimStatus) -> bool {
        match self.status {
            PageClaimStatus::Draft => {
                match next {
                    PageClaimStatus::Pending | PageClaimStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageClaimStatus::Pending => {
                return next == PageClaimStatus::Closed;
            }
            PageClaimStatus::Active => {
                return next == PageClaimStatus::Suspended || next == PageClaimStatus::Closed;
            }
            PageClaimStatus::Suspended => {
                match next {
                    PageClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: PageClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageClaim {
        PageClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            weight: self.weight,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a page claim in status draft.
pub fn new_page_claim(code: &str, name: &str) -> PageClaim {
    PageClaim {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        weight: 0.0,
        cluster: "".to_string(),
        status: PageClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_page_claim("code-90", "name-29");
        assert_eq!(item.status, PageClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_page_claim("code-90", "name-29");
        if item.move_to(PageClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_page_claim("code-90", "name-29");
        item.status = PageClaimStatus::Closed;
        assert!(item.move_to(PageClaimStatus::Draft).is_err(), "expected move to fail");
    }
}

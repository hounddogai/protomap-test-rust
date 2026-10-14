use super::*;

use acme_kit::text;

/// Where a page score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PageScoreStatus {
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

impl std::fmt::Display for PageScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageScore {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub description: String,
    pub limit: i64,
    pub capacity: i64,
    pub channel: String,
    pub namespace: String,
    pub rate: f64,
    pub locale: String,
    pub owner_id: String,
    pub status: PageScoreStatus,
}

impl PageScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageScoreStatus) -> bool {
        match self.status {
            PageScoreStatus::Draft => {
                match next {
                    PageScoreStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageScoreStatus::Pending => {
                return next == PageScoreStatus::Active || next == PageScoreStatus::Suspended;
            }
            PageScoreStatus::Active => {
                return next == PageScoreStatus::Closed;
            }
            PageScoreStatus::Suspended => {
                match next {
                    PageScoreStatus::Closed => {
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

    pub fn move_to(&mut self, next: PageScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "page score", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.limit as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageScore {
        PageScore {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            description: self.description.clone(),
            limit: self.limit,
            capacity: self.capacity,
            channel: self.channel.clone(),
            namespace: self.namespace.clone(),
            rate: self.rate,
            locale: self.locale.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a page score in status draft.
pub fn new_page_score(code: &str, name: &str) -> PageScore {
    PageScore {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        description: "".to_string(),
        limit: 0,
        capacity: 0,
        channel: "".to_string(),
        namespace: "".to_string(),
        rate: 0.0,
        locale: "".to_string(),
        owner_id: "".to_string(),
        status: PageScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_page_score("code-895", "name-758");
        assert_eq!(item.status, PageScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_page_score("code-895", "name-758");
        if item.move_to(PageScoreStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageScoreStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_page_score("code-895", "name-758");
        item.status = PageScoreStatus::Closed;
        assert!(item.move_to(PageScoreStatus::Draft).is_err(), "expected move to fail");
    }
}

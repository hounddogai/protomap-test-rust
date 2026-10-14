use super::*;

use acme_kit::text;

/// Where a capped query mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQueryMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedQueryMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedQueryMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped query mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryMapping {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub locale: String,
    pub principal: String,
    pub reviewer: String,
    pub active: bool,
    pub visible: bool,
    pub category: String,
    pub amount: f64,
    pub status: CappedQueryMappingStatus,
}

impl CappedQueryMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQueryMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQueryMappingStatus) -> bool {
        match self.status {
            CappedQueryMappingStatus::New => {
                match next {
                    CappedQueryMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryMappingStatus::Queued => {
                match next {
                    CappedQueryMappingStatus::Running | CappedQueryMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryMappingStatus::Running => {
                match next {
                    CappedQueryMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryMappingStatus::Done => {
                return next == CappedQueryMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedQueryMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped query mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped query mapping", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> CappedQueryMapping {
        CappedQueryMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            locale: self.locale.clone(),
            principal: self.principal.clone(),
            reviewer: self.reviewer.clone(),
            active: self.active,
            visible: self.visible,
            category: self.category.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped query mapping in status new.
pub fn new_capped_query_mapping(code: &str, name: &str) -> CappedQueryMapping {
    CappedQueryMapping {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        locale: "".to_string(),
        principal: "".to_string(),
        reviewer: "".to_string(),
        active: false,
        visible: false,
        category: "".to_string(),
        amount: 0.0,
        status: CappedQueryMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_query_mapping("code-963", "name-324");
        assert_eq!(item.status, CappedQueryMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_capped_query_mapping("code-963", "name-324");
        if item.move_to(CappedQueryMappingStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedQueryMappingStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_query_mapping("code-963", "name-324");
        item.status = CappedQueryMappingStatus::Failed;
        assert!(
            item.move_to(CappedQueryMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}

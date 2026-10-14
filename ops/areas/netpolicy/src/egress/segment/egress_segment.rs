use super::*;

use acme_kit::text;

/// Where a egress segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressSegmentStatus {
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

impl std::fmt::Display for EgressSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegment {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub tags: Vec<String>,
    pub phone: String,
    pub maximum: f64,
    pub ceiling: f64,
    pub status: EgressSegmentStatus,
}

impl EgressSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressSegmentStatus) -> bool {
        match self.status {
            EgressSegmentStatus::Draft => {
                match next {
                    EgressSegmentStatus::Suspended | EgressSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSegmentStatus::Pending => {
                match next {
                    EgressSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSegmentStatus::Active => {
                return next == EgressSegmentStatus::Suspended || next == EgressSegmentStatus::Closed;
            }
            EgressSegmentStatus::Suspended => {
                match next {
                    EgressSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.discount + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressSegment {
        EgressSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            tags: self.tags.clone(),
            phone: self.phone.clone(),
            maximum: self.maximum,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a egress segment in status draft.
pub fn new_egress_segment(code: &str, title: &str) -> EgressSegment {
    EgressSegment {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        tags: Vec::new(),
        phone: "".to_string(),
        maximum: 0.0,
        ceiling: 0.0,
        status: EgressSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_segment("code-872", "title-562");
        assert_eq!(item.status, EgressSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_egress_segment("code-872", "title-562");
        if item.move_to(EgressSegmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressSegmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_segment("code-872", "title-562");
        item.status = EgressSegmentStatus::Closed;
        assert!(
            item.move_to(EgressSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}

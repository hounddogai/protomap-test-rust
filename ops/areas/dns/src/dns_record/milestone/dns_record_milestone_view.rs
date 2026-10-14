use super::*;

use acme_kit::text;

/// How screens and exports show a dns record milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMilestoneView {
    pub code_text: String,
    pub label_text: String,
    pub name_text: String,
    pub notes: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_dns_record_milestone_view(item: &DnsRecordMilestone) -> DnsRecordMilestoneView {
    DnsRecordMilestoneView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        name_text: item.name.clone(),
        notes: item.notes.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_milestone_views(items: &[DnsRecordMilestone]) -> Vec<DnsRecordMilestoneView> {
    let mut views: Vec<DnsRecordMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_milestone_view(item));
    }
    views
}

pub fn sum_dns_record_milestone_view(view: &DnsRecordMilestoneView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_milestone("code-687", "label-942");
        let view = to_dns_record_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}

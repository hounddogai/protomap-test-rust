use super::*;

/// How screens and exports show a dns record proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposalView {
    pub code_text: String,
    pub name: String,
    pub principal: String,
    pub ends_at: String,
    pub visible: String,
    pub port: String,
    pub burst: String,
    pub window_days: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_dns_record_proposal_view(item: &DnsRecordProposal) -> DnsRecordProposalView {
    DnsRecordProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        principal: item.principal.clone(),
        ends_at: item.ends_at.to_string(),
        visible: format!("{}", item.visible),
        port: item.port.to_string(),
        burst: item.burst.to_string(),
        window_days: item.window_days.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_proposal_views(items: &[DnsRecordProposal]) -> Vec<DnsRecordProposalView> {
    let mut views: Vec<DnsRecordProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_proposal_view(item));
    }
    views
}

pub fn sum_dns_record_proposal_view(view: &DnsRecordProposalView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_port as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_proposal("code-847", "name-19");
        let view = to_dns_record_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}

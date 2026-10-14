use super::*;

use acme_kit::text;

/// How screens and exports show a dns record budget.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudgetView {
    pub code: String,
    pub name: String,
    pub burst: String,
    pub host_text: String,
    pub channel: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_dns_record_budget_view(item: &DnsRecordBudget) -> DnsRecordBudgetView {
    DnsRecordBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        burst: item.burst.to_string(),
        host_text: text::truncate(&item.host, 40),
        channel: item.channel.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_budget_views(items: &[DnsRecordBudget]) -> Vec<DnsRecordBudgetView> {
    let mut views: Vec<DnsRecordBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_budget_view(item));
    }
    views
}

pub fn sum_dns_record_budget_view(view: &DnsRecordBudgetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_burst as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_budget("code-517", "name-305");
        let view = to_dns_record_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}

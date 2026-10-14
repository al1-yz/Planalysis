//! The shapes exchanged with the interface over IPC. Field names are camelCase on the wire and
//! mirror the TypeScript entities; `name_key` is the normalized comparison key computed there.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub name_key: String,
    #[serde(rename = "type")]
    pub account_type: String,
    /// Cents at the start of `opening_date`.
    pub opening_balance: i64,
    pub opening_date: String,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub name_key: String,
    pub kind: String,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub name_key: String,
    pub closing_day: i64,
    pub due_day: i64,
    pub credit_limit: Option<i64>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transaction {
    pub id: String,
    pub account_id: Option<String>,
    pub card_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_id: Option<String>,
    pub date: String,
    /// Signed cents: positive is money in, negative is money out.
    pub amount: i64,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct TransactionFilter {
    pub account_id: Option<String>,
    pub card_id: Option<String>,
    pub category_id: Option<String>,
    /// Inclusive lower bound on the transaction date.
    pub from: Option<String>,
    /// Inclusive upper bound on the transaction date.
    pub to: Option<String>,
}

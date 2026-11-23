//! Your reseller account: balances, profile and the balance ledger. CLI: `dna account`.

use crate::de::{int_enum, model};
use crate::products::ProductType;
use crate::request::paged;
use crate::{Client, Endpoint, Error, ItemStream, Page, Paging, Query, Request, Validator};
use chrono::{DateTime, SecondsFormat, Utc};

int_enum! {
    /// The currency an account is kept in.
    pub enum AccountCurrency {
        /// Turkish lira.
        Try = 1 => "TRY",
        /// US dollar.
        Usd = 2 => "USD",
        /// Euro (the API's name for it is `EURO`, not the ISO code).
        Euro = 3 => "EURO",
    }
}

int_enum! {
    /// The state of a reseller account.
    pub enum ResellerStatus {
        /// Active.
        Active = 0 => "Active",
        /// Waiting for approval.
        WaitingForApproval = 1 => "WaitingForApproval",
        /// Suspended.
        Suspend = 2 => "Suspend",
        /// The setup wizard is not completed.
        WaitingForCompletionWizard = 3 => "WaitingForCompletionWizard",
    }
}

int_enum! {
    /// What a balance transaction was for.
    pub enum TransactionType {
        /// A registration.
        Register = 1 => "Register",
        /// A renewal.
        Renew = 2 => "Renew",
        /// A transfer.
        Transfer = 3 => "Transfer",
        /// A restore.
        Restore = 4 => "Restore",
        /// A deposit.
        Deposit = 5 => "Deposit",
        /// A backorder.
        Backorder = 6 => "Backorder",
        /// A refund.
        Refund = 7 => "Refund",
        /// A deduction.
        Deduction = 8 => "Deduction",
        /// A pre-registration.
        PreRegister = 9 => "PreRegister",
        /// A purchase.
        Shopping = 10 => "Shopping",
        /// A payment to a reseller.
        ResellerPayment = 11 => "ResellerPayment",
        /// A transfer between resellers of the platform.
        InternalTransfer = 12 => "InternalTransfer",
    }
}

int_enum! {
    /// The state of a balance transaction.
    pub enum TransactionStatus {
        /// Failed.
        Failed = 0 => "Failed",
        /// Succeeded.
        Success = 1 => "Success",
        /// Cancelled.
        Canceled = 2 => "Canceled",
        /// Still running.
        Processing = 3 => "Processing",
        /// Refunded.
        Refunded = 4 => "Refunded",
    }
}

int_enum! {
    /// Whether a transaction added to or took from the balance.
    pub enum ChargeType {
        /// A credit.
        Add = 0 => "Add",
        /// A debit.
        Subtract = 1 => "Subtract",
    }
}

model! {
    /// The reseller account and its balances.
    pub struct Account {
        /// The reseller ID (a UUID).
        "resellerId" => pub reseller_id: String,
        /// The reseller's name.
        "resellerName" => pub reseller_name: String,
        /// The reseller group.
        "resellerGroupName" => pub reseller_group_name: String,
        /// The balance in Turkish lira.
        "tryBalance" => pub try_balance: f64,
        /// The balance in US dollars.
        "usdBalance" => pub usd_balance: f64,
    }
}

impl Account {
    /// Returns the balance in a currency, `"USD"` or `"TRY"` (any case; `"TL"` works too), or `None` for other
    /// currencies: the account holds only these two.
    pub fn balance(&self, currency: &str) -> Option<f64> {
        match currency.trim().to_ascii_uppercase().as_str() {
            "USD" => Some(self.usd_balance),
            "TRY" | "TL" => Some(self.try_balance),
            _ => None,
        }
    }
}

model! {
    /// The profile of the reseller the credentials belong to.
    pub struct AccountProfile {
        /// The reseller ID (a UUID), the one the client takes.
        "id" => pub id: String,
        /// The reseller's numeric ID.
        "resellerAutoId" => pub reseller_auto_id: i64,
        /// The reseller's name.
        "name" => pub name: String,
        /// The company name.
        "companyName" => pub company_name: String,
        /// The reseller group.
        "resellerGroupName" => pub reseller_group_name: String,
        /// The currency the account is kept in.
        "accountCurrency" => pub account_currency: AccountCurrency,
        /// The account's state.
        "status" => pub status: ResellerStatus,
        /// Whether this is a sub-reseller account.
        "isSubReseller" => pub is_sub_reseller: bool,
        /// Whether the balance is suspended: the API refuses billable operations while it is set.
        "isBalanceSuspended" => pub is_balance_suspended: bool,
        /// Whether MCP access is enabled.
        "isMcpEnabled" => pub is_mcp_enabled: bool,
    }
}

model! {
    /// One entry of the balance ledger.
    pub struct Transaction {
        /// The transaction's trace ID.
        "traceId" => pub trace_id: String,
        /// What the transaction was for.
        "type" => pub transaction_type: TransactionType,
        /// The transaction's state.
        "status" => pub status: TransactionStatus,
        /// Credit or debit.
        "chargeType" => pub charge_type: ChargeType,
        /// The amount.
        "amount" => pub amount: f64,
        /// The currency of the amount.
        "amountCurrency" => pub amount_currency: String,
        /// The amount after currency exchange.
        "exchangeAmount" => pub exchange_amount: f64,
        /// The exchange currency, as the API's number.
        "exchangeCurrency" => pub exchange_currency: f64,
        /// The kind of product.
        "productType" => pub product_type: ProductType,
        /// The product's ID.
        "productId" => pub product_id: String,
        /// The description, usually with the domain name.
        "description" => pub description: String,
        /// The number of years bought, for domain transactions.
        "period" => pub period: i32,
        /// When the transaction was made.
        "creationTime" => pub creation_time: Option<DateTime<Utc>>,
        /// The reseller's name.
        "resellerName" => pub reseller_name: String,
        /// The balance after the transaction.
        "balance" => pub balance: f64,
        /// The balance before the transaction.
        "oldBalance" => pub old_balance: f64,
    }
}

impl Transaction {
    /// Returns how the transaction moved the balance: positive for credits, negative for charges. It is reliable
    /// even when `amount` is 0.
    pub fn change(&self) -> f64 {
        ((self.balance - self.old_balance) * 100.0).round() / 100.0
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Reads the account and its balances. CLI: `dna account get`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AccountGetRequest {}

impl AccountGetRequest {
    /// Creates the request.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Request for AccountGetRequest {
    type Response = Account;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/deposit/accounts/me")
    }
}

/// Reads the reseller's profile. CLI: `dna account profile`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AccountProfileRequest {}

impl AccountProfileRequest {
    /// Creates the request.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Request for AccountProfileRequest {
    type Response = AccountProfile;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/resellers/me")
    }
}

/// Lists balance transactions. CLI: `dna account transactions`
///
/// ```
/// use domainnameapi::account::{AccountTransactionsRequest, ChargeType, TransactionType};
///
/// let request = AccountTransactionsRequest::new()
///     .charge_type(ChargeType::Subtract)
///     .transaction_types([TransactionType::Register, TransactionType::Renew])
///     .limit(50);
/// ```
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AccountTransactionsRequest {
    /// Keeps only these kinds of transaction.
    pub transaction_types: Vec<TransactionType>,
    /// Keeps only these kinds of product.
    pub product_types: Vec<ProductType>,
    /// Keeps only credits or debits.
    pub charge_type: Option<ChargeType>,
    /// Keeps only these currencies, for example `"USD"`.
    pub currencies: Vec<String>,
    /// Keeps transactions made at or after this time.
    pub start_date: Option<DateTime<Utc>>,
    /// Keeps transactions made at or before this time.
    pub end_date: Option<DateTime<Utc>>,
    /// Finds one transaction by its trace ID.
    pub trace_id: String,
    /// Searches the description.
    pub description: String,
    /// Searches the text.
    pub filter_text: String,
    /// Keeps only transactions of this amount.
    pub amount: Option<f64>,
    /// The page.
    pub paging: Paging,
}

impl AccountTransactionsRequest {
    /// Lists every transaction, newest first.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps only these kinds of transaction.
    pub fn transaction_types(mut self, types: impl IntoIterator<Item = TransactionType>) -> Self {
        self.transaction_types = types.into_iter().collect();
        self
    }

    /// Keeps only these kinds of product.
    pub fn product_types(mut self, types: impl IntoIterator<Item = ProductType>) -> Self {
        self.product_types = types.into_iter().collect();
        self
    }

    /// Keeps only credits ([`ChargeType::Add`]) or debits ([`ChargeType::Subtract`]).
    pub fn charge_type(mut self, charge_type: ChargeType) -> Self {
        self.charge_type = Some(charge_type);
        self
    }

    /// Keeps only these currencies.
    pub fn currencies<S: Into<String>>(mut self, currencies: impl IntoIterator<Item = S>) -> Self {
        self.currencies = currencies.into_iter().map(Into::into).collect();
        self
    }

    /// Keeps transactions made at or after this time.
    pub fn start_date(mut self, start: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self
    }

    /// Keeps transactions made at or before this time.
    pub fn end_date(mut self, end: DateTime<Utc>) -> Self {
        self.end_date = Some(end);
        self
    }

    /// Finds one transaction by its trace ID.
    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }

    /// Searches the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Searches the text.
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Keeps only transactions of this amount.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = Some(amount);
        self
    }
}

paged!(AccountTransactionsRequest => Transaction, sortable);

impl Request for AccountTransactionsRequest {
    type Response = Page<Transaction>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/deposit/transactions")
    }

    fn query(&self) -> Query {
        let date = |t: Option<DateTime<Utc>>| t.map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true));
        let query = Query::new()
            .set_each("TransactionTypes", self.transaction_types.iter().map(|t| t.code()))
            .set_each("ProductTypes", self.product_types.iter().map(|t| t.code()))
            .set_each("AccountCurrencyTypes", self.currencies.iter().map(|c| c.trim().to_ascii_uppercase()))
            .set_opt("ChargeType", self.charge_type.map(ChargeType::code))
            .set_opt("StartDate", date(self.start_date))
            .set_opt("EndDate", date(self.end_date))
            .set_str("TraceId", &self.trace_id)
            .set_str("Description", &self.description)
            .set_str("FilterText", &self.filter_text)
            .set_opt("Amount", self.amount);
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            v.check(end >= start, "endDate", "must not be before startDate");
        }
    }
}

// ------------------------------------------------------------------------------------------------ client

/// Your account. Get it with [`Client::account`].
#[derive(Debug, Clone, Copy)]
pub struct AccountClient<'a> {
    client: &'a Client,
}

impl<'a> AccountClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        AccountClient { client }
    }

    /// Returns the account and its TRY and USD balances. It is also the cheapest way to check that the credentials
    /// work.
    pub async fn get(&self) -> Result<Account, Error> {
        self.client.send(&AccountGetRequest::new()).await
    }

    /// Returns the profile of the reseller the credentials belong to: names, reseller group, account currency and
    /// status.
    pub async fn profile(&self) -> Result<AccountProfile, Error> {
        self.client.send(&AccountProfileRequest::new()).await
    }

    /// Returns one page of the balance ledger.
    pub async fn transactions(&self, request: AccountTransactionsRequest) -> Result<Page<Transaction>, Error> {
        self.client.send(&request).await
    }

    /// Returns every matching transaction, fetching pages as the stream is read.
    pub fn all_transactions(&self, request: AccountTransactionsRequest) -> ItemStream<Transaction> {
        self.client.list_all(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn account_balance_by_currency() {
        let account: Account = serde_json::from_str(r#"{"tryBalance":"100.5","usdBalance":12}"#).unwrap();
        assert_eq!(account.balance("usd"), Some(12.0));
        assert_eq!(account.balance("TL"), Some(100.5));
        assert_eq!(account.balance("EUR"), None);
    }

    #[test]
    fn transaction_change() {
        let t: Transaction = serde_json::from_str(
            r#"{"type":2,"chargeType":1,"balance":90.1,"oldBalance":100.2,"creationTime":"2026-11-01T10:00:00"}"#,
        )
        .unwrap();
        assert_eq!(t.change(), -10.1);
        assert_eq!(t.transaction_type, TransactionType::Renew);
        assert_eq!(t.charge_type, ChargeType::Subtract);
        assert_eq!(t.creation_time, Some(Utc.with_ymd_and_hms(2026, 11, 1, 10, 0, 0).unwrap()));
    }

    #[test]
    fn transactions_query() {
        let from = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let request = AccountTransactionsRequest::new()
            .transaction_types([TransactionType::Register, TransactionType::Renew])
            .charge_type(ChargeType::Subtract)
            .currencies(["usd"])
            .start_date(from)
            .limit(20);
        let pairs: Vec<String> = request.query().pairs().iter().map(|(k, v)| format!("{k}={v}")).collect();
        assert_eq!(
            pairs,
            [
                "TransactionTypes=1",
                "TransactionTypes=2",
                "AccountCurrencyTypes=USD",
                "ChargeType=1",
                "StartDate=2026-01-01T00:00:00Z",
                "MaxResultCount=20"
            ]
        );

        let mut v = Validator::new();
        AccountTransactionsRequest::new()
            .start_date(from)
            .end_date(from - chrono::Duration::days(1))
            .limit(0)
            .validate(&mut v);
        assert_eq!(v.errors().len(), 2);
    }
}

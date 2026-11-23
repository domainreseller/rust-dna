//! The product catalogue and your prices: TLDs, SSL certificates and other products. CLI: `dna product`.

use crate::de::{int_enum, model};
use crate::request::paged;
use crate::{Client, Endpoint, Error, ItemStream, Page, Paging, Query, Request, Validator};
use futures_util::stream::TryStreamExt;
use std::fmt;

int_enum! {
    /// The kind of a product.
    pub enum ProductType {
        /// A domain name.
        Domain = 1 => "Domain",
        /// An SSL certificate.
        Ssl = 2 => "SSL",
        /// A balance deposit.
        Deposit = 3 => "Deposit",
        /// The trustee (local presence) service of a TLD.
        DomainTrustee = 4 => "DomainTrustee",
        /// Web hosting.
        Hosting = 5 => "Hosting",
        /// A registrant change.
        OwnerChange = 6 => "OwnerChange",
    }
}

int_enum! {
    /// The kind of an order, and the operation a price is for.
    pub enum OrderType {
        /// A new registration.
        Register = 1 => "Register",
        /// A renewal.
        Renew = 2 => "Renew",
        /// An incoming transfer.
        Transfer = 3 => "Transfer",
        /// A restore during the redemption period.
        Restore = 4 => "Restore",
        /// A transfer between sub-resellers.
        SubresellerTransfer = 5 => "SubresellerTransfer",
        /// A backorder.
        Backorder = 6 => "Backorder",
        /// A transfer between resellers of the platform.
        InternalTransfer = 7 => "InternalTransfer",
        /// A charge.
        Charge = 8 => "Charge",
        /// The trustee service.
        Trustee = 9 => "Trustee",
        /// Leasing.
        Leasing = 10 => "Leasing",
        /// A creation (SSL, hosting).
        Create = 11 => "Create",
        /// A registrant change.
        OwnerChange = 12 => "OwnerChange",
    }
}

model! {
    /// The price of a product for one period.
    pub struct Price {
        /// The number of years.
        "period" => pub period: i32,
        /// The amount.
        "price" => pub price: f64,
        /// The ISO 4217 code, for example `"USD"`.
        "currency" => pub currency: String,
    }
}

impl fmt::Display for Price {
    /// `11.99 USD`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.price, self.currency)
    }
}

model! {
    /// A product of the catalogue.
    pub struct Product {
        /// The product's ID.
        "id" => pub id: String,
        /// The parent product's ID.
        "parentProductId" => pub parent_product_id: String,
        /// The product name, for example a TLD without the dot.
        "productName" => pub name: String,
        /// The product code.
        "productCode" => pub product_code: String,
        /// The ID of the product's entity.
        "entityId" => pub entity_id: String,
        /// The kind of product.
        "productType" => pub product_type: ProductType,
    }
}

model! {
    /// A sub-product of [`ProductInfo`], with its buying and selling price.
    pub struct SubProduct {
        /// The parent product's ID.
        "parentProductId" => pub parent_product_id: String,
        /// The product name.
        "productName" => pub name: String,
        /// The product code.
        "productCode" => pub product_code: String,
        /// The kind of product.
        "productType" => pub product_type: ProductType,
        /// The ID of the product's entity.
        "entityId" => pub entity_id: String,
        /// What you pay.
        "buyingPrice" => pub buying_price: f64,
        /// The currency of the buying price.
        "buyingCurrency" => pub buying_currency: String,
        /// The selling price.
        "sellingPrice" => pub selling_price: f64,
        /// The currency of the selling price.
        "sellingCurrency" => pub selling_currency: String,
    }
}

model! {
    /// The price of a product for one order type and period.
    pub struct ProductInfo {
        /// The parent product's ID.
        "parentProductId" => pub parent_product_id: String,
        /// The product name.
        "productName" => pub name: String,
        /// The product code.
        "productCode" => pub product_code: String,
        /// The ID of the product's entity.
        "entityId" => pub entity_id: String,
        /// The kind of product.
        "productType" => pub product_type: ProductType,
        /// Your price.
        "sellingPrice" => pub selling_price: f64,
        /// The currency of the price.
        "sellingCurrency" => pub selling_currency: String,
        /// The parts of the product, with their prices.
        "subproducts" => pub subproducts: Vec<SubProduct>,
    }
}

model! {
    /// The prices of a TLD in one price group.
    pub struct TldPriceGroup {
        /// The price group.
        "priceGroup" => pub price_group: String,
        /// Registration prices per period.
        "register" => pub register: Vec<Price>,
        /// Renewal prices per period.
        "renew" => pub renew: Vec<Price>,
        /// Transfer prices per period.
        "transfer" => pub transfer: Vec<Price>,
        /// The restore price.
        "restore" => pub restore: Option<Price>,
        /// The refund amount.
        "refund" => pub refund: Option<Price>,
        /// The backorder price.
        "backorder" => pub backorder: Option<Price>,
    }
}

model! {
    /// One allowed value of a [`TldAttribute`].
    pub struct TldAttributeOption {
        /// The value to send.
        "value" => pub value: String,
    }
}

model! {
    /// A registry attribute of a TLD. Pass the required ones as `tld_attributes` when registering.
    pub struct TldAttribute {
        /// The attribute's key.
        "key" => pub key: String,
        /// What the attribute means.
        "description" => pub description: String,
        /// Whether a registration must include it.
        "isRequired" => pub required: bool,
        /// The value type.
        "type" => pub value_type: String,
        /// The allowed values, when the attribute has a fixed list.
        "options" => pub options: Vec<TldAttributeOption>,
    }
}

model! {
    /// A top-level domain you can sell: prices, periods, rules and the registry attributes it needs.
    pub struct Tld {
        /// The TLD without the leading dot, for example `"com"` or `"com.tr"`.
        "name" => pub name: String,
        /// The shortest registration period in years.
        "minRegistrationPeriod" => pub min_registration_period: i32,
        /// The longest registration period in years.
        "maxRegistrationPeriod" => pub max_registration_period: i32,
        /// The registration periods on offer, in years.
        "registrationPeriods" => pub registration_periods: Vec<i32>,
        /// The renewal periods on offer, in years.
        "renewalPeriods" => pub renewal_periods: Vec<i32>,
        /// The add grace period, in days.
        "addGracePeriod" => pub add_grace_period: i32,
        /// The auto-renew grace period, in days.
        "autoRenewGracePeriod" => pub auto_renew_grace_period: i32,
        /// The finalization period, in days.
        "finalizationPeriod" => pub finalization_period: i32,
        /// The failure period, in days.
        "failurePeriod" => pub failure_period: i32,
        /// The payment period, in days.
        "paymentPeriod" => pub payment_period: i32,
        /// The days a deleted domain can be restored.
        "deletionRestorablePeriod" => pub deletion_restorable_period: i32,
        /// The deletion hold period, in days.
        "deletionHoldPeriod" => pub deletion_hold_period: i32,
        /// The transfer period, in days.
        "transferPeriod" => pub transfer_period: i32,
        /// The register period, in days.
        "registerPeriod" => pub register_period: i32,
        /// The renewal period, in days.
        "renewalPeriod" => pub renewal_period: i32,
        /// Whether incoming transfers are supported.
        "isSupportTransfer" => pub supports_transfer: bool,
        /// Whether internationalised names are supported.
        "isSupportIdn" => pub supports_idn: bool,
        /// Whether the registry asks for documents.
        "isRequireDocument" => pub requires_document: bool,
        /// Whether the trustee service is offered.
        "isSupportTrustee" => pub supports_trustee: bool,
        /// Whether the trustee service is required.
        "isTrusteeRequired" => pub trustee_required: bool,
        /// Whether you may sell the TLD.
        "isSaleAllowed" => pub sale_allowed: bool,
        /// Whether a transfer lock is possible.
        "isTransferLockPossible" => pub transfer_lock_possible: bool,
        /// Whether contact handles can be updated.
        "isHandleUpdateAvailable" => pub handle_update_available: bool,
        /// Whether WHOIS privacy is available.
        "isPrivacyAvailable" => pub privacy_available: bool,
        /// Whether DNSSEC is available.
        "isDnssecAvailable" => pub dnssec_available: bool,
        /// Whether IPv6 glue records are accepted.
        "isIpv6Available" => pub ipv6_available: bool,
        /// Whether explicit renewals are supported.
        "isExplicitRenewalSupported" => pub explicit_renewal_supported: bool,
        /// Whether the TLD exists in the OTE test environment.
        "availableInTestEnvironment" => pub available_in_test_environment: bool,
        /// Whether a transfer needs an auth code.
        "isRequiredAuthCode" => pub auth_code_required: bool,
        /// Whether the registry checks the nameservers.
        "registryDnsCheck" => pub registry_dns_check: bool,
        /// Whether transfer checks are supported.
        "isTransferCheckSupported" => pub transfer_check_supported: bool,
        /// Whether glue records are allowed.
        "isGlueRecordAllowed" => pub glue_record_allowed: bool,
        /// Whether the registry uses host objects.
        "isHostObjectAccepted" => pub host_object_accepted: bool,
        /// The prices per price group.
        "prices" => pub prices: Vec<TldPriceGroup>,
        /// The registry attributes of the TLD.
        "attributes" => pub attributes: Vec<TldAttribute>,
    }
}

impl Tld {
    /// Returns the registration price for a period in years.
    pub fn register_price(&self, years: i32) -> Option<&Price> {
        self.prices.iter().flat_map(|g| &g.register).find(|p| p.period == years)
    }

    /// Returns the renewal price for a period in years.
    pub fn renew_price(&self, years: i32) -> Option<&Price> {
        self.prices.iter().flat_map(|g| &g.renew).find(|p| p.period == years)
    }

    /// Returns the transfer price for a period in years.
    pub fn transfer_price(&self, years: i32) -> Option<&Price> {
        self.prices.iter().flat_map(|g| &g.transfer).find(|p| p.period == years)
    }

    /// Returns the attributes a registration must include.
    pub fn required_attributes(&self) -> impl Iterator<Item = &TldAttribute> {
        self.attributes.iter().filter(|a| a.required)
    }
}

model! {
    /// The prices of an SSL product in one price group.
    pub struct SslPriceGroup {
        /// The price group.
        "priceGroup" => pub price_group: String,
        /// Purchase prices per period.
        "create" => pub create: Vec<Price>,
    }
}

model! {
    /// An SSL certificate product.
    pub struct SslProduct {
        /// The product name.
        "name" => pub name: String,
        /// The package code [`SslClient::initialize`](crate::ssl::SslClient::initialize) takes.
        "productCode" => pub product_code: String,
        /// The certificate authority.
        "vendorName" => pub vendor_name: String,
        /// The validation type (DV, OV, EV).
        "validationType" => pub validation_type: String,
        /// Whether the certificate covers `*.domain`.
        "isWildcard" => pub is_wildcard: bool,
        /// Whether the certificate covers several names.
        "isMultiDomain" => pub is_multi_domain: bool,
        /// The number of names included.
        "defaultSanCount" => pub default_san_count: i32,
        /// The largest number of names.
        "maxSanCount" => pub max_san_count: i32,
        /// The warranty.
        "warrantyAmount" => pub warranty_amount: f64,
        /// The usual issuance time, in minutes.
        "issueTimeMinutes" => pub issue_time_minutes: i32,
        /// Whether reissues are free.
        "isReissueFree" => pub is_reissue_free: bool,
        /// Whether a CSR is required.
        "requiresCsr" => pub requires_csr: bool,
        /// The prices per price group.
        "prices" => pub prices: Vec<SslPriceGroup>,
    }
}

// ------------------------------------------------------------------------------------------------ requests

/// Lists products. CLI: `dna product list`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ProductListRequest {
    /// Keeps products matching this text.
    pub filter_text: String,
    /// Keeps only products of this kind.
    pub product_type: Option<ProductType>,
    /// The page.
    pub paging: Paging,
}

impl ProductListRequest {
    /// Lists every product.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps products matching this text.
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Keeps only products of this kind.
    pub fn product_type(mut self, product_type: ProductType) -> Self {
        self.product_type = Some(product_type);
        self
    }
}

paged!(ProductListRequest => Product, sortable);

impl Request for ProductListRequest {
    type Response = Page<Product>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/products")
    }

    fn query(&self) -> Query {
        let query = Query::new()
            .set_str("FilterText", &self.filter_text)
            .set_opt("ProductType", self.product_type.map(ProductType::code));
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
    }
}

/// Reads the price of a product for an order type and period. CLI: `dna product info`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProductInfoRequest {
    /// A TLD without the dot (`"com"`, `"com.tr"`) or another product name.
    pub product_name: String,
    /// The operation the price is for.
    pub order_type: OrderType,
    /// The period in years, 1 to 10.
    pub years: u32,
}

impl ProductInfoRequest {
    /// The price of `product_name` for `order_type` and `years`.
    pub fn new(product_name: impl Into<String>, order_type: OrderType, years: u32) -> Self {
        ProductInfoRequest { product_name: product_name.into(), order_type, years }
    }
}

impl Request for ProductInfoRequest {
    type Response = ProductInfo;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/products/info")
    }

    fn query(&self) -> Query {
        Query::new()
            .set_str("ProductName", self.product_name.trim().trim_start_matches('.'))
            .set("OrderType", self.order_type.code())
            .set("Period", self.years)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("productName", &self.product_name);
        v.check(self.order_type.is_known(), "orderType", "must be a known OrderType such as OrderType::Renew");
        v.between("years", Some(self.years), 1, 10);
    }
}

/// Lists TLDs with prices and rules. CLI: `dna product tlds`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ProductTldsRequest {
    /// Keeps TLDs matching this text, for example `"com.tr"`.
    pub filter_text: String,
    /// The currency of the prices; empty uses the account's.
    pub currency: String,
    /// The page.
    pub paging: Paging,
}

impl ProductTldsRequest {
    /// Lists every TLD.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps TLDs matching this text.
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Sets the currency of the prices, for example `"USD"`.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }
}

paged!(ProductTldsRequest => Tld, sortable);

impl Request for ProductTldsRequest {
    type Response = Page<Tld>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/products/tlds")
    }

    fn query(&self) -> Query {
        let query = Query::new()
            .set_str("FilterText", &self.filter_text)
            .set_str("Currency", &self.currency.trim().to_ascii_uppercase());
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
    }
}

/// Lists SSL certificate products. CLI: `dna product ssls`
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ProductSslsRequest {
    /// Keeps products matching this text.
    pub filter_text: String,
    /// Keeps products of this certificate authority.
    pub vendor_name: String,
    /// The currency of the prices; empty uses the account's.
    pub currency: String,
    /// Keeps products of the API's numeric validation type.
    pub validation_type: Option<i32>,
    /// Keeps wildcard (or non-wildcard) products.
    pub is_wildcard: Option<bool>,
    /// Keeps multi-domain (or single-domain) products.
    pub is_multi_domain: Option<bool>,
    /// The page.
    pub paging: Paging,
}

impl ProductSslsRequest {
    /// Lists every SSL product.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps products matching this text.
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Keeps products of this certificate authority.
    pub fn vendor_name(mut self, vendor: impl Into<String>) -> Self {
        self.vendor_name = vendor.into();
        self
    }

    /// Sets the currency of the prices.
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }

    /// Keeps products of the API's numeric validation type.
    pub fn validation_type(mut self, validation_type: i32) -> Self {
        self.validation_type = Some(validation_type);
        self
    }

    /// Keeps wildcard (`true`) or non-wildcard (`false`) products.
    pub fn is_wildcard(mut self, wildcard: bool) -> Self {
        self.is_wildcard = Some(wildcard);
        self
    }

    /// Keeps multi-domain (`true`) or single-domain (`false`) products.
    pub fn is_multi_domain(mut self, multi_domain: bool) -> Self {
        self.is_multi_domain = Some(multi_domain);
        self
    }
}

paged!(ProductSslsRequest => SslProduct, sortable);

impl Request for ProductSslsRequest {
    type Response = Page<SslProduct>;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/products/ssls")
    }

    fn query(&self) -> Query {
        let query = Query::new()
            .set_str("FilterText", &self.filter_text)
            .set_str("VendorName", &self.vendor_name)
            .set_str("Currency", &self.currency.trim().to_ascii_uppercase())
            .set_opt("ValidationType", self.validation_type)
            .set_opt("IsWildcard", self.is_wildcard)
            .set_opt("IsMultiDomain", self.is_multi_domain);
        self.paging.apply(query)
    }

    fn validate(&self, v: &mut Validator) {
        self.paging.validate(v);
    }
}

// ------------------------------------------------------------------------------------------------ client

/// The product catalogue. Get it with [`Client::products`].
#[derive(Debug, Clone, Copy)]
pub struct ProductsClient<'a> {
    client: &'a Client,
}

impl<'a> ProductsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        ProductsClient { client }
    }

    /// Returns one page of products.
    pub async fn list(&self, request: ProductListRequest) -> Result<Page<Product>, Error> {
        self.client.send(&request).await
    }

    /// Returns every matching product, fetching pages as the stream is read.
    pub fn list_all(&self, request: ProductListRequest) -> ItemStream<Product> {
        self.client.list_all(request)
    }

    /// Returns the price of a product for an order type and period, for example the price of a 2-year `.com` renewal:
    /// `products().info("com", OrderType::Renew, 2)`.
    pub async fn info(&self, product_name: &str, order_type: OrderType, years: u32) -> Result<ProductInfo, Error> {
        self.client.send(&ProductInfoRequest::new(product_name, order_type, years)).await
    }

    /// Returns one page of the TLD catalogue. It is not rate limited; cache it rather than asking for every search.
    pub async fn tlds(&self, request: ProductTldsRequest) -> Result<Page<Tld>, Error> {
        self.client.send(&request).await
    }

    /// Returns the whole TLD catalogue (or the matching part), fetching pages as the stream is read.
    pub fn all_tlds(&self, request: ProductTldsRequest) -> ItemStream<Tld> {
        self.client.list_all(request)
    }

    /// Returns one TLD by name (`"com.tr"` or `".com.tr"`), or `None` when the catalogue does not have it.
    pub async fn tld(&self, name: &str) -> Result<Option<Tld>, Error> {
        let name = name.trim().trim_start_matches('.').to_ascii_lowercase();
        let tlds: Vec<Tld> = self.all_tlds(ProductTldsRequest::new().filter_text(name.clone())).try_collect().await?;
        Ok(tlds.into_iter().find(|t| t.name.eq_ignore_ascii_case(&name)))
    }

    /// Returns one page of SSL certificate products.
    pub async fn ssls(&self, request: ProductSslsRequest) -> Result<Page<SslProduct>, Error> {
        self.client.send(&request).await
    }

    /// Returns every matching SSL product, fetching pages as the stream is read.
    pub fn all_ssls(&self, request: ProductSslsRequest) -> ItemStream<SslProduct> {
        self.client.list_all(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_read_numbers_and_names() {
        assert_eq!(serde_json::from_str::<ProductType>("2").unwrap(), ProductType::Ssl);
        assert_eq!(serde_json::from_str::<ProductType>("\"Hosting\"").unwrap(), ProductType::Hosting);
        assert_eq!(serde_json::from_str::<ProductType>("\"5\"").unwrap(), ProductType::Hosting);
        assert_eq!(serde_json::from_str::<ProductType>("99").unwrap(), ProductType::Unknown(99));
        assert_eq!(serde_json::from_str::<ProductType>("null").unwrap(), ProductType::Unknown(-1));
        assert_eq!("renew".parse::<OrderType>().unwrap(), OrderType::Renew);
        assert_eq!("2".parse::<OrderType>().unwrap(), OrderType::Renew);
        assert!("Rent".parse::<OrderType>().is_err());
        assert_eq!(serde_json::to_string(&OrderType::Transfer).unwrap(), "3");
        assert_eq!(OrderType::Unknown(42).to_string(), "Unknown(42)");
    }

    #[test]
    fn tld_prices() {
        let tld: Tld = serde_json::from_value(serde_json::json!({
            "name": "com",
            "isDnssecAvailable": "true",
            "prices": [{"priceGroup": "A", "register": [{"period": 1, "price": "9.99", "currency": "USD"}],
                        "renew": [{"period": 2, "price": 20, "currency": "USD"}], "restore": null}],
            "attributes": [{"key": "x", "isRequired": 1}, {"key": "y", "isRequired": false}]
        }))
        .unwrap();
        assert!(tld.dnssec_available);
        assert_eq!(tld.register_price(1).unwrap().to_string(), "9.99 USD");
        assert_eq!(tld.renew_price(2).unwrap().price, 20.0);
        assert!(tld.transfer_price(1).is_none());
        assert!(tld.prices[0].restore.is_none());
        assert_eq!(tld.required_attributes().map(|a| a.key.as_str()).collect::<Vec<_>>(), ["x"]);
    }

    #[test]
    fn info_request_checks_its_input() {
        let mut v = Validator::new();
        ProductInfoRequest::new(" ", OrderType::Unknown(0), 0).validate(&mut v);
        assert_eq!(v.errors().len(), 3);
        let q = ProductInfoRequest::new(".com.tr", OrderType::Renew, 2).query();
        assert_eq!(q.pairs()[0].1, "com.tr");
    }
}

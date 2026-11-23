//! Orders and their lines. CLI: `dna order`.

use crate::de::{int_enum, model};
use crate::products::{OrderType, ProductType};
use crate::request::path_segment;
use crate::{Client, Endpoint, Error, Request, Validator};

int_enum! {
    /// The state of an order.
    pub enum OrderStatus {
        /// Completed.
        Active = 1 => "Active",
        /// Waiting.
        Pending = 2 => "Pending",
        /// Cancelled.
        Canceled = 3 => "Canceled",
        /// Being processed.
        InProgress = 4 => "InProgress",
    }
}

int_enum! {
    /// The state of one line of an order.
    pub enum OrderProductStatus {
        /// Ordered.
        Ordered = 1 => "Ordered",
        /// Cancelled.
        Canceled = 2 => "Canceled",
        /// Being processed.
        InProgress = 3 => "InProgress",
    }
}

model! {
    /// One line of an order.
    pub struct OrderProduct {
        /// The product code.
        "productCode" => pub product_code: String,
        /// The kind of product.
        "productType" => pub product_type: ProductType,
        /// The kind of order.
        "orderType" => pub order_type: OrderType,
        /// The line's state.
        "status" => pub status: OrderProductStatus,
        /// The price.
        "price" => pub price: f64,
        /// The currency of the price.
        "priceCurrency" => pub price_currency: String,
        /// The product name.
        "name" => pub name: String,
        /// The description.
        "description" => pub description: String,
        /// Extra data of the line.
        "additionalData" => pub additional_data: String,
        /// The period in years.
        "period" => pub period: i32,
    }
}

model! {
    /// An order and its lines.
    pub struct Order {
        /// The order code.
        "orderCode" => pub order_code: String,
        /// The order's state.
        "status" => pub status: OrderStatus,
        /// The total amount.
        "amount" => pub amount: f64,
        /// The currency of the amount.
        "amountCurrency" => pub amount_currency: String,
        /// The lines of the order.
        "products" => pub products: Vec<OrderProduct>,
    }
}

/// Reads an order. CLI: `dna order get`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct OrderGetRequest {
    /// The order code.
    pub order_code: String,
}

impl OrderGetRequest {
    /// Reads the order with this code.
    pub fn new(order_code: impl Into<String>) -> Self {
        OrderGetRequest { order_code: order_code.into() }
    }
}

impl Request for OrderGetRequest {
    type Response = Order;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get(format!("/api/v1/order/{}", path_segment(&self.order_code)))
    }

    fn validate(&self, v: &mut Validator) {
        v.required("orderCode", &self.order_code);
    }
}

/// Orders. Get it with [`Client::orders`].
#[derive(Debug, Clone, Copy)]
pub struct OrdersClient<'a> {
    client: &'a Client,
}

impl<'a> OrdersClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        OrdersClient { client }
    }

    /// Returns an order and its lines.
    pub async fn get(&self, order_code: &str) -> Result<Order, Error> {
        self.client.send(&OrderGetRequest::new(order_code)).await
    }
}

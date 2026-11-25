<div align="center">
  <a href="README-TR.md"   >   TR <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/TR.png" alt="TR" height="20" /></a>
  <a href="README.md"> | EN <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/US.png" alt="EN" height="20" /></a>
   <a href="README-DE.md"> | DE <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/DE.png" alt="DE" height="20" /></a>
  <a href="README-SA.md"> | SA <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/SA.png" alt="AR" height="20" /></a>
  <a href="README-NL.md"> | NL <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/NL.png" alt="NL" height="20" /></a>
  <a href="README-AZ.md"> | AZ <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/AZ.png" alt="AZ" height="20" /></a>
  <a href="README-CN.md"> | CN <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/CN.png" alt="CN" height="20" /></a>
  <a href="README-FR.md"> | FR <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/FR.png" alt="FR" height="20" /></a>
  <a href="README-IT.md"> | IT <img style="padding-top: 8px" src="https://raw.githubusercontent.com/yammadev/flag-icons/master/png/IT.png" alt="IT" height="20" /></a>
</div>

# DomainNameAPI Rust SDK (rust-dna)

<p>
  <a href="https://github.com/domainreseller/rust-dna/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/domainreseller/rust-dna/ci.yml?branch=main&label=CI&style=flat-square" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-0B8DB5?style=flat-square" alt="Apache-2.0"></a>
  <img src="https://img.shields.io/badge/Rust-1.83%2B-0B8DB5?style=flat-square" alt="Rust 1.83+">
</p>

[Domain Name API](https://www.domainnameapi.com) 经销商平台的官方 Rust SDK。它用带类型的请求和模型覆盖了 REST API 的全部
67 个操作：域名、联系人、DNS、DNSSEC、SSL 证书、主机、订单、产品和你的账户。

```rust
let client = Client::builder().credentials(reseller_id, api_key).environment(Environment::Ote).build()?;

let search = client.domains().search("example.com").await?;
if search.info.is_available() {
    let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact), ["tr.apiname.com", "eu.apiname.com"]);
    client.domains().register_with_contacts(request).await?;
}
```

- **处处有类型。** 每个操作都是一个请求类型，每个响应都是一个带文档的 struct。没有 Map，也不用解析 JSON。
- **自定义请求。** 每个请求都实现了 `Request` trait。SDK 尚未覆盖的端点，可以自己实现该 trait 来调用。
- **涉及资金时更安全。** 每个请求都会声明自己是读取、写入、扣除余额还是删除数据。需付费的请求绝不会重试，只读模式会阻止一切更改。
- **内置容错。** 自动遵守速率限制，429 响应会重试，读取请求遇到网络错误也会重试。
- **清晰的错误。** 只有一种 `Error` 类型，带有稳定的错误类别、API 的错误代码、字段级验证错误、关联 ID 和提示。
- **现代 Rust。** 基于 Tokio 的异步、使用 rustls 的 `reqwest`（不依赖 OpenSSL）、serde 模型、以 `Stream` 形式惰性分页、可克隆的 `Send + Sync` 客户端，以及 `#![deny(missing_docs)]`。
- **注重隐私。** 没有遥测。除了你发往 API 的请求之外，不会有任何数据离开你的进程。

它是 [`dna`（Domain Name API CLI）](https://github.com/domainreseller/cli) 的 Rust 对应版本：使用同一份 API
文档、同样的操作和分组、同样的安全类别、重试规则和错误代码。

## 目录

- [安装](#安装)
- [凭据](#凭据)
- [快速开始](#快速开始)
- [配置](#配置)
- [用法](#用法)：[账户](#账户) · [域名](#域名) · [联系人](#联系人) · [DNS](#dns) · [DNSSEC](#dnssec) · [SSL](#ssl) · [主机](#主机) · [订单与产品](#订单与产品)
- [请求与 `Request` trait](#请求与-request-trait)
- [错误](#错误)
- [安全、重试与速率限制](#安全重试与速率限制)
- [异步代码与阻塞代码](#异步代码与阻塞代码)
- [测试你的代码](#测试你的代码)
- [项目结构](#项目结构)
- [开发](#开发)
- [从 1.x 升级](#从-1x-升级)

## 安装

> [!IMPORTANT]
> **请始终使用[最新版本](https://github.com/domainreseller/rust-dna/releases/latest)。** 发布包均带有版本号并经过测试。
> 绿色的 **Code → Download ZIP** 按钮下载的是开发分支。

```toml
[dependencies]
domainnameapi = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"   # 用于读取 list_all 返回的流
```

在该 crate 发布到 crates.io 之前，请从发布标签获取：

```toml
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v2.0.0" }
```

**要求：** Rust 1.83 或更高版本，以及一个 Tokio 运行时。依赖为 `reqwest`（使用 rustls，不依赖 OpenSSL）、
`serde`/`serde_json`、`tokio`（仅用于计时器）、`futures-util`、`chrono`、`base64` 和 `httpdate`。

## 凭据

API 用两个值验证每个请求，SDK 会替你发送它们：

| 值 | 请求头 | 在哪里找到 |
|---|---|---|
| Reseller ID（一个 UUID） | `__reseller` | 经销商面板 → **API 设置** |
| API Key | `X-API-KEY` | 经销商面板 → **API 设置** |

> [!NOTE]
> 这些是 **API 凭据**：你的面板登录信息在这里无法使用。**正式环境和 OTE（测试环境）使用
> 不同的 API Key。** 混用会失败，并返回 `INVALID_API_KEY` 或 `INVALID_RESELLER`。

## 快速开始

```rust
use domainnameapi::contacts::Contact;
use domainnameapi::domains::{DomainContacts, DomainListRequest, DomainRegisterWithContactsRequest};
use domainnameapi::{Client, Environment};
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    // OTE 是测试环境：真实的 API，独立的数据，不涉及真实资金。
    let client = Client::builder()
        .credentials("your-reseller-id", "your-api-key")
        .environment(Environment::Ote)
        .build()?;

    // 1. 检查凭据和余额。
    let account = client.account().get().await?;
    println!("{}: {} USD", account.reseller_name, account.usd_balance);

    // 2. 该域名是否可注册，价格是多少？
    let search = client.domains().search("my-new-domain.com").await?;
    println!("{} {} {}", search.info.status, search.info.price, search.info.currency);

    // 3. 用同一个联系人担任全部四种角色来注册。这会扣除余额。
    let contact = Contact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        state: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        ..Default::default()
    };
    let request = DomainRegisterWithContactsRequest::new(
        "my-new-domain.com",
        DomainContacts::same(contact),
        ["tr.apiname.com", "eu.apiname.com"],
    );
    let result = client.domains().register_with_contacts(request).await?;
    println!("{} {}, expires {:?}", result.success, result.status, result.expiration_date);

    // 4. 列出所有域名；读取流时才会获取各页。
    let mut domains = client.domains().list_all(DomainListRequest::new());
    while let Some(domain) = domains.try_next().await? {
        println!("{} expires in {} days", domain.domain_name, domain.remaining_days);
    }
    Ok(())
}
```

[examples](examples) 文件夹中有针对这些场景的可运行程序（`quickstart`、`register`、`dns`、`ssl`、
`custom_request`、`error_handling`）。它们读取 `DNA_*` 环境变量，因此请将它们指向 OTE：

```bash
export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
cargo run --example quickstart -- example.com
```

## 配置

创建客户端有三种方式：

```rust
// 正式环境，默认设置
let client = Client::new(reseller_id, api_key)?;

// 全部设置
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .environment(Environment::Ote)
    .timeout(Duration::from_secs(30))
    .read_only(true)
    .build()?;

// 与 dna CLI 读取的环境变量相同 ...
let client = Client::from_env()?;

// ... 再加上你自己的设置
let client = ClientBuilder::from_env()?.http_client(my_reqwest_client).build()?;
```

| `ClientBuilder` 方法 | 默认值 | 含义 |
|---|---|---|
| `credentials(reseller_id, api_key)` | （必填） | 凭据。其中任一为空时，`build()` 会以 `NO_CREDENTIALS` 失败。 |
| `environment(...)` | `Production` | `Production`（`https://api.domainresellerapi.com`）或 `Ote`（`https://ote.domainresellerapi.com`）。 |
| `base_url(...)` | 无 | 覆盖环境的地址，例如用于代理。路径前缀会被保留。 |
| `timeout(...)` | 60 秒 | 单次尝试的时间限制。 |
| `max_retries(...)` | 2 | 失败的请求重复多少次（见[重试](#安全重试与速率限制)）。0 表示关闭重试。 |
| `max_retry_delay(...)` | 30 秒 | 重试前的最长等待时间。 |
| `pace_requests(...)` | `true` | 控制请求间隔，使其保持在网关的速率限制之内。 |
| `read_only(...)` | `false` | 在发送之前阻止所有非读取请求。 |
| `user_agent(...)` | `DomainNameAPI-Rust/<version>` | `User-Agent` 请求头。 |
| `app(name, version)` | `rust`、SDK 版本 | `X-App` 和 `X-Version` 请求头：使用 SDK 的应用（见下文）。 |
| `http_client(...)` | SDK 自带的客户端 | 你自己的 `reqwest::Client`（代理、TLS 设置）。 |

`from_env()` 会读取：

| 变量 | 含义 |
|---|---|
| `DNA_RESELLER_ID`、`DNA_API_KEY` | 凭据。 |
| `DNA_ENV` | `production` 或 `ote`（也可用 `prod`、`live`、`test`、`sandbox`）。 |
| `DNA_TEST_MODE` | `true`、`1`、`on` 或 `yes` 表示选择 OTE。未设置 `DNA_ENV` 时使用。 |
| `DNA_BASE_URL` | 可选，覆盖基础地址。 |
| `DNA_READ_ONLY` | `true` 表示开启只读模式。 |
| `DNA_APP`, `DNA_APP_VERSION` | 使用 SDK 的应用及其版本，作为 `X-App` 和 `X-Version` 发送。 |

`Client` 实现了 `Clone + Send + Sync`，克隆开销很小（它内部持有一个 `Arc`）。每组凭据创建一个客户端，并在应用的整个
生命周期内共享：请求间隔控制是按客户端进行的。

### 标识你的应用

每个请求都带有 `X-App` 和 `X-Version` 请求头，用来告诉 API 团队是哪个集成发出的请求。默认值是 `rust` 和 SDK 版本。
如果你基于 SDK 开发面板模块或其他产品，请设置你自己的值：

```rust
let client = Client::builder()
    .credentials(reseller_id, api_key)
    .app("WiseCP5", "V2.0.20-beta") // 发送为 X-App: wisecp5、X-Version: 2.0.20
    .build()?;
```

名称会转为小写，只保留 `a-z`、`0-9`、`.`、`_` 和 `-`；版本只保留数字和点。名称为空时回退为 `rust`，版本为空时回退为
SDK 版本。`from_env()` 从 `DNA_APP` 和 `DNA_APP_VERSION` 读取相同的值。`User-Agent` 请求头不受影响。

## 用法

API 的每个领域都有各自的客户端，命名与 CLI 的命令组一致：`client.account()`、
`client.contacts()`、`client.domains()`（含 `forwarding()`、`child_name_servers()` 和 `documents()`）、
`client.dns()`、`client.dnssec()`、`client.ssl()`、`client.hosting()`、`client.orders()` 和 `client.products()`。
必填输入作为参数传入；带有可选输入的操作接受一个用 `new(...)` 构建、再链式调用 setter 的请求。
会扣除余额的方法会在其文档中注明。

### 账户

```rust
let account = client.account().get().await?;          // reseller_id, reseller_name, try_balance, usd_balance
let profile = client.account().profile().await?;      // name, company_name, account_currency, status, ...

// 本月的扣费记录，跨所有分页
let request = AccountTransactionsRequest::new()
    .charge_type(ChargeType::Subtract)
    .start_date("2026-11-01T00:00:00Z".parse()?)
    .transaction_types([TransactionType::Register, TransactionType::Renew]);
let mut transactions = client.account().all_transactions(request);
while let Some(t) = transactions.try_next().await? {
    println!("{:?} {} {} {}", t.creation_time, t.transaction_type, t.amount, t.amount_currency);
}
```

### 域名

**搜索**

```rust
let one = client.domains().search("example.com").await?;
let available = one.success && one.info.is_available();

let many = client.domains().bulk_search(["example.com", "example.net", "example.com.tr"]).await?;
for domain in many.available() {
    println!("{}: {} {}", domain.domain_name, domain.price, domain.currency);
}

// 指定币种
client.domains().search_with(DomainSearchRequest::new("example.com").currency("USD")).await?;
```

**列表与详情**

```rust
let page = client.domains().list(DomainListRequest::new().limit(50).statuses([DomainStatus::Active])).await?;

let all: Vec<Domain> = client.domains().list_all(DomainListRequest::new().domain_name("example")).try_collect().await?;

let info = client.domains().info("example.com").await?;
println!("{} {:?}", info.auth_code, info.expiration_date);
if let Some(registrant) = info.contact(ContactType::Registrant) {
    println!("{}", registrant.email);
}
```

**注册、续费、转移、恢复**（这些操作会扣除你的余额）

```rust
// 同一个联系人担任全部四种角色，两年
let request = DomainRegisterWithContactsRequest::new("example.com", DomainContacts::same(contact.clone()), ["tr.apiname.com", "eu.apiname.com"])
    .years(2);
client.domains().register_with_contacts(request).await?;

// 每个角色使用不同的联系人、注册局属性、受托服务
let request = DomainRegisterWithContactsRequest::new(
    "example.com.tr",
    DomainContacts { registrant: Some(owner), admin: Some(staff.clone()), tech: Some(staff.clone()), billing: Some(staff) },
    ["tr.apiname.com", "eu.apiname.com"],
)
.tld_attribute("<key>", "<value>"); // 可用的键：client.products().tld("com.tr").await?.unwrap().attributes

// 使用已有的联系人句柄
let request = DomainRegisterRequest::new("example.com", ContactHandles::same("DNA-12345"), ["tr.apiname.com", "eu.apiname.com"]);
client.domains().register(request).await?;

let renewal = client.domains().renew_check("example.com", Some(1)).await?;   // 免费
if renewal.renewable {
    client.domains().renew("example.com", 1).await?;
}

let check = client.domains().transfer_check("example.com", "EPP-CODE").await?;  // 免费
if check.transferable {
    client.domains().transfer(DomainTransferRequest::new("example.com", "EPP-CODE")).await?;
}

client.domains().restore("example.com").await?;
```

请求在发送之前会先经过检查。缺少四种联系人角色之一的注册、没有电子邮件地址的联系人，或注册期限为 0 年，
都会返回一个列出全部问题的 `ErrorKind::InvalidInput` 错误，并且不会向 API 发送任何内容。

> [!NOTE]
> 对于注册、续费或转移，API 可能返回 HTTP 200 且 `"success": false`（例如注册局拒绝该域名时）。
> 与 Java SDK 一样，SDK 会原样返回该响应：请检查 `DomainOperationResult::success` 和 `message`。
> `SslOperationResult` 和 `ContactCreateResult` 同样如此。

**设置**

```rust
client.domains().lock("example.com").await?;
client.domains().unlock("example.com").await?;
client.domains().set_privacy("example.com", true).await?;
client.domains()
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net", "ns2.example.net"]))
    .await?;
client.domains()   // 已启用 DNSSEC 签名的域名
    .set_name_servers(DomainNameServersSetRequest::new("example.com", ["ns1.example.net"]).dnssec_mode(DnssecMode::DisableDnssec))
    .await?;
let auth_code = client.domains().update_auth_code("example.com").await?;

client.domains().update_contacts("example.com", DomainContacts::same(contact)).await?;
client.domains().set_contact_handles("example.com", ContactHandles { tech: "DNA-12345".into(), ..Default::default() }).await?;
```

**URL 转发、子名称服务器与文档**

```rust
client.domains().forwarding().set("example.com", "https://www.example.net", ForwardType::Standard).await?;
let forwarding = client.domains().forwarding().get("example.com").await?;   // 未设置时为 None
client.domains().forwarding().delete("example.com").await?;

// 每个地址的 IP 版本会被自动识别
client.domains().child_name_servers().add("example.com", "ns1.example.com", ["192.0.2.10", "2001:db8::10"]).await?;
client.domains().child_name_servers()
    .update(DomainHostUpdateRequest::new("example.com", "ns1.example.com", ["192.0.2.20"]))
    .await?;
client.domains().child_name_servers().delete("example.com", "ns1.example.com").await?;

// 注册局文档（主要用于 .tr）
let document = DocumentUpload::from_file(DocumentType::IdentityCard, "id.pdf")?;
client.domains().documents().upload("example.com.tr", [document]).await?;
let documents: Vec<Document> = client.domains().documents().list_all("example.com.tr").try_collect().await?;
```

### 联系人

联系人句柄（handle）是可重复使用的联系人。创建一次，即可用于多个域名。

```rust
let created = client.contacts().create(contact.clone()).await?;
let handle = created.handle;                                         // 例如 DNA-12345

let details = client.contacts().get(&handle).await?;                 // 联系人及其验证状态
let result = client.contacts().update(&handle, contact).await?;
println!("{} of {} domains updated", result.stats.success_count, result.stats.total_domain_count);

let verifications = client.contacts().verification(&handle).await?;
client.contacts().resend_verification(&handle).await?;
```

### DNS

使用平台名称服务器的域名的区域记录。只有当域名使用这些名称服务器后，记录才会在公网上解析，
这一步由 `apply` 负责。

```rust
for set in client.dns().list("example.com").await? {
    println!("{} {} {} {:?}", set.name, set.record_type, set.ttl, set.values());
}

let dns = client.dns();
dns.add(DnsAddRequest::new("example.com", "www", RecordType::A, ["192.0.2.10"])).await?;
dns.add(DnsAddRequest::new("example.com", "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;   // "txt" 即 TXT
dns.add(DnsAddRequest::new("example.com", "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"]).ttl(3600)).await?;
dns.update(DnsUpdateRequest::new("example.com", "www", RecordType::A, ["192.0.2.20"]).ttl(300)).await?;
dns.delete(DnsDeleteRequest::new("example.com", "www", RecordType::A, "192.0.2.20")).await?;

let applied = dns.apply("example.com").await?;
```

SDK 会提前检查 API 会拒绝的情况：根域上的 CNAME、缺少优先级的 MX、缺少三个部分的 CAA 值、不是 IPv4 的 A 记录、
超出 1–86400 范围的 TTL。区域调用之间会保持 1 秒间隔，这是网关的限制。OTE 测试环境对区域调用返回 HTTP 500，
因此 DNS 相关代码只能在正式环境中试用。

### DNSSEC

```rust
let info = client.dnssec().get("example.com").await?;      // status, ds_records, allowed_algorithms, ...

let record: DsData = "12345 13 2 49FD46E6C4B45C55D4AC69CBD3CD34AC1AFE51DE".parse()?;
client.dnssec().add("example.com", [record.clone()]).await?;
client.dnssec().remove("example.com", record).await?;
client.dnssec().disable("example.com").await?;            // 删除所有 DS 记录
```

### SSL

一个域名可以有多张证书，因此已有的证书通过**证书 ID**（一个 GUID）来指定，而不是域名。
`initialize` 和 `renew` 会返回该 ID：请保存它。`list` 可以查到你已有证书的 ID。

```rust
// 1. 购买（扣除余额）。套餐代码来自 client.products().ssls(...)。
let approvers = client.ssl().approvers(SslApproversRequest::for_purchase("example.com", "PKG-CODE")).await?;
let order = client.ssl().initialize("example.com", "PKG-CODE").await?;
let certificate_id = order.certificate_id;                  // 请保存：之后的每个调用都需要它

// 2. 如果还没有 CSR 和私钥，先生成
let csr = client.ssl().generate_csr(
    SslGenerateCsrRequest::new("example.com")
        .country("TR").state("Istanbul").locality("Istanbul").organization("Example Ltd").organization_unit("IT"),
).await?;

// 3. 在 15 天内完成配置：CSR、验证方式和联系人
client.ssl().configure(
    SslConfigureRequest::new(&certificate_id, &csr.csr, certificate_contact.clone())
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0])
        .web_server_type(WebServerType::Apache),
).await?;

// 4. 等待签发，然后下载
let status = client.ssl().status(&certificate_id).await?;
if status.is_issued {
    let content = client.ssl().content(&certificate_id).await?;
    std::fs::write("fullchain.pem", content.full_chain())?;
}

let details = client.ssl().get(&certificate_id).await?;     // 订阅、配置、联系人
client.ssl().reissue(SslReissueRequest::new(&certificate_id).csr(new_csr).private_key(new_key)).await?;
client.ssl().revoke(&certificate_id).await?;                // 无法撤销
```

**列出证书并查找 ID**

```rust
let page = client.ssl().list(SslListRequest::new().domain_name("example.com")).await?;   // 最新的在前

let mut all = client.ssl().list_all(SslListRequest::new());
while let Some(c) = all.try_next().await? {
    println!("{} {} {} {:?}", c.certificate_id, c.domain_name, c.status, c.subscription_expires_at);
}

// 已有证书的审批人邮箱地址
let mails = client.ssl().approvers(SslApproversRequest::for_certificate(&certificate_id)).await?;
```

**续费**（此操作会扣除你的余额）

```rust
// 从 details.subscription.expires_at 之前 30 天起允许续费。续费得到的是一张拥有自己 ID 的新证书。
let renewal = client.ssl().renew(&certificate_id).await?;
client.ssl().configure(
    SslConfigureRequest::new(&renewal.certificate_id, &new_csr, certificate_contact)
        .authentication_type(AuthenticationType::Email)
        .approver_mail(&approvers[0]),
).await?;
```

购买或续费的证书必须在 15 天内完成配置，否则会被取消。续费时先调用 `renew`，再用新的 ID 调用 `configure`。
不要用 `initialize` 再买一次：那是按购买价格进行的另一笔购买，旧证书也永远不会被标记为已续费。
为空或不是 GUID 的证书 ID（例如旧代码传入的域名）会在发送任何内容之前被拒绝。

### 主机

```rust
let packages = client.hosting().packages().await?;
let created = client.hosting().create("example.com", "starter-linux").await?;          // 扣除余额
let status = client.hosting().status(HostingStatusRequest::by_service_id(&created.service_id)).await?;
if status.is_active() {
    println!("{} {}", status.panel_address, status.username);
}
let hosting = client.hosting().get(HostingGetRequest::by_domain("example.com")).await?;
let plans = client.hosting().plans(None).await?;
```

### 订单与产品

```rust
let order = client.orders().get("ORDER-CODE").await?;

if let Some(com) = client.products().tld("com").await? {
    println!("{:?}, DNSSEC: {}", com.register_price(1), com.dnssec_available);
}

let mut tlds = client.products().all_tlds(ProductTldsRequest::new().currency("USD"));
while let Some(tld) = tlds.try_next().await? {
    println!("{}", tld.name);
}

let price = client.products().info("com", OrderType::Renew, 1).await?;
let wildcard = client.products().ssls(ProductSslsRequest::new().is_wildcard(true)).await?;
```

## 请求与 `Request` trait

每个操作都是一个以其 CLI 命令命名的请求类型：`dna domain renew` 对应 `DomainRenewRequest`，
`dna domain renew-check` 对应 `DomainRenewCheckRequest`，`dna dns add` 对应 `DnsAddRequest`。各领域的方法会替你构建它们，
你也可以自己创建并发送：

```rust
let request = DomainRenewRequest::new("example.com", 2);
let endpoint = request.endpoint();
println!("{endpoint} is {}", endpoint.safety);   // "POST /api/v1/domains/renew is billable"

let result = client.send(&request).await?;

// 同样的调用，附带 HTTP 详情：关联 ID、软配额状态、尝试次数、耗时
let response = client.send_with_response(&request).await?;
println!("{} quota {:?}", response.correlation_id, response.quota);
```

Rust 没有继承；每个请求的基础是一个 trait：

| Trait | 用途 |
|---|---|
| `Request` | 所有操作。`type Response` 是响应类型；不返回数据的操作（锁定、解锁、添加 DNS 记录等）使用 `Empty`。 |
| `PagedRequest<T>` | 列表操作（`Response = Page<T>`）。可访问 `Paging`（skip、limit、sort），并可配合 `Client::list_all` 使用。 |

请求通过 `endpoint()` 描述自己：方法、路径、安全类别，以及是否幂等。另外三个方法有默认实现：`query()` 返回查询字符串
（`Query` 会省略空值），`body()` 返回 JSON 请求体（`JsonBody` 可以构建请求体并省略空值，也可以使用 `serde_json::json!`），
`validate()` 在发送前收集问题。要调用 SDK 尚未覆盖的端点，请编写你自己的请求：

```rust
use domainnameapi::{Endpoint, Query, Request, Validator};
use serde::Deserialize;

struct DomainNotesRequest {
    domain_name: String,
}

#[derive(Deserialize)]
struct DomainNotes {
    notes: Vec<String>,
}

impl Request for DomainNotesRequest {
    type Response = DomainNotes;

    fn endpoint(&self) -> Endpoint {
        Endpoint::get("/api/v1/domains/notes")          // GET 是读取
    }

    fn query(&self) -> Query {
        Query::new().set_str("DomainName", &self.domain_name)
    }

    fn validate(&self, v: &mut Validator) {
        v.required("domainName", &self.domain_name);
    }
}

let notes = client.send(&DomainNotesRequest { domain_name: "example.com".into() }).await?;
```

你的请求可以获得内置请求的一切能力：凭据、按安全类别执行的重试、请求间隔控制、只读模式和
错误处理。对于更改操作，请使用 `Endpoint::post(path, OperationSafety::Write)`（或 `Billable`、`Destructive`），
只有当发送两次也无害时才添加 `.idempotent()`。

## 错误

每个失败的请求都会返回一个 `Error`，即使请求未通过检查、没有发送任何内容时也是如此。

```rust
use domainnameapi::ErrorKind;

match client.domains().renew("example.com", 1).await {
    Ok(result) => println!("{} {}", result.success, result.message),
    Err(e) if e.kind() == ErrorKind::Validation => {
        println!("{}: {}", e.code(), e.message());   // Dna.DomainService:Domain:10007: ...
        for field in e.validation_errors() {
            println!("  {field}");
        }
    }
    Err(e) if e.is_outcome_unknown() => {
        // 一个需付费或会更改数据的请求在网络上失败：它可能已经到达 API。
        // 再次发送之前请先检查状态。
        let info = client.domains().info("example.com").await?;
    }
    Err(e) => return Err(e.into()),
}
```

| 方法 | 含义 |
|---|---|
| `kind()` | 稳定的分类（见下表）。请根据它分支处理，而不是根据消息。 |
| `code()` | API 的错误代码（如 `Dna.DomainService:Domain:10007`），或下面列出的 SDK 自身的代码之一。常用代码在 `domainnameapi::codes` 中定义为常量。 |
| `message()` | 发生了什么。模板式的消息会被替换为 API 给出的详情。 |
| `status()` | HTTP 状态码；没有收到响应时为 `None`。 |
| `details()`、`validation_errors()` | 附加详情和字段级问题（来自 API，或来自发送前的检查）。 |
| `correlation_id()` | 网关的 `X-Correlation-Id`。联系支持人员时请提供它。 |
| `hint()` | 关于下一步该怎么做的简短建议。 |
| `is_retryable()` | 稍后重复同一请求是否可能成功。 |
| `is_outcome_unknown()` | 请求可能已经执行：一个更改操作在网络上失败，或者 API 表示注册局可能已经执行了它。 |
| `is_not_found()`、`is_insufficient_balance()` | 常见情况的快捷方法。 |
| `operation()`、`safety()` | 失败的请求（`POST /api/v1/domains/renew`）及其安全类别。 |
| `retry_after()`、`response_body()` | 网关的 `Retry-After` 和原始响应。 |

`Error` 实现了 `std::error::Error` 和 `Display`（`[CODE] message (HTTP 400); field: problem`），因此可以配合
`?`、`anyhow` 和 `thiserror` 使用。

| `kind()` | 何时出现 |
|---|---|
| `Auth` | 凭据错误或缺失，或者该经销商在此环境中未知或已被停用（401、重定向）。 |
| `Forbidden` | 已通过身份验证，但无权执行（403）。 |
| `NotFound` | 域名、联系人、证书、订单或其他资源不存在（404）。 |
| `Validation` | API 拒绝了输入，或业务规则未通过：字段错误、域名已锁定、余额不足（400、422）。 |
| `Conflict` | 资源的状态不允许该操作（409）。 |
| `RateLimited` | 自动重试后仍受速率限制（429）。 |
| `Server` | 服务器端故障（5xx）。通常是暂时的。 |
| `Network`、`Timeout` | 没有响应：DNS、TLS、代理、连接被重置，或在超时时间内没有响应。 |
| `InvalidInput` | 请求未通过检查，或某项客户端设置无效；未发送任何内容。 |
| `ReadOnly` | 被只读模式阻止；未发送任何内容。 |
| `Unknown` | 其他任何情况，包括 SDK 无法读取的响应。 |

`ErrorKind::as_str()` 给出 CLI 使用的名称（`auth`、`not_found`、`rate_limited` 等）。SDK 自行设置的代码：

| 代码 | 含义 |
|---|---|
| `INVALID_RESELLER` | 网关重定向到了它的停用页面：该 Reseller ID 在**此环境中**未知、未激活或已被停用。 |
| `INVALID_API_KEY` | 该密钥与经销商不匹配。正式环境和 OTE 的密钥不同。 |
| `MISSING_API_KEY`、`MISSING_RESELLER_ID`、`NO_CREDENTIALS` | 某个凭据为空。 |
| `SUBRESELLER_NOT_ALLOWED` | 子经销商账户不能使用 API。请使用主经销商的凭据。 |
| `UNAUTHORIZED` | 其他任何 401。 |
| `RATE_LIMITED` | 重试之后仍返回 429。 |
| `NETWORK`、`TIMEOUT` | 没有收到响应。 |
| `READ_ONLY` | 被只读模式阻止。 |
| `INVALID_INPUT`、`INVALID_CONFIGURATION` | 某个请求未通过检查，或某项客户端设置无效。 |
| `EMPTY_RESPONSE`、`INVALID_RESPONSE` | 成功的响应为空或无法读取。 |
| `HTTP_<status>` | API 没有返回错误代码。 |

## 安全、重试与速率限制

每个请求都有一个安全类别，与 `dna` CLI 使用的相同：

| `OperationSafety` | 含义 | 示例 |
|---|---|---|
| `Read` | 没有副作用，即使 API 使用 POST 或 PUT | 搜索、详情、列表、续费和转移检查、CSR 生成、证书状态 |
| `Write` | 免费且可撤销的更改 | DNS 记录、锁定、隐私保护、名称服务器、联系人 |
| `Billable` | 扣除你的余额 | 注册、续费、转移、恢复、购买和续费 SSL、订购主机 |
| `Destructive` | 删除数据或难以撤销 | 删除 DNS 记录、移除 DNSSEC、吊销证书 |

- **只读模式**（`read_only(true)` 或 `DNA_READ_ONLY=true`）会在发送之前阻止一切不属于 `Read` 的请求。
  浏览探索、生成报表和搭建仪表板时可以使用。
- **重试。** 429 响应总是会重试，并遵循 `Retry-After`。即使对需付费的请求，这样做也是安全的，因为
  网关会在执行任何操作之前拒绝它们。网络错误、超时和 502/503/504 响应只会对读取请求，以及 CLI 标记为幂等的
  写入请求（锁定、解锁、隐私保护、名称服务器、DNS 更新和应用）重试。注册或续费绝不会被发送两次。
- **请求间隔控制。** 网关允许每个密钥每秒 5 个请求，域名搜索每秒 2 个。客户端会控制请求间隔，使其保持在
  这些限制之内，并让 DNS 区域调用之间保持 1 秒间隔。域名信息和 TLD 列表不受速率限制。
- **软配额。** 搜索和信息查询消耗 1 点，其他一切消耗 10 点。超过 200 点后，网关会让每个
  请求最多延迟 5 秒；点数每分钟衰减 20%。`send_with_response` 会在 `quota` 中显示当前状态。

## 异步代码与阻塞代码

SDK 是异步的，需要一个 Tokio 运行时（`#[tokio::main]` 启动的运行时，或你的服务器所用的运行时）。这些 future
都是 `Send`，因此可以在 `tokio::spawn`、axum 和 actix 的处理函数中使用。列表可以逐页读取（`list`），也可以作为
元素的 `Stream` 读取（`list_all`），后者只在需要时才获取下一页：

```rust
use futures_util::TryStreamExt;

let expiring: Vec<Domain> = client
    .domains()
    .list_all(DomainListRequest::new())
    .try_filter(|d| std::future::ready(d.remaining_days < 30))
    .try_collect()
    .await?;
```

SDK 没有阻塞式 API：那会使接口规模翻倍，而收益很小。在同步代码中，请在一个运行时上执行这些调用：

```rust
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
let account = runtime.block_on(client.account().get())?;
```

## 测试你的代码

SDK 只需要一个 HTTP 端点。要在不调用 API 的情况下测试使用该 SDK 的代码，请将 `base_url` 指向一个本地
服务器（例如用 `wiremock`、`httpmock` 或 `axum` 构建的服务器），并关闭请求间隔控制：

```rust
let client = Client::builder()
    .credentials("test", "test")
    .base_url(format!("http://{}", server_address))
    .pace_requests(false)
    .build()?;
```

模型实现了 `Default` 和 `Deserialize`，因此测试数据可以来自 JSON；可以用 `Error::new(kind, code, message)`
为你的模拟对象构建错误。要针对真实 API 测试，请使用 OTE 及其专用的密钥。

## 项目结构

```
src/
├── lib.rs           crate 文档与重新导出
├── client.rs        Client, ClientBuilder, Environment：发送、重试、请求间隔控制、错误解析、list_all
├── request.rs       Request, PagedRequest, Endpoint, OperationSafety, Query, JsonBody, Validator, Page, Empty
├── error.rs         Error, ErrorKind, codes
├── de.rs            宽容的解码，以及声明模型和枚举的宏（内部）
├── account.rs       AccountClient     余额、资料和交易记录
├── contacts.rs      ContactsClient    联系人句柄
├── domains/         DomainsClient     域名，以及 forwarding()、child_name_servers() 和 documents()
├── dns.rs           DnsClient         区域记录
├── dnssec.rs        DnssecClient      DS 记录
├── ssl.rs           SslClient         证书
├── hosting.rs       HostingClient     主机套餐与服务
├── orders.rs        OrdersClient      订单
└── products.rs      ProductsClient    TLD、SSL 产品和价格
tests/               伪造 API 测试、规范覆盖测试以及 testdata/openapi.json
examples/            可运行的程序
```

每个领域模块的结构相同：其响应所用的模型和枚举、每个操作对应一个请求类型（`XxxRequest`），以及一个 `XxxClient`。

## 开发

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                    # 单元测试、伪造 API 测试、规范测试和文档测试；都不会调用真实 API
cargo build --examples
```

如果本地没有工具链，可以使用 Docker：

```bash
docker run --rm -v "$PWD":/src -w /src rust:1.83 sh -c 'rustup component add rustfmt clippy; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
```

测试从不调用真实 API：它们使用一个运行在 127.0.0.1 上的小型 HTTP 服务器（`tests/common`）。CI 会在
最低支持的 Rust 版本（1.83）和最新稳定版上运行这些测试。

`tests/testdata/openapi.json` 是 `dna` CLI 内嵌的 API 文档。`tests/spec.rs` 会检查其中的每个操作都有且只有一个
请求类型，并且方法、路径、安全类别和幂等性与 CLI 一致；检查每个请求只发送文档中描述的查询参数和请求体属性
（包括嵌套对象，且必填项都存在）；并检查响应类型使用文档中的名称。当 API 新增操作时，请更新该文档；
在对应的请求类型创建出来之前，测试都会失败。

## 从 1.x 升级

2.0 是全新的 SDK。1.x 是 PHP 库的移植版本：一个阻塞式的 `DomainNameApi` 客户端，返回无类型的
`serde_json::Value` 信封。2.0 是异步的，提供带类型的请求和模型、`Result<T, Error>` 以及 builder。

| 1.x | 2.0 |
|---|---|
| `DomainNameApi::new(id, key, true)` | `Client::builder().credentials(id, key).environment(Environment::Ote).build()?` |
| `dna.get_reseller_details()`, `dna.get_current_balance(..)` | `client.account().get()` / `profile()` |
| `dna.check_availability(..)` | `client.domains().search(..)` / `bulk_search(..)` |
| `dna.get_list(..)` | `client.domains().list(..)` / `list_all(..)` |
| `dna.get_details(domain)` | `client.domains().info(domain)` |
| `dna.register_with_contact_info(..)` | `client.domains().register_with_contacts(..)` |
| `dna.renew(domain, period)` | `client.domains().renew(domain, years)` |
| `dna.transfer(domain, epp_code, period)` | `client.domains().transfer(DomainTransferRequest::new(domain, auth_code).years(years))` |
| `dna.check_transfer(domain, auth_code)` | `client.domains().transfer_check(domain, auth_code)` |
| `dna.modify_name_server(..)` | `client.domains().set_name_servers(..)` |
| `dna.enable_theft_protection_lock(domain)` | `client.domains().lock(domain)` |
| `dna.modify_privacy_protection_status(..)` | `client.domains().set_privacy(..)` |
| `dna.save_contacts(..)` | `client.domains().update_contacts(..)` |
| `dna.add_child_name_server(..)` | `client.domains().child_name_servers().add(..)` |
| `dna.get_resource_records(domain)` | `client.dns().list(domain)` |
| `dna.add_resource_record(..)` | `client.dns().add(..)` |
| `dna.set_resource_record_set(..)` | `client.dns().update(..)` |
| `dna.apply_resource_records(domain)` | `client.dns().apply(domain)` |
| `dna.set_forwarding(..)` | `client.domains().forwarding().set(..)` |
| `dna.get_tld_list(..)` | `client.products().all_tlds(..)` |
| `result["result"] == "ERROR"` | 带有 `e.kind()` 和 `e.code()` 的 `Err(e)` |

1.x 中的转移批准/取消/拒绝调用和 `sync_from_registry` 不在 REST API 的文档中，在 2.0 中没有对应的方法。
MSRV 仍为 1.83；`ureq` 被 `reqwest` 取代，因此 2.0 需要 Tokio 运行时。1.x 会向 Sentry 发送错误报告；
2.0 除了 API 之外，不会向任何地方发送任何内容。

## 链接

- [`dna` CLI 和 MCP 服务器](https://github.com/domainreseller/cli)：在终端和 AI 智能体中使用同一套 API。
- [API 文档](https://www.domainnameapi.com/docs/)
- [更新日志](CHANGELOG.md)

## 许可证

Apache License 2.0。参见 [LICENSE](LICENSE)。

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

# rust-dna — 适用于 Rust 的 DomainNameAPI 客户端

DomainNameAPI 域名注册商 REST API 的 Rust 客户端。它是参考 PHP 库 [php-dna](https://github.com/domainreseller/php-dna) 的忠实移植：相同的方法、相同的网络请求和相同的返回值。每个移植版本都通过 php-dna 的 134 个一致性测试用例（conformance fixtures）进行验证。

## 📦 下载 — 请务必使用 Releases！

⬇️ **在此获取最新的已测试版本：https://github.com/domainreseller/rust-dna/releases/latest**

> ⚠️ **不要**使用绿色的 **Code → Download ZIP** 按钮 — 它下载的是原始开发分支。Release 包带有版本号、经过测试，可直接用于生产环境。

## 安装和集成指南

### 最低要求

- Rust 1.83 或更高版本（2021 edition）
- 无需异步运行时：API 是阻塞式的
- 运行时依赖：`ureq`（HTTP，rustls TLS）、`rustls` 和 `serde_json`。不需要 OpenSSL。

### 安装

**A) Git 依赖（固定到某个 release 标签）**

```toml
[dependencies]
domainnameapi = { git = "https://github.com/domainreseller/rust-dna", tag = "v1.0.1" }
```

**B) 下载的 release 包**

将 release 压缩包解压到项目旁边，并让 Cargo 指向它：

```toml
[dependencies]
domainnameapi = { path = "../rust-dna" }
```

该 crate 重新导出了 `serde_json::Value` 和 `json!` 宏，因此无需自行添加 `serde_json`（如有需要也可以添加）。

## 🔑 API 凭据 — Reseller ID 与 API Key

本库与 DomainNameAPI REST API 通信，恰好需要两个凭据，并通过构造函数传入：

| 参数 | 值 |
|---|---|
| 第一个 | Reseller ID — 形如 `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` 的 UUID |
| 第二个 | API Key |

> 💡 您可以在 DomainNameAPI 面板的 **API 设置** 中找到您的 **Reseller ID** 和 **API Key**。
> ⚠️ 这些是 **API 凭据** — 您的面板登录信息在此处**无效**。

```rust
use domainnameapi::DomainNameApi;

let reseller_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
let api_key = "your-api-key";

// Live environment
let mut dna = DomainNameApi::new(reseller_id, api_key, false);

// Test (OTE) environment
let mut dna = DomainNameApi::new(reseller_id, api_key, true);
```

两个凭据的首尾空白都会被去除。由于面板常将复选框保存为文本，第三个参数也接受字符串：`"true"`、`"1"`、`"on"` 和 `"yes"`（不区分大小写）选择测试环境，其他任何值（包括 `"false"` 和 `"off"`）选择正式环境。

## 快速开始

```rust
use domainnameapi::DomainNameApi;

fn main() {
    let mut dna = DomainNameApi::new("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx", "your-api-key", false);

    let details = dna.get_details("example.com");
    if details["result"] == "OK" {
        println!("Expires: {}", details["data"]["Dates"]["Expiration"]);
    } else {
        eprintln!("{}: {}", details["error"]["Code"], details["error"]["Message"]);
    }
}
```

[examples/](examples) 中的每个示例都可以通过 `cargo run --example <名称>` 运行，例如 `cargo run --example get_details`。

## API 的结构

- **只有一个客户端类型**：`DomainNameApi`。方法接收 `&mut self`（客户端会记住最近一次请求，用于诊断）。
- **返回值为 `serde_json::Value`**，其结构与 php-dna 返回的完全一致 — 而不是强类型模型。键名、字符串标志（`"true"` / `"false"`）、格式化的金额字符串（`"11.9900"`）以及外层结构（envelope）完全相同：
  - 成功：`{"result": "OK", "data": {...}}`
  - 失败：`{"result": "ERROR", "error": {"Code": "...", "Message": "...", "Details": "..."}}`
  - 少数方法使用沿袭自 php-dna 的其他外层结构：`get_current_balance` 返回 `OperationResult` 对象，`check_availability` 成功时返回普通列表，`get_reseller_details` 在顶层返回其字段。
- **错误以返回值形式给出，从不抛出。** 网关错误、HTTP 错误和网络故障都会以错误结构返回；不会引发 panic。当网关以 HTTP 200 返回 `success: false` 时，注册、转移和续费同样会返回错误。
- **结构化输入**（联系人、域名服务器列表、筛选条件、TLD 属性、记录内容）是用 `json!` 构建的 `&Value`。联系人映射使用 php-dna 的键（`FirstName`、`EMail`、`AddressLine1`、`PhoneCountryCode`……）；平铺和嵌套（`Address.Line1`、`Phone.Phone.Number`）两种联系人格式均被接受。域名服务器列表可以是列表或映射（使用其中的值）。
- **Rust 没有默认参数。** 每个方法都按相同顺序接收 php-dna 的完整参数列表。php-dna 的默认值以常量形式导出：`DEFAULT_TTL`（3600）、`DEFAULT_REASON`（"Owner request"）、`DEFAULT_FORWARD_TYPE`（"Standard"）、`DEFAULT_CURRENCY`（"USD"）、`DEFAULT_TLD_COUNT`（20）、`DEFAULT_COMMAND`（"create"）。带有四个可选参数的 `register_with_contact_info` 还提供 `register_with_contact_info_opts`，它接收一个 `RegisterOptions` 结构体，其 `Default` 即为 php-dna 的默认值。

## 使用方法

所有代码片段均假定已有 `use domainnameapi::{json, DomainNameApi};` 和客户端 `let mut dna = DomainNameApi::new(...)`。

#### 域名注册

注意：.tr 域名需要额外参数。额外属性参数用于需要额外信息的域名，例如 .tr。

```rust
let contact = json!({
    "FirstName": "John",
    "LastName": "Doe",
    "Company": "Example Corp",
    "EMail": "john.doe@example.com",
    "AddressLine1": "123 Lorem Street",
    "AddressLine2": "Suite 456",
    "AddressLine3": "",
    "City": "Springfield",
    "Country": "US",
    "Fax": "1234567890",
    "FaxCountryCode": "1",
    "Phone": "9876543210",
    "PhoneCountryCode": 1,
    "Type": "Contact",
    "ZipCode": "12345",
    "State": "IL"
});
let contacts = json!({
    "Administrative": contact,
    "Billing": contact,
    "Technical": contact,
    "Registrant": contact
});

let result = dna.register_with_contact_info(
    "example.com",
    1,
    &contacts,
    &json!(["ns1.example.com", "ns2.example.com"]),
    true,  // registrar (theft protection) lock
    false, // privacy protection
    &json!({}),
);

// Same, with php-dna's defaults for the optional arguments
// (default nameservers, lock on, privacy off, no attributes):
use domainnameapi::RegisterOptions;
let result = dna.register_with_contact_info_opts("example.com", 1, &contacts, &RegisterOptions::default());
```

对于 `.tr` 域名，请传入 TRABIS 属性。本库只发送网关接受的属性（`TRABISDOMAINCATEGORY`、`TRABISCITIZENID`、`TRABISTAXOFFICE`、`TRABISTAXNUMBER`），将 `1`/`0` 映射为 `Owner`/`Corporate`，并修正旧的拼写 `TRABISCITIZIENID`：

```rust
let result = dna.register_with_contact_info_opts(
    "example.com.tr",
    1,
    &contacts,
    &RegisterOptions {
        name_servers: json!(["ns1.example.com", "ns2.example.com"]),
        additional_attributes: json!({
            "TRABISDOMAINCATEGORY": 1,
            "TRABISCITIZIENID": "12345678901",
            "TRABISNAMESURNAME": "John Doe",
            "TRABISCOUNTRYID": "840",
            "TRABISCITYID": "17"
        }),
        ..RegisterOptions::default()
    },
);
```

<details>
<summary>域名注册的示例输出</summary>

```json
{
  "data": {
    "ID": 123456,
    "Status": "Active",
    "DomainName": "example.com",
    "AuthCode": "Xy9#mK2$pL5@vN8",
    "LockStatus": "true",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "false",
    "Contacts": {
      "Billing": {"ID": 987654},
      "Technical": {"ID": 987654},
      "Administrative": {"ID": 987654},
      "Registrant": {"ID": 987654}
    },
    "Dates": {
      "Start": "2024-03-15T10:00:00",
      "Expiration": "2025-03-15T10:00:00",
      "RemainingDays": 365
    },
    "NameServers": ["ns1.example.com", "ns2.example.com"],
    "Additional": [],
    "ChildNameServers": []
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 域名续费

```rust
let result = dna.renew("example.com", 1);
```

<details>
<summary>域名续费的示例输出</summary>

```json
{
  "result": "OK",
  "data": {
    "ExpirationDate": "2025-03-15T10:00:00"
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 域名转移

转移从不发送联系人信息：注册商会从原注册商接管联系人，因此每个转移请求都携带 `contacts: []`。

```rust
let result = dna.transfer("example.com", "Xy9#mK2$", 3);
```

<details>
<summary>域名转移的示例输出</summary>

```json
{
  "data": {
    "ID": 0,
    "Status": "Active",
    "DomainName": "",
    "AuthCode": "",
    "LockStatus": "false",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "false",
    "Contacts": {
      "Billing": {"ID": 0},
      "Technical": {"ID": 0},
      "Administrative": {"ID": 0},
      "Registrant": {"ID": 0}
    },
    "Dates": {"Start": "", "Expiration": "2027-11-09T12:00:00", "RemainingDays": 365},
    "NameServers": [],
    "Additional": [],
    "ChildNameServers": []
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 转移管理

```rust
// Can the domain be transferred with this auth code?
let check = dna.check_transfer("example.com", "Xy9#mK2$");

// Cancel an incoming transfer, approve or reject an outgoing one
let result = dna.cancel_transfer("example.com");
let result = dna.approve_transfer("example.com");
let result = dna.reject_transfer("example.com");
```

<details>
<summary>转移管理的示例输出</summary>

```json
{
  "result": "OK",
  "data": {
    "TransferAvailabilityStatus": true,
    "AuthCodeIsRequired": true,
    "AuthCodeIsValid": true,
    "UserTransferRequired": false,
    "TransferLock": false,
    "Message": "Domain can be transferred",
    "MessageKey": "TransferAvailable",
    "Additional": []
  }
}
```

当注册局报告域名不可转移时，`check_transfer` 返回 `"result": "ERROR"` 以及 `error`（`TRANSFER_NOT_AVAILABLE`）；两种情况下 `data` 都会填充。`cancel_transfer`、`approve_transfer` 和 `reject_transfer` 返回：

```json
{"result": "OK", "data": {"DomainName": "example.com", "Status": "Cancelled"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 域名列表

```rust
let result = dna.get_list(&json!({
    "OrderColumn": "Id",
    "OrderDirection": "ASC",
    "PageNumber": 0,
    "PageSize": 1000
}));

// No filters:
let result = dna.get_list(&json!({}));
```

`PageNumber` 从 0 开始。也可以传入网关自身的 `MaxResultCount`、`SkipCount` 和 `Sorting`，以及 `DomainName` 等筛选条件。

<details>
<summary>域名列表的示例输出</summary>

```json
{
  "data": {
    "Domains": [
      {
        "ID": 564346,
        "Status": "Active",
        "DomainName": "example.com",
        "AuthCode": "",
        "LockStatus": "true",
        "PrivacyProtectionStatus": "false",
        "IsChildNameServer": "false",
        "Contacts": {
          "Billing": {"ID": ""},
          "Technical": {"ID": ""},
          "Administrative": {"ID": ""},
          "Registrant": {"ID": ""}
        },
        "Dates": {
          "Start": "2024-03-15T10:00:00",
          "Expiration": "2025-03-15T10:00:00",
          "RemainingDays": 365
        },
        "NameServers": ["ns1.example.com", "ns2.example.com"],
        "Additional": [],
        "ChildNameServers": []
      }
    ]
  },
  "result": "OK",
  "TotalCount": 1
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### TLD 列表

```rust
let result = dna.get_tld_list(100);
```

<details>
<summary>TLD 列表的示例输出</summary>

```json
{
  "data": [
    {
      "id": 1,
      "status": "Active",
      "maxchar": 63,
      "maxperiod": 10,
      "minchar": 1,
      "minperiod": 1,
      "tld": "com",
      "pricing": {
        "registration": {"1": "10.8100"},
        "renew": {"1": "11.0100"},
        "transfer": {"1": "10.6100"},
        "restore": {"1": "99.9000"}
      },
      "currencies": {
        "registration": "USD",
        "renew": "USD",
        "transfer": "USD",
        "restore": "USD"
      }
    }
  ],
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 域名可用性查询

```rust
let result = dna.check_availability(["hello", "world123x0"], ["com", "net"], 1, "create");
```

<details>
<summary>域名可用性查询的示例输出</summary>

成功时结果是一个普通列表（没有 `result` 键）；失败时为错误结构。

```json
[
  {
    "TLD": "com",
    "DomainName": "hello",
    "Status": "notavailable",
    "Command": "create",
    "Period": 1,
    "IsFee": false,
    "Price": "10.8100",
    "Currency": "USD",
    "Reason": "Domain exists"
  },
  {
    "TLD": "com",
    "DomainName": "world123x0",
    "Status": "available",
    "Command": "create",
    "Period": 1,
    "IsFee": false,
    "Price": "10.8100",
    "Currency": "USD",
    "Reason": null
  }
]
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 域名详情

```rust
let result = dna.get_details("example.com");

// Re-read the domain from the registry (same result shape):
let result = dna.sync_from_registry("example.com");
```

<details>
<summary>域名详情的示例输出</summary>

```json
{
  "data": {
    "ID": 564346,
    "Status": "Active",
    "DomainName": "example.com",
    "AuthCode": "DHQ!K52",
    "LockStatus": "true",
    "PrivacyProtectionStatus": "false",
    "IsChildNameServer": "true",
    "Contacts": {
      "Billing": {"ID": "DNA-B-4"},
      "Technical": {"ID": "DNA-T-3"},
      "Administrative": {"ID": "DNA-A-2"},
      "Registrant": {"ID": "DNA-R-1"}
    },
    "Dates": {
      "Start": "2025-05-26T16:08:37",
      "Expiration": "2027-05-26T16:08:37",
      "RemainingDays": 449
    },
    "NameServers": ["ns1.example.com", "ns2.example.com"],
    "Additional": {"TRABISDOMAINCATEGORY": "1"},
    "ChildNameServers": [
      {"ns": "ns1.example.com", "ip": "1.2.3.4"}
    ]
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 修改域名服务器

```rust
let result = dna.modify_name_server("example.com", &json!(["ns1.example.com", "ns2.example.com"]));
```

<details>
<summary>修改域名服务器的示例输出</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServers": ["ns1.example.com", "ns2.example.com"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 启用域名锁

```rust
let result = dna.enable_theft_protection_lock("example.com");
```

<details>
<summary>启用域名锁的示例输出</summary>

```json
{"result": "OK", "data": {"LockStatus": true}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 禁用域名锁

```rust
let result = dna.disable_theft_protection_lock("example.com");
```

<details>
<summary>禁用域名锁的示例输出</summary>

```json
{"result": "OK", "data": {"LockStatus": false}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 添加子域名服务器

```rust
let result = dna.add_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>添加子域名服务器的示例输出</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServer": "ns1.example.com",
    "IPAdresses": ["1.2.3.4"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 删除子域名服务器

```rust
let result = dna.delete_child_name_server("example.com", "ns1.example.com");
```

<details>
<summary>删除子域名服务器的示例输出</summary>

```json
{"result": "OK", "data": {"NameServer": "ns1.example.com"}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 更新子域名服务器

```rust
let result = dna.modify_child_name_server("example.com", "ns1.example.com", "1.2.3.4");
```

<details>
<summary>更新子域名服务器的示例输出</summary>

```json
{
  "result": "OK",
  "data": {
    "NameServer": "ns1.example.com",
    "IPAdresses": ["1.2.3.4"]
  }
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 修改域名隐私保护

`status` 可以是 `bool`，也可以是面板保存的值：`"1"`、`"on"`、`"true"`、`"yes"` 和 `1` 表示开启，其他任何值表示关闭。原因（reason）参数仅为兼容而保留，不会被发送，因为网关没有该字段。

```rust
let result = dna.modify_privacy_protection_status("example.com", true, "Owner request");
```

<details>
<summary>修改域名隐私保护的示例输出</summary>

```json
{"data": {"PrivacyProtectionStatus": true}, "result": "OK"}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 获取域名联系人

```rust
let result = dna.get_contacts("example.com");
```

<details>
<summary>获取域名联系人的示例输出</summary>

```json
{
  "data": {
    "contacts": {
      "Administrative": {
        "ID": "DNA-A-2",
        "Status": "Active",
        "AuthCode": "",
        "FirstName": "John",
        "LastName": "Doe",
        "Company": "Example Corp",
        "EMail": "john@example.com",
        "Type": "Administrative",
        "Address": {
          "Line1": "123 Main Street",
          "Line2": "",
          "Line3": "",
          "State": "IL",
          "City": "Springfield",
          "Country": "US",
          "ZipCode": "62701"
        },
        "Phone": {
          "Phone": {"Number": "5551234567", "CountryCode": "1"},
          "Fax": {"Number": "", "CountryCode": ""}
        },
        "Additional": []
      },
      "Billing": {"...": "same shape"},
      "Registrant": {"...": "same shape"},
      "Technical": {"...": "same shape"}
    }
  },
  "result": "OK"
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 保存域名联系人

```rust
let contact = json!({
    "FirstName": "John",
    "LastName": "Doe",
    "Company": "Example Corp",
    "EMail": "john@example.com",
    "AddressLine1": "123 Main Street",
    "AddressLine2": "",
    "AddressLine3": "",
    "City": "Springfield",
    "Country": "US",
    "Fax": "5559876543",
    "FaxCountryCode": "1",
    "Phone": "5551234567",
    "PhoneCountryCode": 1,
    "Type": "Contact",
    "ZipCode": "62701",
    "State": "IL"
});

let result = dna.save_contacts("example.com", &json!({
    "Administrative": contact,
    "Billing": contact,
    "Technical": contact,
    "Registrant": contact
}));
```

<details>
<summary>保存域名联系人的示例输出</summary>

```json
{"result": "OK", "data": {"contacts": []}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### DNS 区域记录

网关以**记录集**的形式存储记录（一个名称 + 类型，包含一个或多个值）。写入记录集会替换其全部值，删除其中任意一个值会删除整个记录集。`*_value` 方法会替你完成"读取-合并-写入"，使面板可以逐条处理记录。名称可以是相对名称（`www`）、表示顶点（apex）的 `@` / 空字符串，或完全限定名称；它们会被归约为网关期望的相对标签。支持的类型：A、AAAA、ALIAS、CAA、CNAME、DS、MX、NAPTR、PTR、SRV、TLSA、TXT。当 `apply` 为 `true`（php-dna 的默认值）时更改会立即生效；否则请调用 `apply_resource_records`。由于网关会限制突发请求，区域调用之间至少间隔一秒。如果自动应用失败，该方法会返回应用错误（更改仍处于暂存状态）。结果中报告的是转换为大写的类型和实际发送的 TTL（小于 1 时变为 3600，大于 86400 时变为 86400）。

```rust
// List every record (one row per value; names come back as FQDNs)
let records = dna.get_resource_records("example.com");

// Make a set hold exactly these values (an empty list removes the set)
dna.set_resource_record_set("example.com", "@", "MX", ["10 mx1.example.com", "20 mx2.example.com"], 3600, true);

// Add, remove or replace one value, keeping the others (ttl None keeps the set's TTL)
dna.add_resource_record_value("example.com", "@", "MX", "30 mx3.example.com", None, true);
dna.remove_resource_record_value("example.com", "@", "MX", "30 mx3.example.com", true);
dna.replace_resource_record_value("example.com", "@", "A", "192.0.2.1", "192.0.2.2", None, true);

// Remove a whole set (pass a current value to save a lookup, or "")
dna.delete_resource_record_set("example.com", "www", "CNAME", true, "");

// Low-level calls, mirroring the gateway endpoints
dna.add_resource_record("example.com", "www", "A", &json!("192.0.2.5"), 3600, true);
dna.edit_resource_record("example.com", "www", "www", "A", &json!("192.0.2.6"), 600, true);
dna.delete_resource_record("example.com", "www", "192.0.2.5", "A", true);
dna.apply_resource_records("example.com");
```

<details>
<summary>DNS 区域记录的示例输出</summary>

`get_resource_records`：

```json
{
  "result": "OK",
  "data": {
    "records": [
      {"Name": "example.com.", "Type": "A", "TTL": 3600, "Content": "192.0.2.1", "Disabled": false},
      {"Name": "www.example.com.", "Type": "CNAME", "TTL": 300, "Content": "example.com.", "Disabled": false},
      {"Name": "example.com.", "Type": "MX", "TTL": 3600, "Content": "10 mx1.example.com.", "Disabled": false}
    ]
  }
}
```

`set_resource_record_set` / `add_resource_record_value`：

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "TTL": 3600, "Contents": ["10 mx1.example.com.", "20 mx2.example.com.", "30 mx3.example.com"]}}
```

已经存在的值（删除时为不存在的值）不会造成任何改变：

```json
{"result": "OK", "data": {"Name": "", "Type": "MX", "Contents": ["10 mx1.example.com."], "Changed": false}}
```

不支持的类型在发出任何请求之前即被拒绝：

```json
{"result": "ERROR", "error": {"Code": "ADD_ZONE", "Message": "Record type \"NS\" is not supported by the DNS zone API. Supported types: A, AAAA, ALIAS, CAA, CNAME, DS, MX, NAPTR, PTR, SRV, TLSA, TXT.", "Details": "..."}}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### URL 转发

```rust
let current = dna.get_forwarding("example.com");
let result = dna.set_forwarding("example.com", "https://www.example.org", "Standard"); // or "Frame" (masked)
let result = dna.delete_forwarding("example.com");
```

<details>
<summary>URL 转发的示例输出</summary>

```json
{"result": "OK", "data": {"Enabled": true, "RedirectAddress": "https://www.example.org/landing", "ForwardType": "Frame"}}
```

未配置转发时（网关返回 `Redirect information not found`），`get_forwarding` 返回 `{"result": "OK", "data": {"Enabled": false, "RedirectAddress": "", "ForwardType": ""}}`；其他任何失败（例如域名不存在）都会作为错误返回。`set_forwarding` 返回 `{"result": "OK", "data": {"RedirectAddress": "...", "ForwardType": "Standard"}}`。`delete_forwarding` 返回 `{"result": "OK", "data": {"DomainName": "example.com", "Removed": true}}`；若未设置转发则为 `"Removed": false`。

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 获取经销商详情

```rust
let result = dna.get_reseller_details();
let balance = dna.get_current_balance("USD"); // "USD" or "TRY"
```

<details>
<summary>获取经销商详情的示例输出</summary>

```json
{
  "result": "OK",
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "active": true,
  "name": "Example Reseller",
  "balance": "601.64",
  "currency": "USD",
  "symbol": "$",
  "balances": [
    {"balance": "601.64", "currency": "USD", "symbol": "$"},
    {"balance": "1250.50", "currency": "TL", "symbol": "TL"}
  ]
}
```

`get_current_balance`：

```json
{
  "ErrorCode": 0,
  "OperationMessage": "Command completed succesfully.",
  "OperationResult": "SUCCESS",
  "Balance": "601.64",
  "CurrencyId": 2,
  "CurrencyInfo": null,
  "CurrencyName": "USD",
  "CurrencySymbol": "$"
}
```

账户只报告 USD 和 TRY 余额；其他任何货币（EUR、GBP……）都会以 `ErrorCode` `"BALANCE"` 失败。失败时 `OperationResult` 为 `"FAILED"`，`ErrorCode` 包含错误码（例如 `"API_429"`），`Balance` 为 `"0.00"`。

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

#### 实用工具（无需网络）

```rust
let is_tr = dna.is_tr_tld("example.com.tr"); // true

// Complete and normalise a contact the way registration does
let contact = dna.validate_contact(&json!({
    "FirstName": " Jo ",
    "Phone": "+1 (202) 555-0100",
    "Country": "Turkey"
}));
```

<details>
<summary>validate_contact 的示例输出</summary>

```json
{
  "FirstName": "Jo",
  "Phone": "2025550100",
  "Country": "TR",
  "LastName": "Isimyok",
  "AddressLine1": "Addres yok",
  "City": "ISTANBUL",
  "ZipCode": "34000",
  "PhoneCountryCode": "1",
  "State": "ISTANBUL",
  "Fax": "",
  "FaxCountryCode": ""
}
```

</details>

<hr style="border: 4px solid #000; border-style: dashed;">

## 错误处理

每个网络方法都会返回一个值；请检查 `result`：

```rust
let out = dna.get_details("missing.com");
if out["result"] == "ERROR" {
    println!("{}", out["error"]["Code"]);        // "API_404"
    println!("{}", out["error"]["GatewayCode"]); // "Dna.DomainService:Domain:10007"
    println!("{}", out["error"]["Message"]);     // "The domain name you requested could not be found. ..."
    println!("{}", out["error"]["Details"]);
}
```

- HTTP 错误使用 `API_<状态码>` 代码（`API_400`、`API_404`、`API_429`、`API_500`……）。空的错误响应体会转换为可读消息，例如 `HTTP 429 - Too Many Requests`。如果网关发送了自己的错误码，外层结构还会以 `GatewayCode` 携带它（例如 `"Dna.DomainService:Domain:10007"`）。
- 301/302 响应表示凭据错误：`Invalid API credentials`。
- 网关的校验错误会被合并到 `Details` 中，并指明字段，例如 `Contacts Phone - The field Phone must be a string with a maximum length of 16.`
- 网络故障属于错误，绝不会表现为空的成功。超时消息为 `Could not reach the DomainNameAPI service: the connection timed out. (...)`；其他故障消息为 `Curl error during request: ...`。其代码为 `CURL_<curl 错误号>`（超时为 `CURL_28`，DNS 为 `CURL_6`，连接被拒绝为 `CURL_7`）；`API_<n>` 仅用于 HTTP 状态码。
- 本地校验（不支持的记录类型、缺少转发地址、没有要查询的域名）会在发出任何请求之前以描述性代码失败（`ADD_ZONE`、`SET_FORWARD`、`AVAILABILITY`……）。

### 诊断

```rust
dna.get_request_data();        // {"url", "method", "payload"} of the last request
dna.get_response_data();       // decoded last response, or {"raw_response": "<body>"}
dna.get_response_headers();    // {"url", "content_type", "http_code", "total_time", "headers"}
dna.get_parsed_response_data();// parsed body on success, {"Code", "Message", "Details"} on failure
dna.get_last_function();       // e.g. "request:GET domains/info"
dna.get_service_url();
dna.set_service_url("https://ote.domainresellerapi.com/api/v1");
```

### 自定义传输层

HTTP 层是 `Transport` trait。默认实现为 `UreqTransport`（基于 rustls 的阻塞式 `ureq`，总超时 30 秒、连接超时 10 秒，不跟随重定向）。可将其替换以支持代理、自定义客户端或测试：

```rust
use domainnameapi::{HttpRequest, HttpResponse, Transport, TransportError};

struct MyTransport;
impl Transport for MyTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        // perform req.method on req.url with req.headers and req.body
        Ok(HttpResponse { status: 200, body: "{}".into(), headers: vec![] })
    }
}

dna.set_transport(Box::new(MyTransport));
```

## 测试模式

将 `true`（或 `"true"`、`"1"`、`"on"`、`"yes"`）作为构造函数的第三个参数传入，即可使用位于 `https://ote.domainresellerapi.com/api/v1` 的 OTE 测试环境。正式调用发送至 `https://api.domainresellerapi.com/api/v1`。

## 测试

测试套件既不需要网络，也不需要凭据：

```bash
cargo test
```

其中包括针对 HTTP 层、错误映射、遥测脱敏和实用工具的单元测试，以及一个一致性测试：它重放 `tests/conformance/` 中全部 134 个 php-dna 测试用例，并检查每个发出的请求和每个返回值都与 php-dna 一致。

## 错误报告（遥测）

当 API 调用失败时，本库可以向 DomainNameAPI 自有的错误跟踪系统（Sentry）发送错误报告。成功调用中的一小部分随机样本（4%，外加所有耗时超过 1 秒的调用）会作为性能记录发送。这有助于 DomainNameAPI 发现并修复问题。错误报告**默认开启**。

**发送的内容：**

- 错误代码和消息
- 被调用的端点路径和 HTTP 方法
- 调用耗时
- 应用名称（`DomainNameAPI-Rust`）和库版本
- 经销商 ID 和域名
- 服务器的公网 IP 地址（在后台通过 `api.ipify.org` 查询）
- 操作系统用户名
- API 请求和响应，包括域名的联系人信息；只有凭据会被屏蔽 — EPP/授权码、API 密钥、密码、密钥（secret）和令牌会被替换为 `***`

**绝不发送的内容：** 可读形式的 EPP/授权码、API 密钥、密码或令牌。

报告由后台线程以较短的超时时间发送；它们绝不会阻塞您的调用，也不会导致其失败。诸如"域名不存在"或"余额不足"之类的预期错误不会被报告。

**原因及保存时长：** 这些数据仅用于使用量统计和问题的早期诊断，在 DomainNameAPI 自有的 Sentry 服务器上保存 30 天，之后自动删除。

**如何关闭或减少：**

```rust
let mut dna = DomainNameApi::new(reseller_id, api_key, false);
dna.set_error_reporting_enabled(false); // nothing is sent

// or keep it on but send only the error code and message, endpoint,
// timing, versions and reseller ID (no request/response data, domain, IP or user name)
dna.set_telemetry_minimal(true);
```

> 🔒 **安全：** TLS 证书验证处于开启状态。在 CA 证书包过旧的主机上，可以通过 `dna.set_ssl_verification(false);` 关闭 — 不建议这样做；更新 CA 证书包才是更好的解决办法。

## 返回和错误代码及说明

| 代码 | 描述 | 详细 |
|------|------|------|
| 1000 | 命令成功完成 | 操作成功。 |
| 1001 | 命令成功完成；操作待处理 | 操作成功。但是，操作当前已排队等待完成。 |
| 2003 | 缺少必需参数 | 参数缺失错误。例如：联系信息中没有电话输入。 |
| 2105 | 对象不符合续费条件 | 域名状态不适合续费，已锁定更新操作。状态不应为"clientupdateprohibited"。可能由于其他状态条件。 |
| 2200 | 身份验证错误 | 授权错误，安全代码不正确或域名在另一个注册商处注册。 |
| 2302 | 对象已存在 | 域名或名称服务器信息已存在于数据库中。无法注册。 |
| 2303 | 对象不存在 | 域名或名称服务器信息在数据库中不存在。需要新注册。 |
| 2304 | 对象状态禁止操作 | 域名状态不适合更新，已锁定更新操作。状态不应为"clientupdateprohibited"。可能由于其他状态条件。 |

| 代码 | 详细 |
|------|------|
| 101 | 系统错误详细信息 ({0})！ |
| 102 | 多个错误详细信息 ({0})！ |
| 103 | 未知错误详细信息 ({0})！ |
| 200 | 未找到 API 命令 ({0})！ |
| 210 | 未找到 API 服务 ({0})！ |
| 211 | 未设置 API 服务提供商！ |
| 300 | 未找到经销商！ |
| 301 | 您当前的 IP 地址未被授权访问。请确保您从授权的 IP 地址连接并重试 |
| 310 | 不支持 TLD！ |
| 320 | 未找到 API！ |
| 321 | 不支持货币！ |
| 330 | 未设置必需参数 ({0})。 |
| – | 确保您完整发送所有联系对象 |
| 340 | 未找到价格定义 ({0}[{1}]-{2}{3})。 |
| 350 | 经销商余额不足。(经销商 Id : {0} - 当前余额 : {1} {2})。 |
| 350 | 会计货币不匹配或余额不足。 |
| 360 | 字段 ({0}) 的无效 API 请求。 |
| 360 | API 配额已超出！ |
| 361 | 限流错误！ |
| 362 | 高级域名目前无法注册。 |
| 363 | 由于域名处于自动续费期间，操作已取消 |
| 364 | 由于注册处问题，此域名目前无法进行交易 |

| 代码 | 详细 |
|------|------|
| 400 | 无效联系人 ({0})。 |
| 401 | 联系信息无法同步。 |
| 402 | 无法从注册处访问联系信息。 |
| 403 | 系统缺少信息。输入您的默认信息或联系支持团队。 |
| 404 | 域名联系人无法更新。来自注册处的权限。 |
| 410 | 未找到联系人。 |
| 410 | 未找到联系人 ({0})。 |
| 420 | 联系人 {0} 的无效 Api 命令。 |
| 430 | 未找到联系人 api。 |
| 440 | 联系人未同步。 |
| 450 | 域名联系人太多..！ |
| 451 | 无法继续进行联系人更新。 |

| 代码 | 详细 |
|------|------|
| 500 | 无效域名 id。 |
| 500 | 无效域名 id ({0})。 |
| 501 | 域名无法同步({0}) |
| 502 | 内部转移失败 |
| 503 | 域名注册不可用。 |
| 504 | 域名信息不匹配 .. 之前 :{0} |
| 505 | 输入 IP 地址。 |
| 506 | 域名转移已开始但无法读取联系信息 .. |
| 507 | 授权错误。 |
| 510 | 未找到域名。 |
| 511 | 无法找到过期域名。 |
| 512 | 域名不可续费。 |
| 513 | 域名不处于可更新状态。必须处于活动状态才能更新 |
| 514 | 预期赎回期。 |
| 520 | 域名 "{0}" 的无效 Api 命令。 |
| 530 | 无效域名期限。期限必须为 {0} 到 {1} 年。请求的期限为 {2} 年。 |
| 540 | 域名无法延长到当前日期 {0} 年以后。 |
| 550 | 无效域名。域名必须为 {0} 到 {1} 个字符长度。请求的域名为 {2} 个字符。 |
| 560 | 无效名称服务器数量。域名必须有 {0} 到 {1} 个名称服务器。请求的名称服务器数量为 {2}。 |
| 561 | 在传入请求中未找到名称服务器信息 |
| 570 | tld "{0}" 不支持 Idn。 |
| 571 | 期限无效。 |
| 572 | 命令无效。 |
| 573 | 未找到域名。 |
| 574 | 未找到 TLD 名称。 |
| 575 | 域名不处于可更新状态。必须处于活动状态才能更新名称服务器 |
| 576 | 域名可在到期日期前最后 1 个月内续费 |
| 580 | tld "{0}" 不支持转移。 |
| 581 | 未找到子名称服务器 |
| 582 | 转移由其他经销商发起。 |
| 583 | 转移未初始化。请联系支持团队。 |
| 584 | 对象状态禁止操作 |
| 590 | 此转移需要授权代码。 |
| 591 | 授权代码无效。 |
| 592 | 域名上存在转移锁定。 |
| 593 | 域名无法转移，状态信息和转移锁定必须符合所需标准。(状态信息: #ok) |
| 594 | 域名服务器地址无法解析 ({0}) |
| 595 | 无法读取联系信息 (whois.registrar.tld) 请确保隐私保护状态开放 |
| 596 | 联系信息无法验证 |
| 597 | 未找到 Tld |
| 598 | 发生未知错误 |
| 599 | 未找到域名转发 |

## 许可证

Apache License 2.0 — 参见 [LICENSE](LICENSE)。

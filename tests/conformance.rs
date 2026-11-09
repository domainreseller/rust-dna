//! Runs every php-dna conformance fixture (tests/conformance/*.json) against
//! this port: same requests on the wire, same return value.

mod common;

use std::path::PathBuf;

use common::{client, first_diff, loose_eq, split_url, Reply, API_KEY, RESELLER};
use domainnameapi::php;
use domainnameapi::{DomainNameApi, RegisterOptions};
use serde_json::{json, Map, Value};

struct Args<'a>(&'a [Value]);

impl Args<'_> {
    fn has(&self, i: usize) -> bool {
        i < self.0.len()
    }
    fn val(&self, i: usize) -> Value {
        self.0.get(i).cloned().unwrap_or(Value::Null)
    }
    fn s(&self, i: usize) -> String {
        php::strval(&self.val(i))
    }
    fn s_or(&self, i: usize, default: &str) -> String {
        if self.has(i) {
            self.s(i)
        } else {
            default.to_string()
        }
    }
    fn int(&self, i: usize) -> i64 {
        php::intval(&self.val(i))
    }
    fn int_or(&self, i: usize, default: i64) -> i64 {
        if self.has(i) {
            self.int(i)
        } else {
            default
        }
    }
    fn opt_int(&self, i: usize) -> Option<i64> {
        match self.0.get(i) {
            None | Some(Value::Null) => None,
            Some(v) => Some(php::intval(v)),
        }
    }
    fn b_or(&self, i: usize, default: bool) -> bool {
        if self.has(i) {
            php::truthy(&self.val(i))
        } else {
            default
        }
    }
    fn v_or(&self, i: usize, default: Value) -> Value {
        if self.has(i) {
            self.val(i)
        } else {
            default
        }
    }
    fn strings(&self, i: usize) -> Vec<String> {
        php::values(&self.val(i))
            .into_iter()
            .map(php::strval)
            .collect()
    }
}

/// Map a php-dna (camelCase) call onto this port's method.
fn dispatch(dna: &mut DomainNameApi, call: &str, args: &[Value]) -> Value {
    let a = Args(args);
    match call {
        "getResellerDetails" => dna.get_reseller_details(),
        "getCurrentBalance" => dna.get_current_balance(&a.s_or(0, "USD")),
        "checkAvailability" => {
            dna.check_availability(a.strings(0), a.strings(1), a.int(2), &a.s_or(3, "create"))
        }
        "getList" => dna.get_list(&a.v_or(0, json!({}))),
        "getTldList" => dna.get_tld_list(a.int_or(0, 20)),
        "getDetails" => dna.get_details(&a.s(0)),
        "syncFromRegistry" => dna.sync_from_registry(&a.s(0)),
        "getContacts" => dna.get_contacts(&a.s(0)),
        "saveContacts" => dna.save_contacts(&a.s(0), &a.val(1)),
        "modifyNameServer" => dna.modify_name_server(&a.s(0), &a.val(1)),
        "enableTheftProtectionLock" => dna.enable_theft_protection_lock(&a.s(0)),
        "disableTheftProtectionLock" => dna.disable_theft_protection_lock(&a.s(0)),
        "addChildNameServer" => dna.add_child_name_server(&a.s(0), &a.s(1), &a.s(2)),
        "deleteChildNameServer" => dna.delete_child_name_server(&a.s(0), &a.s(1)),
        "modifyChildNameServer" => dna.modify_child_name_server(&a.s(0), &a.s(1), &a.s(2)),
        "modifyPrivacyProtectionStatus" => dna.modify_privacy_protection_status(
            &a.s(0),
            a.val(1) == json!(true),
            &a.s_or(2, "Owner request"),
        ),
        "renew" => dna.renew(&a.s(0), a.int(1)),
        "registerWithContactInfo" => {
            if a.has(3) {
                dna.register_with_contact_info(
                    &a.s(0),
                    a.int(1),
                    &a.val(2),
                    &a.v_or(3, json!([])),
                    a.b_or(4, true),
                    a.b_or(5, false),
                    &a.v_or(6, json!([])),
                )
            } else {
                dna.register_with_contact_info_opts(
                    &a.s(0),
                    a.int(1),
                    &a.val(2),
                    &RegisterOptions::default(),
                )
            }
        }
        // The php-dna facade's transfer() takes three arguments; a fourth
        // (contacts) is dropped there, so it is dropped here too.
        "transfer" => dna.transfer(&a.s(0), &a.s(1), a.int(2)),
        "cancelTransfer" => dna.cancel_transfer(&a.s(0)),
        "approveTransfer" => dna.approve_transfer(&a.s(0)),
        "rejectTransfer" => dna.reject_transfer(&a.s(0)),
        "checkTransfer" => dna.check_transfer(&a.s(0), &a.s(1)),
        "getResourceRecords" => dna.get_resource_records(&a.s(0)),
        "addResourceRecord" => dna.add_resource_record(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            &a.val(3),
            a.int_or(4, 3600),
            a.b_or(5, true),
        ),
        "editResourceRecord" => dna.edit_resource_record(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            &a.s(3),
            &a.val(4),
            a.int_or(5, 3600),
            a.b_or(6, true),
        ),
        "deleteResourceRecord" => {
            dna.delete_resource_record(&a.s(0), &a.s(1), &a.s(2), &a.s(3), a.b_or(4, true))
        }
        "applyResourceRecords" => dna.apply_resource_records(&a.s(0)),
        "setResourceRecordSet" => dna.set_resource_record_set(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            a.strings(3),
            a.int_or(4, 3600),
            a.b_or(5, true),
        ),
        "deleteResourceRecordSet" => dna.delete_resource_record_set(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            a.b_or(3, true),
            &a.s_or(4, ""),
        ),
        "addResourceRecordValue" => dna.add_resource_record_value(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            &a.s(3),
            a.opt_int(4),
            a.b_or(5, true),
        ),
        "removeResourceRecordValue" => {
            dna.remove_resource_record_value(&a.s(0), &a.s(1), &a.s(2), &a.s(3), a.b_or(4, true))
        }
        "replaceResourceRecordValue" => dna.replace_resource_record_value(
            &a.s(0),
            &a.s(1),
            &a.s(2),
            &a.s(3),
            &a.s(4),
            a.opt_int(5),
            a.b_or(6, true),
        ),
        "getForwarding" => dna.get_forwarding(&a.s(0)),
        "setForwarding" => dna.set_forwarding(&a.s(0), &a.s(1), &a.s_or(2, "Standard")),
        "deleteForwarding" => dna.delete_forwarding(&a.s(0)),
        "isTrTLD" => json!(dna.is_tr_tld(&a.s(0))),
        "validateContact" => dna.validate_contact(&a.val(0)),
        other => panic!("fixture calls unknown method {other}"),
    }
}

fn value_at<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(v, |cur, key| php::get(cur, key).or_else(|| cur.get(key)))
}

fn set_at(v: &mut Value, path: &str, new: Value) {
    let mut cur = v;
    let keys: Vec<&str> = path.split('.').collect();
    for key in &keys[..keys.len() - 1] {
        cur = match cur {
            Value::Object(m) => m.get_mut(*key).expect("ignore path exists"),
            Value::Array(l) => l
                .get_mut(key.parse::<usize>().unwrap())
                .expect("ignore path exists"),
            _ => return,
        };
    }
    if let Value::Object(m) = cur {
        m.insert(keys[keys.len() - 1].to_string(), new);
    }
}

fn run_fixture(fixture: &Value) -> Result<(), String> {
    let call = fixture["call"].as_str().unwrap();
    let args = fixture["args"].as_array().cloned().unwrap_or_default();
    let replies: Vec<Reply> = fixture["responses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            Reply::Http(
                r["status"].as_u64().unwrap() as u16,
                r["body"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    let (mut dna, transport) = client(replies);

    let output = dispatch(&mut dna, call, &args);
    let mut output: Value = serde_json::from_str(&serde_json::to_string(&output).unwrap()).unwrap();

    if transport.exhausted() {
        return Err("ran out of queued responses (extra request)".into());
    }

    // Requests
    let expected_requests = fixture["requests"].as_array().unwrap();
    let actual = transport.requests();
    if actual.len() != expected_requests.len() {
        return Err(format!(
            "expected {} request(s), sent {}: {:?}",
            expected_requests.len(),
            actual.len(),
            actual
                .iter()
                .map(|r| format!("{} {}", r.method, r.url))
                .collect::<Vec<_>>()
        ));
    }
    for (i, (req, exp)) in actual.iter().zip(expected_requests).enumerate() {
        let (path, query) = split_url(&req.url);
        if req.method != exp["method"].as_str().unwrap() {
            return Err(format!(
                "request {i}: method {} != {}",
                req.method, exp["method"]
            ));
        }
        if path != exp["path"].as_str().unwrap() {
            return Err(format!("request {i}: path {path} != {}", exp["path"]));
        }
        if !loose_eq(&Value::Object(query.clone()), &exp["query"]) {
            return Err(format!(
                "request {i}: query {} != {}",
                Value::Object(query),
                exp["query"]
            ));
        }
        let body: Value = match &req.body {
            None => Value::Null,
            Some(b) => serde_json::from_str(b)
                .map_err(|e| format!("request {i}: body is not JSON: {e}"))?,
        };
        if let Some(d) = first_diff(&body, &exp["body"], "body") {
            return Err(format!("request {i}: {d}"));
        }
        let header = |name: &str| {
            req.headers
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        if header("X-API-KEY") != API_KEY || exp["headers"]["X-API-KEY"] != API_KEY {
            return Err(format!("request {i}: X-API-KEY header mismatch"));
        }
        if header("__reseller") != RESELLER || exp["headers"]["__reseller"] != RESELLER {
            return Err(format!("request {i}: __reseller header mismatch"));
        }
        if header("User-Agent") != domainnameapi::USER_AGENT {
            return Err(format!("request {i}: User-Agent header missing"));
        }
    }

    // Output
    let mut expected = fixture["output"].clone();
    for path in fixture["ignore"].as_array().cloned().unwrap_or_default() {
        let path = path.as_str().unwrap();
        match value_at(&output, path) {
            Some(Value::Number(n)) if n.as_i64().map(|x| x >= 0).unwrap_or(false) => {}
            other => {
                return Err(format!(
                    "{path}: expected a non-negative integer, got {other:?}"
                ))
            }
        }
        set_at(&mut output, path, json!(0));
        set_at(&mut expected, path, json!(0));
    }
    if let Some(d) = first_diff(&output, &expected, "output") {
        return Err(d);
    }
    Ok(())
}

#[test]
fn all_php_dna_fixtures_pass() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/conformance");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/conformance exists")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    files.sort();
    assert_eq!(files.len(), 126, "expected the 126 php-dna fixtures");

    let mut failures = Vec::new();
    let mut methods = Map::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let fixture: Value = serde_json::from_str(&text).unwrap();
        methods.insert(fixture["call"].as_str().unwrap().to_string(), Value::Null);
        if let Err(e) = run_fixture(&fixture) {
            failures.push(format!("{}: {e}", fixture["id"].as_str().unwrap()));
        }
    }
    println!(
        "{} of {} fixtures passed ({} methods covered)",
        files.len() - failures.len(),
        files.len(),
        methods.len()
    );
    assert!(
        failures.is_empty(),
        "{} fixture(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The runner must reject a fixture whose expectations do not hold.
#[test]
fn runner_detects_mismatches() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/conformance");
    let load = |name: &str| -> Value {
        serde_json::from_str(&std::fs::read_to_string(dir.join(name)).unwrap()).unwrap()
    };

    let mut f = load("getDetails.ok.json");
    assert!(run_fixture(&f).is_ok());
    f["output"]["data"]["Status"] = json!("Deleted");
    assert!(run_fixture(&f).is_err(), "changed output must fail");

    let mut f = load("saveContacts.messy.json");
    f["requests"][0]["body"]["contacts"][1]["phone"] = json!("0");
    assert!(run_fixture(&f).is_err(), "changed request body must fail");

    let mut f = load("getList.paging.json");
    f["requests"][0]["query"]["SkipCount"] = json!("50");
    assert!(run_fixture(&f).is_err(), "changed query must fail");

    let mut f = load("setResourceRecordSet.ok.json");
    f["responses"].as_array_mut().unwrap().pop();
    assert!(run_fixture(&f).is_err(), "missing response must fail");
}

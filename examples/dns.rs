//! Manages DNS records of a domain on the platform nameservers. The OTE test environment answers zone calls with
//! HTTP 500, so this needs production and a domain you do not mind changing.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=...
//! cargo run --example dns -- example.com
//! ```

use domainnameapi::dns::{DnsAddRequest, DnsDeleteRequest, DnsUpdateRequest, RecordType};
use domainnameapi::Client;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;
    let domain = std::env::args().nth(1).expect("usage: dns <domain>");
    let dns = client.dns();

    dns.add(DnsAddRequest::new(&domain, "www", RecordType::A, ["192.0.2.10"]).ttl(300)).await?;
    dns.add(DnsAddRequest::new(&domain, "@", RecordType::Mx, ["10 mx1.example.net", "20 mx2.example.net"])).await?;
    dns.add(DnsAddRequest::new(&domain, "@", "txt", ["v=spf1 include:_spf.example.net ~all"])).await?;
    dns.update(DnsUpdateRequest::new(&domain, "www", RecordType::A, ["192.0.2.20"]).ttl(600)).await?;

    for set in dns.list(&domain).await? {
        println!("{:<30} {:<6} {:>6} {:?}", set.name, set.record_type, set.ttl, set.values());
    }

    dns.delete(DnsDeleteRequest::new(&domain, "www", RecordType::A, "192.0.2.20")).await?;

    // Serve the zone: switch the domain to the platform nameservers.
    let applied = dns.apply(&domain).await?;
    println!("nameservers changed: {} -> {:?}", applied.name_servers_changed, applied.current_name_servers);
    Ok(())
}

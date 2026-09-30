use safebrowse_common::NetworkPolicy;
use safenet::SafeNetSession;

fn main() {
    let session = SafeNetSession::new(NetworkPolicy::safe_net_1());

    println!("SafeBrowse");
    println!("SafeNet state: {:?}", session.state());
}

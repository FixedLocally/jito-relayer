use std::net::IpAddr;
use log::info;
use solana_sdk::{packet::Meta, transaction::VersionedTransaction};

pub fn log_tx(meta: &Meta, tx: &VersionedTransaction) {
    let ip = match meta.addr {
        IpAddr::V4(ipv4) => ipv4.to_string(),
        IpAddr::V6(ipv6) => ipv6.to_string(),
    };
    let msg = &tx.message;
    let msg_header = msg.header();

    let sig = tx.signatures[0];
    let sig_count = msg_header.num_required_signatures;

    // default values
    let mut ix_count = msg.instructions().len() as u32;
    let mut compute_units: u32 = 0;
    let mut comput_unit_price: u64 = 0; // microlamports

    // locate compute ixs in the message
    let compute_budget_index = msg.static_account_keys().iter().position(|k| *k == solana_sdk::compute_budget::id());
    if let Some(index) = compute_budget_index {
        msg.instructions().iter().for_each(|ix| {
            if ix.program_id_index as usize == index {
                let len = ix.data.len();
                ix_count -= 1; // compute unit ixs doesn't increase the compute unit count
                if len == 0 {
                    return;
                }
                match ix.data[0] {
                    2 => { // setComputeUnitLimit
                        if len >= 5 {
                            compute_units = u32::from_le_bytes(ix.data[1..5].try_into().unwrap());
                        }
                    },
                    3 => { // setComputeUnitPrice
                        if len >= 9 {
                            comput_unit_price = u64::from_le_bytes(ix.data[1..9].try_into().unwrap());
                        }
                    },
                    _ => {},
                }
            }
        });
    }

    if compute_units == 0 {
        compute_units = ix_count * 200_000;
    }
    compute_units = compute_units.clamp(0, 1_400_000);
    let fee = (compute_units as u64 * comput_unit_price + 1_000_000 - 1) / 1_000_000 + sig_count as u64 * 5000;
    info!("txlog {} {} {} {} {}", ip, sig, compute_units, comput_unit_price, fee);
}
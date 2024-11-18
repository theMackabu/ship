use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::{cell::RefMut, str::FromStr};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::IpAddr;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        cidrnetmask => cidr::netmask(String),
        cidrrange => cidr::range(String),
        cidrhost => cidr::host(String, Number),
        cidrsubnets => cidr::subnets(String, Number)
    });
}

fn cidrnetmask(args: FuncArgs) -> Result<hcl::Value, String> {
    let prefix = args[0].as_str().unwrap();
    let network = IpNetwork::from_str(prefix).map_err(|e| format!("Invalid CIDR prefix: {}", e))?;

    match network {
        IpNetwork::V4(net) => Ok(hcl::Value::String(net.mask().to_string())),
        IpNetwork::V6(net) => Ok(hcl::Value::String(net.mask().to_string())),
    }
}

fn cidrrange(args: FuncArgs) -> Result<hcl::Value, String> {
    let prefix = args[0].as_str().unwrap();
    let network = IpNetwork::from_str(prefix).map_err(|e| format!("Invalid CIDR prefix: {}", e))?;

    let first = network.network();
    let last = network.broadcast();

    let result = vec![hcl::Value::String(first.to_string()), hcl::Value::String(last.to_string())];

    Ok(hcl::Value::Array(result))
}

fn cidrhost(args: FuncArgs) -> Result<hcl::Value, String> {
    let prefix = args[0].as_str().unwrap();
    let host_num = args[1].as_number().unwrap().as_f64().unwrap() as u32;

    let network = IpNetwork::from_str(prefix).map_err(|e| format!("Invalid CIDR prefix: {}", e))?;

    let host: IpAddr = match network {
        IpNetwork::V4(net) => {
            let network_u32: u32 = u32::from(net.network());
            let host_addr = network_u32 + host_num;
            IpAddr::V4(std::net::Ipv4Addr::from(host_addr))
        }
        IpNetwork::V6(net) => {
            let network_u128: u128 = u128::from(net.network());
            let host_addr = network_u128 + host_num as u128;
            IpAddr::V6(std::net::Ipv6Addr::from(host_addr))
        }
    };

    Ok(hcl::Value::String(host.to_string()))
}

fn cidrsubnets(args: FuncArgs) -> Result<hcl::Value, String> {
    let prefix = args[0].as_str().unwrap();
    let newbits = args[1].as_number().unwrap().as_f64().unwrap() as u8;

    let network = IpNetwork::from_str(prefix).map_err(|e| format!("Invalid CIDR prefix: {}", e))?;

    let mut subnets = Vec::new();
    let num_subnets = 1 << newbits;

    match network {
        IpNetwork::V4(net) => {
            let new_prefix_len = net.prefix() + newbits;
            if new_prefix_len > 32 {
                return Err("New prefix length exceeds 32 bits".to_string());
            }

            let network_u32: u32 = u32::from(net.network());
            let subnet_size = 1u32 << (32 - new_prefix_len);

            for i in 0..num_subnets {
                let subnet_start = network_u32 + (i as u32 * subnet_size);
                let new_net = Ipv4Network::new(std::net::Ipv4Addr::from(subnet_start), new_prefix_len).unwrap();
                subnets.push(hcl::Value::String(new_net.to_string()));
            }
        }
        IpNetwork::V6(net) => {
            let new_prefix_len = net.prefix() + newbits;
            if new_prefix_len > 128 {
                return Err("New prefix length exceeds 128 bits".to_string());
            }

            let network_u128: u128 = u128::from(net.network());
            let subnet_size = 1u128 << (128 - new_prefix_len);

            for i in 0..num_subnets {
                let subnet_start = network_u128 + (i as u128 * subnet_size);
                let new_net = Ipv6Network::new(std::net::Ipv6Addr::from(subnet_start), new_prefix_len).unwrap();
                subnets.push(hcl::Value::String(new_net.to_string()));
            }
        }
    }

    Ok(hcl::Value::Array(subnets))
}

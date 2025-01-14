use crate::nxdk::error::NxNetError;
use core::net::Ipv4Addr;
use core::ptr::null_mut;
use nxdk_sys::nxdk::net::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum NetworkConfigurationMode {
    #[default]
    Auto = nx_net_mode_t__NX_NET_AUTO as isize,
    Dhcp = nx_net_mode_t__NX_NET_DHCP as isize,
    Static = nx_net_mode_t__NX_NET_STATIC as isize,
}

#[derive(Debug)]
pub struct NetParametersBuilder {
    ipv4_mode: NetworkConfigurationMode,
    ipv6_mode: NetworkConfigurationMode,
    ipv4_ip: u32,
    ipv4_gateway: u32,
    ipv4_netmask: u32,
    ipv4_dns1: u32,
    ipv4_dns2: u32,
}

impl NetParametersBuilder {
    /// Creates a new builder with default values
    pub fn new() -> Self {
        Self {
            ipv4_mode: NetworkConfigurationMode::Auto,
            ipv6_mode: NetworkConfigurationMode::Auto,
            ipv4_ip: 0,
            ipv4_gateway: 0,
            ipv4_netmask: 0,
            ipv4_dns1: 0,
            ipv4_dns2: 0,
        }
    }

    /// Sets the IPv4 mode
    pub fn ipv4_mode(mut self, mode: NetworkConfigurationMode) -> Self {
        self.ipv4_mode = mode;
        self
    }

    /// Sets the IPv6 mode. Note that IPv6 isn't currently configured by NXDK.
    /// This is a placeholder.
    pub fn ipv6_mode(mut self, mode: NetworkConfigurationMode) -> Self {
        self.ipv6_mode = mode;
        self
    }

    /// Sets the IPv4 address
    pub fn ipv4_ip(mut self, ip: Ipv4Addr) -> Self {
        // An IP is usually BE, but testing indicates LE is the correct bit order here.
        self.ipv4_ip = u32::from_le_bytes(ip.octets());
        self
    }

    /// Sets the IPv4 gateway
    pub fn ipv4_gateway(mut self, gateway: Ipv4Addr) -> Self {
        self.ipv4_gateway = u32::from_le_bytes(gateway.octets());
        self
    }

    /// Sets the IPv4 netmask
    pub fn ipv4_netmask(mut self, netmask: Ipv4Addr) -> Self {
        self.ipv4_netmask = u32::from_le_bytes(netmask.octets());
        self
    }

    /// Sets the primary IPv4 DNS server
    pub fn ipv4_dns1(mut self, dns1: Ipv4Addr) -> Self {
        self.ipv4_dns1 = u32::from_le_bytes(dns1.octets());
        self
    }

    /// Sets the secondary IPv4 DNS server
    pub fn ipv4_dns2(mut self, dns2: Ipv4Addr) -> Self {
        self.ipv4_dns2 = u32::from_le_bytes(dns2.octets());
        self
    }

    /// Builds the `nx_net_parameters_t_` struct
    fn build(self) -> nx_net_parameters_t_ {
        nx_net_parameters_t_ {
            ipv4_mode: self.ipv4_mode as i32,
            ipv6_mode: self.ipv6_mode as i32,
            ipv4_ip: self.ipv4_ip,
            ipv4_gateway: self.ipv4_gateway,
            ipv4_netmask: self.ipv4_netmask,
            ipv4_dns1: self.ipv4_dns1,
            ipv4_dns2: self.ipv4_dns2,
        }
    }
}

pub fn nx_net_init() -> Result<(), NxNetError> {
    let err_code = unsafe {
        nxNetInit(null_mut())
    };

    if err_code != 0 {
        return Err(NxNetError::from(err_code));
    }

    Ok(())
}

pub fn nx_net_init_with(parameters: NetParametersBuilder) -> Result<(), NxNetError> {
    let param = parameters.build();

    let err_code = unsafe {
        nxNetInit(&param)
    };

    if err_code != 0 {
        return Err(NxNetError::from(err_code));
    }

    Ok(())
}

pub fn nx_net_shutdown() -> Result<(), NxNetError> {
    unimplemented!();
}

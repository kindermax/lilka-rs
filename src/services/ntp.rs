// use alloc::vec;
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use embassy_net::udp::UdpSocket;
use embassy_net::{dns, IpAddress};
use embassy_time::{Duration, Timer};
use esp_println::println;
use smoltcp::socket::udp;
use sntpc::{NtpContext, NtpTimestampGenerator};
use sntpc_net_embassy::UdpSocketWrapper;

use crate::services::{ClockService, NetworkService};

const SYNC_INTERVAL_SECS: u64 = 3600; // 1 hour
const USEC_IN_SEC: u64 = 1_000_000;

// Timestamp generator for sntpc
#[derive(Copy, Clone)]
struct TimestampGen;

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 {
        ClockService::get_current_time().as_second() as u64
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        (ClockService::get_current_time().subsec_nanosecond() / 1000) as u32
    }
}

#[embassy_executor::task]
pub async fn ntp_task(ntp_server: &'static str) {
    loop {
        if let Err(e) = sync_time(ntp_server).await {
            println!("NTP sync failed: {:?}", e);
        }

        Timer::after(Duration::from_secs(SYNC_INTERVAL_SECS)).await;
    }
}

async fn sync_time(ntp_server: &'static str) -> Result<(), &'static str> {
    let stack = NetworkService::wait_for_ip().await;

    // Resolve the ip
    let addrs = stack
        .dns_query(ntp_server, dns::DnsQueryType::A)
        .await
        .map_err(|_| "DNS failed")?;

    let ntp_address = addrs.first().copied().ok_or("No DNS results")?;

    let ntp_ip = match ntp_address {
        IpAddress::Ipv4(addr) => Ipv4Addr::from(addr.octets()),
    };

    let mut udp_rx_meta = [udp::PacketMetadata::EMPTY; 1];
    let mut udp_rx_buffer = [0u8; 512];

    let mut udp_tx_meta = [udp::PacketMetadata::EMPTY; 1];
    let mut udp_tx_buffer = [0u8; 512];

    let mut socket = UdpSocket::new(
        *stack,
        &mut udp_rx_meta,
        &mut udp_rx_buffer,
        &mut udp_tx_meta,
        &mut udp_tx_buffer,
    );

    // 0 means stack picks ephemeral port
    socket.bind(0).map_err(|_| "Bind failed")?;

    // Get time via SNTP
    let socket_wrapper = UdpSocketWrapper::new(socket);
    let context = NtpContext::new(TimestampGen);

    let result = sntpc::get_time(
        SocketAddr::V4(SocketAddrV4::new(ntp_ip, 123)),
        &socket_wrapper,
        context,
    )
    .await
    .map_err(|_| "SNTP request failed")?;

    // Convert NTP time to microseconds and set RTC
    let timestamp_us =
        (result.sec() as u64 * USEC_IN_SEC) + ((result.sec_fraction() as u64 * USEC_IN_SEC) >> 32);

    ClockService::set_current_time(timestamp_us);

    Ok(())
}

use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

use embassy_futures::join::join;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_println::println;
use esp_radio::wifi::{ClientConfig, ModeConfig, ScanConfig, WifiController, WifiEvent};
use static_cell::StaticCell;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: StaticCell<$t> = StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const SSID: &str = "chilla";
const PASSWORD: &str = "40454540";

// Static storage for network stack pointer - accessible from other tasks
// Safety: Stack is initialized once and never moved. Access is read-only after init.
static NETWORK_STACK: AtomicPtr<Stack<'static>> = AtomicPtr::new(ptr::null_mut());

/// Handle for accessing network functionality from other tasks
pub struct NetworkService;

// Usage from other tasks:
//
// // Wait for network to be ready with IP
// let stack = NetworkService::wait_for_ip().await;
//
// // Make TCP/UDP connections using the stack
// let mut rx_buffer = [0; 1024];
// let mut tx_buffer = [0; 1024];
// let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
impl NetworkService {
    /// Get the network stack for TCP/UDP operations.
    /// Returns None if network_task hasn't initialized yet.
    pub fn stack() -> Option<&'static Stack<'static>> {
        let ptr = NETWORK_STACK.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            // Safety: The pointer is only set once during network_task initialization
            // and points to a static allocation that lives for 'static.
            Some(unsafe { &*ptr })
        }
    }

    /// Wait until the network stack is available
    pub async fn wait_for_stack() -> &'static Stack<'static> {
        loop {
            if let Some(stack) = Self::stack() {
                return stack;
            }
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    /// Wait until we have an IP address
    pub async fn wait_for_ip() -> &'static Stack<'static> {
        let stack = Self::wait_for_stack().await;
        stack.wait_config_up().await;
        stack
    }
}

/// Main network task - spawn this from main
#[embassy_executor::task]
pub async fn network_task(wifi: WIFI<'static>) {
    // Initialize radio
    let radio_init = mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize radio")
    );

    // Initialize WiFi
    let (controller, interfaces) = esp_radio::wifi::new(radio_init, wifi, Default::default())
        .expect("Failed to initialize Wi-Fi");

    // Initialize network stack
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, mut runner) = embassy_net::new(
        interfaces.sta,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    // Store stack in static for access from other tasks
    let stack: &'static mut Stack<'static> = mk_static!(Stack<'static>, stack);
    NETWORK_STACK.store(stack as *mut _, Ordering::Release);

    // Run both connection manager and network runner concurrently
    join(connection_loop(controller), runner.run()).await;
}

async fn connection_loop(mut controller: WifiController<'static>) {
    println!("WiFi connection manager started");

    loop {
        if matches!(controller.is_connected(), Ok(true)) {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await;
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let station_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&station_config).unwrap();
            println!("Starting WiFi");
            controller.start_async().await.unwrap();
            println!("WiFi started");

            // Optional: scan
            let scan_config = ScanConfig::default().with_max(10);
            let result = controller
                .scan_with_config_async(scan_config)
                .await
                .unwrap();
            for ap in result {
                println!("{:?}", ap);
            }
        }

        println!("Connecting to WiFi...");
        match controller.connect_async().await {
            Ok(_) => println!("WiFi connected!"),
            Err(e) => {
                println!("WiFi connection failed: {e:?}");
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

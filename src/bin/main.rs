#![no_std]
#![no_main]

use core::net::SocketAddrV4;

use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::{
    AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use esp_wifi::EspWifiController;
use iggy_embedded::client::TcpClient;
use iggy_embedded::command::{Command as _, GetMe, LoginUser, Ping};
use log::{debug, error, info};
use static_cell::StaticCell;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    tcp_server: &'static str,
    #[default("")]
    username: &'static str,
    #[default("")]
    password: &'static str,
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: StaticCell<$t> = StaticCell::new();
        let x = STATIC_CELL.init(($val));
        x
    }};
}

#[embassy_executor::task]
async fn connection_task(mut wifi_ctrl: WifiController<'static>) {
    info!("Start connection task");

    let client_cfg = Configuration::Client(ClientConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().unwrap(),
        password: CONFIG.wifi_psk.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    wifi_ctrl.set_configuration(&client_cfg).unwrap();
    info!("Starting Wi-Fi");
    wifi_ctrl.start_async().await.unwrap();
    info!("Wi-Fi started");

    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            wifi_ctrl.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }

        match wifi_ctrl.connect_async().await {
            Ok(_) => info!("Wi-Fi connected"),
            Err(e) => {
                error!("Failed to connect to Wi-Fi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn ping(tcp_client: &'static TcpClient<'static>) {
    loop {
        info!("PING");
        if let Ok(bytes) = tcp_client.send_command(Ping).await {
            debug!("Received {} bytes in response", bytes.len());
            if Ping::from_response(bytes).is_err() {
                error!("Failed to parse ping response");
            }
            info!("PONG");
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let cfg = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(cfg);

    esp_alloc::heap_allocator!(size: 72 * 1024); // 72 KiB

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    info!("Embassy initialized!");

    let mut rng = Rng::new(peripherals.RNG);
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let _tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let timg1 = TimerGroup::new(peripherals.TIMG1);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController,
        esp_wifi::init(timg1.timer0, rng, peripherals.RADIO_CLK).unwrap()
    );

    let (wifi_ctrl, wifi_interfaces) =
        esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();

    let net_cfg = embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default());

    let (stack, runner) = embassy_net::new(
        wifi_interfaces.sta,
        net_cfg,
        mk_static!(StackResources<3>, StackResources::new()),
        net_seed,
    );

    spawner.spawn(connection_task(wifi_ctrl)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();

    stack.wait_config_up().await;

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(cfg) = stack.config_v4() {
            info!("Got IP: {}", cfg.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let ip = CONFIG.tcp_server.parse::<SocketAddrV4>().unwrap();
    info!("TCP Server: {:?}", ip);

    let rx_buffer = mk_static!([u8; 1024], [0u8; 1024]);
    let tx_buffer = mk_static!([u8; 1024], [0u8; 1024]);
    let tcp_socket = TcpSocket::new(stack, rx_buffer, tx_buffer);

    debug!("Connecting to TCP Server");
    let tcp_client = &*mk_static!(TcpClient<'static>, TcpClient::new(tcp_socket));
    tcp_client.connect(ip).await.unwrap();
    debug!("Connected to TCP Server");

    spawner.spawn(ping(tcp_client)).unwrap();

    if tcp_client.send_command(GetMe).await.is_err() {
        error!("Error while sending GetMe before authentication");
    } else {
        info!("Successfully sent GetMe before authentication");
    }

    let response = tcp_client
        .send_command(LoginUser::new(CONFIG.username, CONFIG.password, None, None))
        .await
        .unwrap();
    let identity_info = LoginUser::from_response(response).unwrap();
    info!("identity_info: {identity_info:?}");

    if tcp_client.send_command(GetMe).await.is_err() {
        error!("Error while sending GetMe after authentication");
    } else {
        info!("Successfully sent GetMe after authentication");
    }

    loop {
        Timer::after(Duration::from_secs(2)).await;
    }
}

use {
    embedded_svc::{
        ipv4::IpInfo,
        wifi::{AuthMethod, ClientConfiguration, Configuration},
    },
    esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        hal::modem::Modem,
        nvs::EspDefaultNvsPartition,
        timer::EspTaskTimerService,
        wifi::{AsyncWifi, EspWifi},
    },
    heapless::String,
};

use log::info;
pub fn create_wifi(modem: Modem) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let inner_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;

    let af = AsyncWifi::wrap(inner_wifi, sys_loop, timer_service)?;

    Ok(af)
}

pub async fn connect_wifi(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> anyhow::Result<IpInfo> {
    info!("Wifi connecting to {ssid} with pass ****");
    let mut sssid: String<32> = String::new();
    sssid.push_str(ssid).unwrap();
    let mut spassword: String<64> = String::new();
    spassword.push_str(password).unwrap();
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: sssid,
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: spassword,
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    info!("Wifi started");

    wifi.connect().await?;
    info!("Wifi connected");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(ip_info)
}

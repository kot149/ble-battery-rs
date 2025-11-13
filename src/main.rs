use std::error::Error;

use bluest::Adapter;
use tracing::info;
use tracing::metadata::LevelFilter;
use uuid::Uuid;
use bluest::btuuid::descriptors::CHARACTERISTIC_USER_DESCRIPTION;

const BATTERY_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180F_0000_1000_8000_00805F9B34FB);
const BATTERY_LEVEL_UUID: Uuid = Uuid::from_u128(0x00002A19_0000_1000_8000_00805F9B34FB);

fn main() -> Result<(), Box<dyn Error>> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    let adapter = Adapter::default().await.ok_or("Bluetooth adapter not found")?;
    adapter.wait_available().await?;

    info!("getting connected devices");
    let devices = adapter.connected_devices().await?;
    // let devices = adapter.connected_devices_with_services(&[BATTERY_SERVICE_UUID, BATTERY_LEVEL_UUID]).await?;
    for target_device in devices.iter() {
        info!("- Found device: {:?}", target_device);

        adapter
        .connect_device(target_device)
        .await
        .map_err(|e| e.to_string())?;

        let services = target_device.services().await.map_err(|e| e.to_string())?;

        for battery_service in services.iter().filter(|service| service.uuid() == BATTERY_SERVICE_UUID) {
            let characteristics = battery_service.characteristics().await.map_err(|e| e.to_string())?;

            for battery_level_characteristic in characteristics.iter().filter(|c| c.uuid() == BATTERY_LEVEL_UUID) {
                let battery_levels = battery_level_characteristic.read().await.map_err(|e| e.to_string())?;

                for battery_level in battery_levels.iter() {
                    info!("Battery level: {:?}", battery_level);
                }
                let descriptors = battery_level_characteristic
                    .descriptors()
                    .await
                    .map_err(|e| e.to_string())?;

                if let Some(user_description_descriptor) = descriptors.iter().find(|d| d.uuid() == CHARACTERISTIC_USER_DESCRIPTION) {
                    let desc_value = user_description_descriptor.read().await.map_err(|e| e.to_string())?;
                    if let Ok(desc_str) = String::from_utf8(desc_value.clone()) {
                        info!("User description: {:?}", desc_str);
                    }
                } else {
                    info!("User description: (not found)");
                }
            }
        }

        // rssi() is only supported on macOS
        #[cfg(target_os = "macos")]{
            let rssi = target_device.rssi().await?;
            info!("  - RSSI: {:?}", rssi);
        }

        adapter.disconnect_device(target_device).await.map_err(|e| e.to_string())?;
    }
    info!("done");

    Ok(())
}
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
    // let devices = adapter.connected_devices().await?;
    let devices = adapter.connected_devices_with_services(&[BATTERY_SERVICE_UUID, BATTERY_LEVEL_UUID]).await?;
    for device in devices {
        info!("found device: {:?}", device);
        let name = device.name()?.to_string();
        if name.contains("roBa") || name.contains("moNa2") {
            info!("target device found: {}", name);
            adapter.connect_device(&device).await?;
            let services = device.services().await?;
            for service in services {
                if service.uuid() == BATTERY_SERVICE_UUID {
                    info!("  found battery service: {:?}", service.uuid());
                    let characteristics = service.characteristics().await?;
                    for characteristic in characteristics {
                        if characteristic.uuid() == BATTERY_LEVEL_UUID {
                            info!("    found battery level characteristic: {:?}", characteristic.uuid());
                            let value = characteristic.read().await?;
                            println!("バッテリーレベル: {:?}", value);

                            // User Descriptionの取得
                            let descriptors = characteristic.descriptors().await?;
                            for descriptor in descriptors {
                                if descriptor.uuid() == CHARACTERISTIC_USER_DESCRIPTION {
                                    let desc_value = descriptor.read().await?;
                                    if let Ok(desc_str) = String::from_utf8(desc_value.clone()) {
                                        println!("ユーザー記述子: {}", desc_str);
                                    } else {
                                        println!("ユーザー記述子(バイナリ): {:?}", desc_value);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    info!("done");

    Ok(())
}
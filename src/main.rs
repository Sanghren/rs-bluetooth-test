use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::time::Duration;
use btleplug::api::{bleuuid::uuid_from_u16, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::api::bleuuid::BleUuid;
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::time;
use futures::stream::StreamExt;
use macaddr::MacAddr6;
use ruuvi_sensor_protocol::{Humidity, MacAddress, ParseError, Pressure, SensorValues, Temperature};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();
    let mut aliases = HashMap::new();
    // ADD HERE LIST OF MAC ADDRESS -> ALIAS FOR YOUR RUUVITAGs
    //     aliases.insert("XX:XX:XX:XX:XX:XX", "Name");

    let adapters = manager.adapters().await?;

    let mut central = adapters.into_iter().nth(0).unwrap();

    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => {
                match central.peripheral(&id).await {
                    Ok(periph) => {
                        if periph.properties().await.unwrap().unwrap().local_name.unwrap_or("Nop".to_string()).contains("Ruuvi") {
                            for (key, value) in manufacturer_data.iter() {
                                let result = SensorValues::from_manufacturer_specific_data(key.clone(), value).unwrap();
                                let mac_addr = MacAddr6::from(result.mac_address().unwrap());
                                if aliases.contains_key(&*mac_addr.to_string()) {
                                    let alias = aliases.get(&*mac_addr.to_string()).unwrap();
                                    println!("{} has sent new data : {} {} {}", alias, result.temperature_as_millicelsius().unwrap(), result.humidity_as_ppm().unwrap(), result.pressure_as_pascals().unwrap());
                                } else {
                                    eprintln!("This is a RuuviTag we dot not know about ? {} // {}", id, mac_addr);
                                }
                            }
                        }
                    }
                    Err(err) => { eprintln!("Issue while trying to fetch {}", id) }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

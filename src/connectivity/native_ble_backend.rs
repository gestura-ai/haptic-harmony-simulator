#![cfg_attr(not(feature = "native-ble"), allow(dead_code))]

//! Cross-platform native BLE peripheral runtime.

#[cfg(feature = "native-ble")]
use anyhow::{Context, Result, anyhow, bail};
#[cfg(not(feature = "native-ble"))]
use anyhow::{Result, bail};
#[cfg(feature = "native-ble")]
use ble_peripheral_rust::{
    Peripheral, PeripheralImpl,
    gatt::{
        characteristic::Characteristic,
        peripheral_event::{
            PeripheralEvent, ReadRequestResponse, RequestResponse, WriteRequestResponse,
        },
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
};
#[cfg(feature = "native-ble")]
use tokio::time::{Duration, sleep};
use tokio::{sync::mpsc, task::JoinHandle};
#[cfg(feature = "native-ble")]
use uuid::Uuid;

use crate::connectivity::ble_peripheral::BlePeripheralTask;
#[cfg(feature = "native-ble")]
use crate::connectivity::ble_peripheral::ring_uuids;

#[derive(Debug, Clone)]
pub(crate) enum NativeBleCommand {
    Stop,
    UpdateCharacteristic { uuid: String, value: Vec<u8> },
}

pub(crate) struct NativeBleRuntime {
    pub(crate) command_tx: mpsc::UnboundedSender<NativeBleCommand>,
    pub(crate) task: JoinHandle<()>,
}

#[cfg(feature = "native-ble")]
pub(crate) async fn start_native_ble_runtime(
    task_context: BlePeripheralTask,
    device_name: String,
) -> Result<NativeBleRuntime> {
    let service = build_primary_service(&task_context).await?;
    let (event_tx, event_rx) = mpsc::channel(256);
    let mut peripheral = Peripheral::new(event_tx)
        .await
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to create native BLE peripheral")?;
    wait_for_power(&mut peripheral).await?;
    peripheral
        .add_service(&service)
        .await
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to register BLE GATT service")?;
    peripheral
        .start_advertising(&device_name, &[service.uuid])
        .await
        .map_err(|error| anyhow!(error.to_string()))
        .with_context(|| format!("failed to start native BLE advertising as '{device_name}'"))?;

    task_context
        .set_connection_state(crate::connectivity::ConnectionState::Connected)
        .await;

    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let task = tokio::spawn(run_native_ble_runtime(
        peripheral,
        task_context,
        event_rx,
        command_rx,
    ));

    Ok(NativeBleRuntime { command_tx, task })
}

#[cfg(not(feature = "native-ble"))]
pub(crate) async fn start_native_ble_runtime(
    _task_context: BlePeripheralTask,
    _device_name: String,
) -> Result<NativeBleRuntime> {
    bail!("native-ble feature is not enabled")
}

#[cfg(feature = "native-ble")]
async fn wait_for_power(peripheral: &mut Peripheral) -> Result<()> {
    for _attempt in 0..50 {
        if peripheral
            .is_powered()
            .await
            .map_err(|error| anyhow!(error.to_string()))?
        {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    bail!("native BLE adapter never reported powered state")
}

#[cfg(feature = "native-ble")]
async fn run_native_ble_runtime(
    mut peripheral: Peripheral,
    task_context: BlePeripheralTask,
    mut event_rx: mpsc::Receiver<PeripheralEvent>,
    mut command_rx: mpsc::UnboundedReceiver<NativeBleCommand>,
) {
    loop {
        tokio::select! {
            maybe_command = command_rx.recv() => match maybe_command {
                Some(NativeBleCommand::Stop) | None => {
                    let _ = peripheral.stop_advertising().await;
                    break;
                }
                Some(NativeBleCommand::UpdateCharacteristic { uuid, value }) => {
                    match Uuid::parse_str(&uuid) {
                        Ok(uuid) => {
                            if let Err(error) = peripheral.update_characteristic(uuid, value).await {
                                tracing::warn!(%error, characteristic = %uuid, "Failed to publish native BLE characteristic update");
                            }
                        }
                        Err(error) => tracing::warn!(%error, characteristic = %uuid, "Invalid BLE characteristic UUID for native update"),
                    }
                }
            },
            maybe_event = event_rx.recv() => match maybe_event {
                Some(event) => handle_native_event(&task_context, event).await,
                None => break,
            }
        }
    }
}

#[cfg(feature = "native-ble")]
async fn handle_native_event(task_context: &BlePeripheralTask, event: PeripheralEvent) {
    match event {
        PeripheralEvent::StateUpdate { is_powered } => {
            tracing::debug!(is_powered, "Native BLE adapter state update");
        }
        PeripheralEvent::CharacteristicSubscriptionUpdate {
            request,
            subscribed,
        } => {
            if let Err(error) = task_context
                .apply_subscription_update(
                    &request.client,
                    &request.characteristic.to_string(),
                    subscribed,
                )
                .await
            {
                tracing::warn!(%error, client = %request.client, characteristic = %request.characteristic, "Failed to apply native BLE subscription update");
            }
        }
        PeripheralEvent::ReadRequest {
            request,
            offset,
            responder,
        } => {
            let response = match task_context
                .read_characteristic(&request.characteristic.to_string())
                .await
            {
                Ok(value) if offset as usize <= value.len() => ReadRequestResponse {
                    value: value[offset as usize..].to_vec(),
                    response: RequestResponse::Success,
                },
                Ok(_) => ReadRequestResponse {
                    value: Vec::new(),
                    response: RequestResponse::InvalidOffset,
                },
                Err(error) => {
                    tracing::warn!(%error, characteristic = %request.characteristic, "Failed native BLE read request");
                    ReadRequestResponse {
                        value: Vec::new(),
                        response: RequestResponse::UnlikelyError,
                    }
                }
            };
            let _ = responder.send(response);
        }
        PeripheralEvent::WriteRequest {
            request,
            value,
            offset,
            responder,
        } => {
            let response = if offset != 0 {
                WriteRequestResponse {
                    response: RequestResponse::InvalidOffset,
                }
            } else {
                match task_context
                    .write_characteristic(&request.characteristic.to_string(), value)
                    .await
                {
                    Ok(()) => WriteRequestResponse {
                        response: RequestResponse::Success,
                    },
                    Err(error) => {
                        tracing::warn!(%error, characteristic = %request.characteristic, "Failed native BLE write request");
                        WriteRequestResponse {
                            response: RequestResponse::UnlikelyError,
                        }
                    }
                }
            };
            let _ = responder.send(response);
        }
    }
}

#[cfg(feature = "native-ble")]
async fn build_primary_service(task_context: &BlePeripheralTask) -> Result<Service> {
    let _ = task_context;
    Ok(Service {
        uuid: parse_uuid(ring_uuids::HAPTIC_SERVICE_UUID)?,
        primary: true,
        characteristics: vec![
            Characteristic {
                uuid: parse_uuid(ring_uuids::GESTURE_EVENT_UUID)?,
                properties: vec![CharacteristicProperty::Notify],
                permissions: vec![],
                value: None,
                descriptors: vec![],
            },
            Characteristic {
                uuid: parse_uuid(ring_uuids::HAPTIC_COMMAND_UUID)?,
                properties: vec![
                    CharacteristicProperty::Write,
                    CharacteristicProperty::WriteWithoutResponse,
                ],
                permissions: vec![AttributePermission::Writeable],
                value: None,
                descriptors: vec![],
            },
            Characteristic {
                uuid: parse_uuid(ring_uuids::BATTERY_LEVEL_UUID)?,
                properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
                permissions: vec![AttributePermission::Readable],
                value: None,
                descriptors: vec![],
            },
            Characteristic {
                uuid: parse_uuid(ring_uuids::STATE_SNAPSHOT_UUID)?,
                properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Notify],
                permissions: vec![AttributePermission::Readable],
                value: None,
                descriptors: vec![],
            },
            Characteristic {
                uuid: parse_uuid(ring_uuids::OTA_UPDATE_UUID)?,
                properties: vec![
                    CharacteristicProperty::Write,
                    CharacteristicProperty::WriteWithoutResponse,
                ],
                permissions: vec![AttributePermission::Writeable],
                value: None,
                descriptors: vec![],
            },
        ],
    })
}

#[cfg(feature = "native-ble")]
fn parse_uuid(uuid: &str) -> Result<Uuid> {
    Ok(Uuid::parse_str(uuid)?)
}

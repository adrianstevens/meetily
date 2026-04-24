use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::devices::configuration::{AudioDevice, DeviceType};

/// Configure Linux audio devices using ALSA/PulseAudio
pub fn configure_linux_audio(host: &cpal::Host) -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    // Add the default input device first (needed for WSLg/PulseAudio where no hardware ALSA cards exist)
    if let Some(device) = host.default_input_device() {
        if let Ok(name) = device.name() {
            devices.push(AudioDevice::new(name, DeviceType::Input));
        }
    }

    // Add any additional input devices
    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            if !devices.iter().any(|d| d.name == name) {
                devices.push(AudioDevice::new(name, DeviceType::Input));
            }
        }
    }

    // Add PulseAudio monitor sources for system audio.
    // On WSLg, RDPSink.monitor is exposed as the ALSA device "meetily_monitor" via ~/.asoundrc.
    // On other Linux setups, scan input_devices for anything with "monitor" in the name.
    // Add PulseAudio monitor sources for system audio.
    // Use host.devices() (all devices) since user-defined .asoundrc devices may not appear
    // in input_devices() or output_devices() alone.
    // On WSLg: "meetily_monitor" is defined in ~/.asoundrc pointing at RDPSink.monitor.
    let alsa_host = cpal::default_host();
    if let Ok(all_devs) = alsa_host.devices() {
        let names: Vec<String> = all_devs.filter_map(|d| d.name().ok()).collect();

        let monitor_name = names.iter()
            .find(|n| *n == "meetily_monitor")
            .or_else(|| names.iter().find(|n| n.contains("monitor")))
            .cloned();

        if let Some(name) = monitor_name {
            devices.push(AudioDevice::new(name, DeviceType::Output));
        }
    }

    Ok(devices)
}
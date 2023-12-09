use std::ffi::{c_char, CStr};
use anyhow::anyhow;
use ash::vk;
use ash::vk::{API_VERSION_1_3, PhysicalDevice, StructureType, SurfaceKHR};
use log::info;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use crate::platform::{create_surface, get_required_instance_extensions};

mod platform;

struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    event_loop: Option<EventLoop<()>>,
    window: Window,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
}

impl App {
    unsafe fn new() -> anyhow::Result<App> {
        let entry = ash::Entry::load()?;

        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Hello!")
            .build(&event_loop)?;

        let app_info = vk::ApplicationInfo::builder()
            .api_version(API_VERSION_1_3).build();

        let required_extensions = get_required_instance_extensions(&window)?;

        let required_extensions_ptrs: Vec<*const c_char> = required_extensions.iter()
            .map(|s| s.as_ptr())
            .collect();

        let instance = entry.create_instance(&vk::InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: std::ptr::null_mut(),
            flags: Default::default(),
            p_application_info: &app_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_extension_count: required_extensions.len() as u32,
            pp_enabled_extension_names: required_extensions_ptrs.as_ptr(),
        }, None)?;
        info!("Created instance");

        let physical_device = instance.enumerate_physical_devices()?.get(0).ok_or(anyhow!("No GPU"))?.clone();

        let physical_device_properties = instance.get_physical_device_properties(physical_device);
        let device_name = CStr::from_ptr(physical_device_properties.device_name.as_ptr());
        info!("Selected physical device: {}", device_name.to_str().unwrap_or("(error)"));

        let surface = create_surface(&window, &entry, &instance)?;
        info!("Created surface");

        Ok(Self {
            entry,
            instance,
            event_loop: Some(event_loop),
            window,
            surface,
            physical_device,
        })
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    info!("Hello!");

    let app = unsafe { App::new() }?;

    Ok(())
}

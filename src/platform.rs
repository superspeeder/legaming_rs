use std::ffi::{c_int, c_ulong, c_void, CStr};
use std::num::{NonZeroIsize, NonZeroU32};
use std::ptr::NonNull;
use ash::extensions::khr;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{HINSTANCE, HWND};
use thiserror::Error;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;

#[derive(Error, Debug)]
pub enum CreateSurfaceError {
    #[error("Unsupported System")]
    Unsupported
}

pub unsafe fn create_surface(window: &Window, entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<vk::SurfaceKHR> {
    let window_handle = window.window_handle()?.as_raw();
    let display_handle = window.display_handle()?.as_raw();

    match (window_handle, display_handle) {
        (RawWindowHandle::Win32(window_handle), RawDisplayHandle::Windows(_)) => {
            create_win32_surface(window_handle.hinstance, window_handle.hwnd, entry, instance)
        }
        (RawWindowHandle::Wayland(window_handle), RawDisplayHandle::Wayland(display_handle)) => {
            create_wayland_surface(window_handle.surface, display_handle.display, entry, instance)
        }
        (RawWindowHandle::Xcb(window_handle), RawDisplayHandle::Xcb(display_handle)) => {
            create_xcb_surface(window_handle.window, display_handle.connection, entry, instance)
        }
        (RawWindowHandle::Xlib(window_handle), RawDisplayHandle::Xlib(_display_handle)) => {
            create_xlib_surface(window_handle.window, entry, instance)
        }
        (_, _) => Err(CreateSurfaceError::Unsupported.into())
    }
}

#[cfg(windows)]
unsafe fn get_hinstance() -> HINSTANCE {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

    GetModuleHandleA(std::ptr::null()) as HINSTANCE
}

#[cfg(not(windows))]
unsafe fn get_hinstance() -> HINSTANCE {
    unimplemented!("get_hinstance() not implemented for non-windows os");
}

unsafe fn create_win32_surface(hinstance: Option<NonZeroIsize>, hwnd: NonZeroIsize, entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<vk::SurfaceKHR> {
    let hinstance_value: HINSTANCE = hinstance.map_or_else(|| get_hinstance(), |v| {
        v.get() as HINSTANCE
    });

    let hwnd_value = hwnd.get() as HWND;

    let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .hwnd(hwnd_value)
        .hinstance(hinstance_value)
        .build();

    let surface_fn = khr::Win32Surface::new(entry, instance);
    match surface_fn.create_win32_surface(&create_info, None) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.into()),
    }
}

unsafe fn create_wayland_surface(surface: NonNull<c_void>, display: NonNull<c_void>, entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<vk::SurfaceKHR> {
    let create_info = vk::WaylandSurfaceCreateInfoKHR::builder()
        .surface(surface.as_ptr())
        .display(display.as_ptr())
        .build();

    let surface_fn = khr::WaylandSurface::new(entry, instance);

    match surface_fn.create_wayland_surface(&create_info, None) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.into()),
    }
}

unsafe fn create_xcb_surface(window: NonZeroU32, connection: Option<NonNull<c_void>>, entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<vk::SurfaceKHR> {
    let create_info = vk::XcbSurfaceCreateInfoKHR::builder()
        .window(window.get())
        .connection(connection.map_or(std::ptr::null_mut(), |v| v.as_ptr()))
        .build();

    let surface_fn = khr::XcbSurface::new(entry, instance);

    match surface_fn.create_xcb_surface(&create_info, None) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.into()),
    }
}

unsafe fn create_xlib_surface(window: c_ulong, entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<vk::SurfaceKHR> {
    let create_info = vk::XlibSurfaceCreateInfoKHR::builder()
        .window(window)
        .build();

    let surface_fn = khr::XlibSurface::new(entry, instance);

    match surface_fn.create_xlib_surface(&create_info, None) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.into()),
    }
}

pub fn get_required_instance_extensions(window: &Window) -> anyhow::Result<Vec<&'static CStr>> {
    let window_handle = window.window_handle()?.as_raw();
    let display_handle = window.display_handle()?.as_raw();

    match (window_handle, display_handle) {
        (RawWindowHandle::Win32(window_handle), RawDisplayHandle::Windows(_)) => {
            Ok(vec![khr::Surface::name(), khr::Win32Surface::name()])
        }
        (RawWindowHandle::Wayland(window_handle), RawDisplayHandle::Wayland(display_handle)) => {
            Ok(vec![khr::Surface::name(), khr::WaylandSurface::name()])
        }
        (RawWindowHandle::Xcb(window_handle), RawDisplayHandle::Xcb(display_handle)) => {
            Ok(vec![khr::Surface::name(), khr::XcbSurface::name()])
        }
        (RawWindowHandle::Xlib(window_handle), RawDisplayHandle::Xlib(_display_handle)) => {
            Ok(vec![khr::Surface::name(), khr::XlibSurface::name()])
        }
        (_, _) => Err(CreateSurfaceError::Unsupported.into())
    }
}

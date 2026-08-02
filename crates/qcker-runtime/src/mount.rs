use nix::mount::{mount, MsFlags};
use std::fs;
use std::path::Path;

use qcker_common::error::{QckerError, Result};

pub fn setup_rootfs(
    lower_dirs: &[&Path],
    upper_dir: &Path,
    work_dir: &Path,
    merged_dir: &Path,
    rootless: bool,
) -> Result<()> {
    fs::create_dir_all(upper_dir)
        .map_err(|e| QckerError::mount(format!("Failed to create upper dir: {}", e)))?;
    fs::create_dir_all(work_dir)
        .map_err(|e| QckerError::mount(format!("Failed to create work dir: {}", e)))?;
    fs::create_dir_all(merged_dir)
        .map_err(|e| QckerError::mount(format!("Failed to create merged dir: {}", e)))?;

    if rootless {
        setup_fuse_overlayfs(lower_dirs, upper_dir, work_dir, merged_dir)?;
    } else {
        setup_kernel_overlayfs(lower_dirs, upper_dir, work_dir, merged_dir)?;
    }

    Ok(())
}

fn setup_kernel_overlayfs(
    lower_dirs: &[&Path],
    upper_dir: &Path,
    work_dir: &Path,
    merged_dir: &Path,
) -> Result<()> {
    let lowerdir = lower_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(":");

    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        lowerdir,
        upper_dir.display(),
        work_dir.display()
    );

    mount(
        Some("overlay"),
        merged_dir,
        Some("overlay"),
        MsFlags::MS_NOATIME,
        Some(options.as_str()),
    )
    .map_err(|e| QckerError::mount(format!("Failed to mount overlayfs: {}", e)))?;

    Ok(())
}

fn setup_fuse_overlayfs(
    lower_dirs: &[&Path],
    upper_dir: &Path,
    work_dir: &Path,
    merged_dir: &Path,
) -> Result<()> {
    let lowerdir = lower_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(":");

    let output = std::process::Command::new("fuse-overlayfs")
        .args([
            "-o",
            &format!(
                "lowerdir={},upperdir={},workdir={}",
                lowerdir,
                upper_dir.display(),
                work_dir.display()
            ),
            merged_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| QckerError::mount(format!("Failed to run fuse-overlayfs: {}", e)))?;

    if !output.status.success() {
        return Err(QckerError::mount(format!(
            "fuse-overlayfs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

pub fn mount_proc(rootfs: &Path) -> Result<()> {
    let proc_path = rootfs.join("proc");
    fs::create_dir_all(&proc_path)
        .map_err(|e| QckerError::mount(format!("Failed to create /proc: {}", e)))?;

    match mount(
        Some("proc"),
        &proc_path,
        Some("proc"),
        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
        None::<&str>,
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("Failed to mount proc (expected in rootless): {}", e);
            Ok(())
        }
    }
}

pub fn mount_sys(rootfs: &Path) -> Result<()> {
    let sys_path = rootfs.join("sys");
    fs::create_dir_all(&sys_path)
        .map_err(|e| QckerError::mount(format!("Failed to create /sys: {}", e)))?;

    match mount(
        Some("sysfs"),
        &sys_path,
        Some("sysfs"),
        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_RDONLY,
        None::<&str>,
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("Failed to mount sys (expected in rootless): {}", e);
            Ok(())
        }
    }
}

pub fn mount_dev(rootfs: &Path) -> Result<()> {
    let dev_path = rootfs.join("dev");
    fs::create_dir_all(&dev_path)
        .map_err(|e| QckerError::mount(format!("Failed to create /dev: {}", e)))?;

    match mount(
        Some("tmpfs"),
        &dev_path,
        Some("tmpfs"),
        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
        Some("mode=755"),
    ) {
        Ok(()) => {}
        Err(e) => {
            tracing::warn!("Failed to mount tmpfs dev (using existing): {}", e);
        }
    }

    let _ = create_dev_nodes(&dev_path);

    Ok(())
}

fn create_dev_nodes(dev_path: &Path) -> Result<()> {
    use nix::sys::stat::{makedev, mknod, Mode, SFlag};

    let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP | Mode::S_IROTH | Mode::S_IWOTH;

    let null_path = dev_path.join("null");
    mknod(&null_path, SFlag::S_IFCHR, mode, makedev(1, 3))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/null: {}", e)))?;

    let zero_path = dev_path.join("zero");
    mknod(&zero_path, SFlag::S_IFCHR, mode, makedev(1, 5))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/zero: {}", e)))?;

    let urandom_path = dev_path.join("urandom");
    mknod(&urandom_path, SFlag::S_IFCHR, mode, makedev(1, 9))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/urandom: {}", e)))?;

    let random_path = dev_path.join("random");
    mknod(&random_path, SFlag::S_IFCHR, mode, makedev(1, 8))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/random: {}", e)))?;

    let tty_path = dev_path.join("tty");
    mknod(&tty_path, SFlag::S_IFCHR, mode, makedev(5, 0))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/tty: {}", e)))?;

    let console_path = dev_path.join("console");
    mknod(&console_path, SFlag::S_IFCHR, mode, makedev(5, 1))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/console: {}", e)))?;

    let ptmx_path = dev_path.join("ptmx");
    mknod(&ptmx_path, SFlag::S_IFCHR, mode, makedev(5, 2))
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/ptmx: {}", e)))?;

    let pts_dir = dev_path.join("pts");
    std::fs::create_dir_all(&pts_dir)
        .map_err(|e| QckerError::mount(format!("Failed to create /dev/pts: {}", e)))?;

    let fd_path = dev_path.join("fd");
    if fd_path.exists() == false {
        std::fs::create_dir_all(&fd_path)
            .map_err(|e| QckerError::mount(format!("Failed to create /dev/fd: {}", e)))?;
    }

    Ok(())
}

pub fn bind_mount(source: &Path, destination: &Path, readonly: bool) -> Result<()> {
    fs::create_dir_all(destination)
        .map_err(|e| QckerError::mount(format!("Failed to create mount point: {}", e)))?;

    let mut flags = MsFlags::MS_BIND;
    if readonly {
        flags |= MsFlags::MS_RDONLY;
    }

    mount(Some(source), destination, None::<&str>, flags, None::<&str>)
        .map_err(|e| QckerError::mount(format!("Failed to bind mount: {}", e)))?;

    Ok(())
}

pub fn pivot_root(new_root: &Path, old_root: &Path) -> Result<()> {
    fs::create_dir_all(old_root)
        .map_err(|e| QckerError::mount(format!("Failed to create old_root: {}", e)))?;

    mount(
        Some(new_root),
        new_root,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| QckerError::mount(format!("Failed to bind mount new_root: {}", e)))?;

    unsafe {
        let new_root_c = std::ffi::CString::new(new_root.to_str().unwrap()).unwrap();
        let old_root_c = std::ffi::CString::new(old_root.to_str().unwrap()).unwrap();
        let ret = libc::syscall(libc::SYS_pivot_root, new_root_c.as_ptr(), old_root_c.as_ptr());
        if ret != 0 {
            let err = std::io::Error::last_os_error();
            tracing::warn!("pivot_root failed: {} (fallback to chroot)", err);
            return Err(QckerError::mount(format!(
                "pivot_root failed: {}. Using chroot fallback.",
                err
            )));
        }
    }

    match nix::mount::umount2(old_root, nix::mount::MntFlags::MNT_DETACH) {
        Ok(()) => {}
        Err(nix::errno::Errno::EINVAL) => {
            tracing::info!("old_root not mounted, skipping unmount");
        }
        Err(e) => {
            tracing::warn!("Failed to unmount old_root: {}", e);
        }
    }

    let _ = fs::remove_dir(old_root);

    Ok(())
}


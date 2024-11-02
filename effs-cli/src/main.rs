use effs::Effs;
use fuse3::{
    MountOptions,
    path::Session,
};
use std::env;
use tokio::signal;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mount_path = env::args_os()
        .skip(1)
        .take(1)
        .next()
        .expect("no mount point specified");

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let mut mount_options = MountOptions::default();
    mount_options
        .uid(uid)
        .gid(gid)
        .read_only(true);

    let mut mount_handle = Session::new(mount_options)
        .mount_with_unprivileged(Effs::default(), mount_path)
        .await
        .unwrap();

    let handle = &mut mount_handle;

    tokio::select! {
        res = handle => res.unwrap(),
        _ = signal::ctrl_c() => {
            mount_handle.unmount().await.unwrap()
        }
    }
}

use effs::Effs;
use fuse3::{
    MountOptions,
    raw::Session,
};
use std::env;
use tokio::signal;
use tracing::Level;

fn log_init() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    log_init();

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
        .force_readdir_plus(true)
        // not using the opendir for now, even though we may support this later.
        .no_open_dir_support(true)
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

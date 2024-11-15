use clap::Parser;
use effs::{
    Effs,
    effect::Mirror,
    source::Source,
};
use fuse3::{
    MountOptions,
    raw::Session,
};
use std::path::Path;
use tokio::signal;
use tracing::Level;

#[derive(Debug, Parser)]
struct Cli {
    mount_path: String,
    #[clap(long)]
    mirror_source: Option<String>,
}

fn log_init() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    log_init();

    let args = Cli::parse();

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

    let effs = Effs::default();
    if let Some(mirror_source) = args.mirror_source {
        effs.push_source(Source::new(
            mirror_source.into(),
            "".into(),
            Mirror,
        )).expect("error with mirror source");
        effs.build_nodes(Path::new(""))
            .expect("failed to build mirror source nodes");
    }

    let mut mount_handle = Session::new(mount_options)
        .mount_with_unprivileged(effs, args.mount_path)
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

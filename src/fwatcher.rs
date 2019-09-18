use actix::{Actor, AsyncContext, Addr, Context};
use std::time::Duration;
use std::path::Path;
use std::fs::{read_dir, DirEntry};

use crate::manager::Manager;

pub struct FWatcher {
    manager: Addr<Manager>,
}

impl FWatcher {
    pub fn new(manager: Addr<Manager>) -> Self {
        FWatcher {
            manager,
        }
    }
}

impl Actor for FWatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(42), move |act, _ctx| {
            let path = Path::new("./watch/");
            if path.is_dir() {
                if let Ok(dir) = read_dir(path) {
                    for entry in dir {
                        if let Ok(entry) = entry {
                            let _path = entry.path();
                        }
                    }
                }
            }
        });
    }
}
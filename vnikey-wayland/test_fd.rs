use std::os::unix::io::OwnedFd;
use std::fs::File;
fn convert(fd: OwnedFd) {
    let mut file = File::from(fd);
}

use std::sync::mpsc;

use super::path::*;
use super::status::*;

pub type PathSender = mpsc::Sender<PathMsg>;
pub type StatusSender = mpsc::Sender<StatusMsg>;
pub type StatusReceiver = mpsc::Receiver<StatusMsg>;
pub type PathReceiver = mpsc::Receiver<PathMsg>;

pub fn path_channel() -> (PathSender, PathReceiver) {
    mpsc::channel()
}
pub fn status_channel() -> (StatusSender, StatusReceiver) {
    mpsc::channel()
}

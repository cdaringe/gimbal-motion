use crate::mv::Move;

pub enum Cmd {
    ClearCmdQueue,
    Mv(Move),
    Halt,
    Fire,
}

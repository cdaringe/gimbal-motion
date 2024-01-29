use crate::gcode::Gcode;

pub enum Cmd {
    ClearCmdQueue,
    ProcessGcode(Gcode),
}

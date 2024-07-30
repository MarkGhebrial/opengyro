static mut PROGRAM_STATE: ProgramState = ProgramState::Initialization;

fn set_program_state(new_state: ProgramState) {
    unsafe {
        PROGRAM_STATE = new_state;
    }
}

enum ProgramState {
    Initialization,
    Flight(FlightMode),
}

enum FlightMode {
    PassThru,
    OrientationControl,
    RateOfChangeControl,
}